use scenix_camera::PerspectiveCamera;
use scenix_core::{GpuError, LightId, MaterialId, MeshId, ScenixError, TextureId};
use scenix_light::{AmbientLight, DirectionalLight, PointLight, SpotLight};
use scenix_material::{
    LambertMaterial, NormalMaterial, PbrMaterial, PhysicalMaterial, ToonMaterial, UnlitMaterial,
    WireframeMaterial,
};
use scenix_math::{Mat4, Vec2, Vec3};
use scenix_mesh::Geometry;
use scenix_scene::SceneGraph;
use scenix_texture::{Sampler, Texture2D};

use crate::gbuffer::TextureTarget;
use crate::pass::culling::collect_visible_draws;
use crate::pass::sort::{sort_opaque_front_to_back, sort_transparent_back_to_front};
use crate::{
    FrameContext, FrameStats, GBuffer, GpuScene, MaterialUniform, PackedVertex, PipelineCache,
    RenderTargetMode, RendererConfig, RendererLight, ShadowMapAtlas,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FrameUniform {
    view_projection: [[f32; 4]; 4],
    camera_position_frame: [f32; 4],
    resolution: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ObjectUniform {
    world: [[f32; 4]; 4],
}

#[derive(Debug)]
struct UniformResources {
    frame_layout: wgpu::BindGroupLayout,
    object_layout: wgpu::BindGroupLayout,
    material_layout: wgpu::BindGroupLayout,
    frame_buffer: wgpu::Buffer,
    object_buffer: wgpu::Buffer,
    material_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    object_bind_group: wgpu::BindGroup,
    material_bind_group: wgpu::BindGroup,
    object_stride: u64,
    material_stride: u64,
    draw_capacity: usize,
}

impl UniformResources {
    fn new(device: &wgpu::Device) -> Self {
        let object_stride = aligned_uniform_size::<ObjectUniform>();
        let material_stride = aligned_uniform_size::<MaterialUniform>();
        let draw_capacity = 256;
        let frame_layout = uniform_layout(
            device,
            "scenix.frame.layout",
            wgpu::ShaderStages::VERTEX,
            false,
        );
        let object_layout = uniform_layout(
            device,
            "scenix.object.layout",
            wgpu::ShaderStages::VERTEX,
            true,
        );
        let material_layout = uniform_layout(
            device,
            "scenix.material.preview.layout",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            true,
        );
        let frame_buffer = uniform_buffer(
            device,
            "scenix.frame.uniform",
            core::mem::size_of::<FrameUniform>(),
        );
        let object_buffer = uniform_buffer(
            device,
            "scenix.object.uniform",
            object_stride as usize * draw_capacity,
        );
        let material_buffer = uniform_buffer(
            device,
            "scenix.material.preview.uniform",
            material_stride as usize * draw_capacity,
        );
        let frame_bind_group = uniform_bind_group(
            device,
            "scenix.frame.bind_group",
            &frame_layout,
            &frame_buffer,
            core::mem::size_of::<FrameUniform>(),
        );
        let object_bind_group = uniform_bind_group(
            device,
            "scenix.object.bind_group",
            &object_layout,
            &object_buffer,
            core::mem::size_of::<ObjectUniform>(),
        );
        let material_bind_group = uniform_bind_group(
            device,
            "scenix.material.preview.bind_group",
            &material_layout,
            &material_buffer,
            core::mem::size_of::<MaterialUniform>(),
        );

        Self {
            frame_layout,
            object_layout,
            material_layout,
            frame_buffer,
            object_buffer,
            material_buffer,
            frame_bind_group,
            object_bind_group,
            material_bind_group,
            object_stride,
            material_stride,
            draw_capacity,
        }
    }

    fn ensure_draw_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.draw_capacity {
            return;
        }
        self.draw_capacity = needed.next_power_of_two();
        self.object_buffer = uniform_buffer(
            device,
            "scenix.object.uniform",
            self.object_stride as usize * self.draw_capacity,
        );
        self.material_buffer = uniform_buffer(
            device,
            "scenix.material.preview.uniform",
            self.material_stride as usize * self.draw_capacity,
        );
        self.object_bind_group = uniform_bind_group(
            device,
            "scenix.object.bind_group",
            &self.object_layout,
            &self.object_buffer,
            core::mem::size_of::<ObjectUniform>(),
        );
        self.material_bind_group = uniform_bind_group(
            device,
            "scenix.material.preview.bind_group",
            &self.material_layout,
            &self.material_buffer,
            core::mem::size_of::<MaterialUniform>(),
        );
    }
}

/// wgpu renderer and GPU resource owner.
pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    offscreen: Option<TextureTarget>,
    config: RendererConfig,
    target_mode: RenderTargetMode,
    pipeline_cache: PipelineCache,
    draw_pipeline: wgpu::RenderPipeline,
    uniforms: UniformResources,
    gpu_scene: GpuScene,
    gbuffer: GBuffer,
    shadow_maps: ShadowMapAtlas,
    #[cfg(feature = "post")]
    post_stack: Option<scenix_post::PostStack>,
    #[cfg(feature = "post")]
    post_source: Option<TextureTarget>,
    frame_index: u64,
}

impl Renderer {
    /// Creates a surface-backed renderer.
    pub async fn new(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        config: RendererConfig,
    ) -> Result<Self, ScenixError> {
        config.validate()?;
        let instance = instance_from_config(&config);
        let surface = instance
            .create_surface(target)
            .map_err(|_| ScenixError::Gpu(GpuError::Init))?;
        Self::from_surface(instance, surface, config).await
    }

    /// Creates a headless offscreen renderer.
    pub async fn headless(config: RendererConfig) -> Result<Self, ScenixError> {
        config.validate()?;
        let instance = instance_from_config(&config);
        let (adapter, device, queue) = request_device(&instance, None).await?;
        let color_format = config.preferred_color_format();
        let uniforms = UniformResources::new(&device);
        let draw_pipeline = create_draw_pipeline(&device, color_format, &uniforms);
        let offscreen = TextureTarget::new(
            &device,
            "scenix.headless.color",
            config.width,
            config.height,
            color_format,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let gbuffer = GBuffer::new(&device, config.width, config.height);
        let shadow_maps = ShadowMapAtlas::new(&device, 1024, 16);
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            surface_config: None,
            offscreen: Some(offscreen),
            config,
            target_mode: RenderTargetMode::Headless,
            pipeline_cache: PipelineCache::new(),
            draw_pipeline,
            uniforms,
            gpu_scene: GpuScene::new(),
            gbuffer,
            shadow_maps,
            #[cfg(feature = "post")]
            post_stack: None,
            #[cfg(feature = "post")]
            post_source: None,
            frame_index: 0,
        })
    }

    async fn from_surface(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        config: RendererConfig,
    ) -> Result<Self, ScenixError> {
        let (adapter, device, queue) = request_device(&instance, Some(&surface)).await?;
        let surface_config = configure_surface(&surface, &adapter, &device, &config);
        let uniforms = UniformResources::new(&device);
        let draw_pipeline = create_draw_pipeline(&device, surface_config.format, &uniforms);
        let gbuffer = GBuffer::new(&device, config.width, config.height);
        let shadow_maps = ShadowMapAtlas::new(&device, 1024, 16);
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: Some(surface),
            surface_config: Some(surface_config),
            offscreen: None,
            config,
            target_mode: RenderTargetMode::Surface,
            pipeline_cache: PipelineCache::new(),
            draw_pipeline,
            uniforms,
            gpu_scene: GpuScene::new(),
            gbuffer,
            shadow_maps,
            #[cfg(feature = "post")]
            post_stack: None,
            #[cfg(feature = "post")]
            post_source: None,
            frame_index: 0,
        })
    }

    /// Resizes renderer-owned targets.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), ScenixError> {
        self.config = self.config.clone().resized(width, height);
        self.config.validate()?;
        self.gbuffer.resize(&self.device, width, height);
        #[cfg(feature = "post")]
        {
            self.post_source = None;
        }

        if let Some(surface) = &self.surface {
            let surface_config =
                configure_surface(surface, &self.adapter, &self.device, &self.config);
            self.draw_pipeline =
                create_draw_pipeline(&self.device, surface_config.format, &self.uniforms);
            self.surface_config = Some(surface_config);
        } else {
            self.offscreen = Some(TextureTarget::new(
                &self.device,
                "scenix.headless.color",
                width,
                height,
                self.config.preferred_color_format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            ));
        }
        Ok(())
    }

    /// Renders a scene with a perspective camera.
    pub fn render(
        &mut self,
        scene: &SceneGraph,
        camera: &PerspectiveCamera,
    ) -> Result<FrameStats, ScenixError> {
        let frame_context = self.frame_context(camera);
        let (mut draws, culling_stats) =
            collect_visible_draws(scene, &self.gpu_scene, camera).map_err(ScenixError::from)?;
        let mut opaque = Vec::new();
        let mut transparent = Vec::new();
        for draw in draws.drain(..) {
            if draw.transparent {
                transparent.push(draw);
            } else {
                opaque.push(draw);
            }
        }
        sort_opaque_front_to_back(&mut opaque);
        sort_transparent_back_to_front(&mut transparent);

        let stats = FrameStats {
            frame_index: self.frame_index,
            scene_meshes: culling_stats.scene_meshes,
            visible_meshes: culling_stats.visible_meshes,
            culled_meshes: culling_stats.culled_meshes,
            opaque_draws: opaque.len() as u32,
            transparent_draws: transparent.len() as u32,
            lights: self.gpu_scene.light_count() as u32,
            target_mode: Some(self.target_mode),
        };

        match self.target_mode {
            RenderTargetMode::Headless => {
                self.render_headless(frame_context, stats.visible_meshes, &opaque, &transparent)?;
            }
            RenderTargetMode::Surface => {
                self.render_surface(frame_context, stats.visible_meshes, &opaque, &transparent)?;
            }
        }

        self.frame_index = self.frame_index.saturating_add(1);
        Ok(stats)
    }

    /// Registers a mesh with the renderer.
    #[inline]
    pub fn register_mesh(
        &mut self,
        mesh_id: MeshId,
        geometry: &Geometry,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_mesh(&self.device, mesh_id, geometry)
            .map_err(ScenixError::from)
    }

    /// Registers a PBR material.
    #[inline]
    pub fn register_pbr_material(
        &mut self,
        material_id: MaterialId,
        material: &PbrMaterial,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_pbr_material(material_id, material)
            .map_err(ScenixError::from)
    }

    /// Registers a physical material.
    #[inline]
    pub fn register_physical_material(
        &mut self,
        material_id: MaterialId,
        material: &PhysicalMaterial,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_physical_material(material_id, material)
            .map_err(ScenixError::from)
    }

    /// Registers an unlit material.
    #[inline]
    pub fn register_unlit_material(
        &mut self,
        material_id: MaterialId,
        material: &UnlitMaterial,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_unlit_material(material_id, material)
            .map_err(ScenixError::from)
    }

    /// Registers a Lambert material.
    #[inline]
    pub fn register_lambert_material(
        &mut self,
        material_id: MaterialId,
        material: &LambertMaterial,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_lambert_material(material_id, material)
            .map_err(ScenixError::from)
    }

    /// Registers a toon material.
    #[inline]
    pub fn register_toon_material(
        &mut self,
        material_id: MaterialId,
        material: &ToonMaterial,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_toon_material(material_id, material)
            .map_err(ScenixError::from)
    }

    /// Registers a wireframe preview material.
    #[inline]
    pub fn register_wireframe_material(
        &mut self,
        material_id: MaterialId,
        material: &WireframeMaterial,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_wireframe_material(material_id, material)
            .map_err(ScenixError::from)
    }

    /// Registers a normal visualization material.
    #[inline]
    pub fn register_normal_material(
        &mut self,
        material_id: MaterialId,
        material: &NormalMaterial,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_normal_material(material_id, material)
            .map_err(ScenixError::from)
    }

    /// Registers CPU texture metadata and sampler state.
    #[inline]
    pub fn register_texture2d(
        &mut self,
        texture_id: TextureId,
        texture: &Texture2D,
        sampler: Sampler,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_texture2d(texture_id, texture, sampler)
            .map_err(ScenixError::from)
    }

    /// Registers a renderer light.
    #[inline]
    pub fn register_light(
        &mut self,
        light_id: LightId,
        light: RendererLight,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_light(light_id, light)
            .map_err(ScenixError::from)
    }

    /// Registers an ambient light.
    #[inline]
    pub fn register_ambient_light(
        &mut self,
        light_id: LightId,
        light: AmbientLight,
    ) -> Result<(), ScenixError> {
        self.register_light(light_id, RendererLight::Ambient(light))
    }

    /// Registers a directional light.
    #[inline]
    pub fn register_directional_light(
        &mut self,
        light_id: LightId,
        light: DirectionalLight,
    ) -> Result<(), ScenixError> {
        self.register_light(light_id, RendererLight::Directional(light))
    }

    /// Registers a point light.
    #[inline]
    pub fn register_point_light(
        &mut self,
        light_id: LightId,
        light: PointLight,
    ) -> Result<(), ScenixError> {
        self.register_light(light_id, RendererLight::Point(light))
    }

    /// Registers a spot light.
    #[inline]
    pub fn register_spot_light(
        &mut self,
        light_id: LightId,
        light: SpotLight,
    ) -> Result<(), ScenixError> {
        self.register_light(light_id, RendererLight::Spot(light))
    }

    /// Returns the wgpu device.
    #[inline]
    pub const fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns the wgpu instance.
    #[inline]
    pub const fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    /// Returns the selected wgpu adapter.
    #[inline]
    pub const fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Returns the wgpu queue.
    #[inline]
    pub const fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Returns renderer configuration.
    #[inline]
    pub const fn config(&self) -> &RendererConfig {
        &self.config
    }

    /// Returns GPU scene resources.
    #[inline]
    pub const fn gpu_scene(&self) -> &GpuScene {
        &self.gpu_scene
    }

    /// Returns the pipeline cache.
    #[inline]
    pub const fn pipeline_cache(&self) -> &PipelineCache {
        &self.pipeline_cache
    }

    /// Returns the mutable pipeline cache.
    #[inline]
    pub const fn pipeline_cache_mut(&mut self) -> &mut PipelineCache {
        &mut self.pipeline_cache
    }

    /// Returns the G-buffer.
    #[inline]
    pub const fn gbuffer(&self) -> &GBuffer {
        &self.gbuffer
    }

    /// Returns shadow maps.
    #[inline]
    pub const fn shadow_maps(&self) -> &ShadowMapAtlas {
        &self.shadow_maps
    }

    /// Returns the render target mode.
    #[inline]
    pub const fn target_mode(&self) -> RenderTargetMode {
        self.target_mode
    }

    /// Sets the optional post-processing stack.
    #[cfg(feature = "post")]
    #[inline]
    pub fn set_post_stack(&mut self, post_stack: Option<scenix_post::PostStack>) {
        self.post_stack = post_stack;
    }

    /// Returns the active post-processing stack.
    #[cfg(feature = "post")]
    #[inline]
    pub const fn post_stack(&self) -> Option<&scenix_post::PostStack> {
        self.post_stack.as_ref()
    }

    /// Returns the mutable active post-processing stack.
    #[cfg(feature = "post")]
    #[inline]
    pub fn post_stack_mut(&mut self) -> Option<&mut scenix_post::PostStack> {
        self.post_stack.as_mut()
    }

    /// Reads the first pixel from the headless render target.
    ///
    /// This is intended for smoke tests and tooling, not per-frame gameplay
    /// readback.
    pub fn read_headless_pixel(&self) -> Result<[u8; 4], ScenixError> {
        let offscreen = self
            .offscreen
            .as_ref()
            .ok_or(ScenixError::Gpu(GpuError::Unsupported))?;
        let padded_bytes_per_row = 256_u32;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.headless.readback"),
            size: padded_bytes_per_row as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("scenix.headless.readback.encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: offscreen.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|_| ScenixError::Gpu(GpuError::Upload))?;
        receiver
            .recv()
            .map_err(|_| ScenixError::Gpu(GpuError::Upload))?
            .map_err(|_| ScenixError::Gpu(GpuError::Upload))?;

        let mapped = slice.get_mapped_range();
        let pixel = [mapped[0], mapped[1], mapped[2], mapped[3]];
        drop(mapped);
        buffer.unmap();
        Ok(pixel)
    }

    fn render_headless(
        &mut self,
        frame_context: FrameContext,
        visible_meshes: u32,
        opaque: &[crate::DrawSubmission],
        transparent: &[crate::DrawSubmission],
    ) -> Result<(), ScenixError> {
        #[cfg(feature = "post")]
        let color_format = self.config.preferred_color_format();
        #[cfg(feature = "post")]
        {
            if self
                .post_stack
                .as_ref()
                .is_some_and(|stack| !stack.is_empty())
            {
                self.ensure_post_source(color_format)?;
                let source_view = self
                    .post_source
                    .as_ref()
                    .unwrap()
                    .texture()
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.render_scene_to_view(
                    &source_view,
                    frame_context,
                    visible_meshes,
                    opaque,
                    transparent,
                );
                let final_view = self
                    .offscreen
                    .as_ref()
                    .ok_or(ScenixError::Gpu(GpuError::Init))?
                    .texture()
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let context = scenix_post::PostContext {
                    frame_index: self.frame_index,
                    resolution: Vec2::new(self.config.width as f32, self.config.height as f32),
                    color_format,
                };
                self.post_stack.as_mut().unwrap().apply_to_view(
                    &self.device,
                    &self.queue,
                    &source_view,
                    &final_view,
                    context,
                )?;
                return Ok(());
            }
        }

        let view = self
            .offscreen
            .as_ref()
            .ok_or(ScenixError::Gpu(GpuError::Init))?
            .texture()
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.render_scene_to_view(&view, frame_context, visible_meshes, opaque, transparent);
        Ok(())
    }

    fn render_surface(
        &mut self,
        frame_context: FrameContext,
        visible_meshes: u32,
        opaque: &[crate::DrawSubmission],
        transparent: &[crate::DrawSubmission],
    ) -> Result<(), ScenixError> {
        let surface = self
            .surface
            .as_ref()
            .ok_or(ScenixError::Gpu(GpuError::Init))?;
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let config = configure_surface(surface, &self.adapter, &self.device, &self.config);
                self.surface_config = Some(config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(ScenixError::Gpu(GpuError::Unsupported));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let format = self.surface_config.as_ref().map_or_else(
            || self.config.preferred_color_format(),
            |config| config.format,
        );
        self.render_to_final_view(
            &view,
            format,
            frame_context,
            visible_meshes,
            opaque,
            transparent,
        )?;
        frame.present();
        Ok(())
    }

    fn render_to_final_view(
        &mut self,
        view: &wgpu::TextureView,
        #[cfg_attr(not(feature = "post"), allow(unused_variables))] format: wgpu::TextureFormat,
        frame_context: FrameContext,
        visible_meshes: u32,
        opaque: &[crate::DrawSubmission],
        transparent: &[crate::DrawSubmission],
    ) -> Result<(), ScenixError> {
        #[cfg(feature = "post")]
        {
            if self
                .post_stack
                .as_ref()
                .is_some_and(|stack| !stack.is_empty())
            {
                self.ensure_post_source(format)?;
                let source_view = self
                    .post_source
                    .as_ref()
                    .unwrap()
                    .texture()
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.render_scene_to_view(
                    &source_view,
                    frame_context,
                    visible_meshes,
                    opaque,
                    transparent,
                );
                let context = scenix_post::PostContext {
                    frame_index: self.frame_index,
                    resolution: Vec2::new(self.config.width as f32, self.config.height as f32),
                    color_format: format,
                };
                self.post_stack.as_mut().unwrap().apply_to_view(
                    &self.device,
                    &self.queue,
                    &source_view,
                    view,
                    context,
                )?;
                return Ok(());
            }
        }

        self.render_scene_to_view(view, frame_context, visible_meshes, opaque, transparent);
        Ok(())
    }

    #[cfg(feature = "post")]
    fn ensure_post_source(&mut self, format: wgpu::TextureFormat) -> Result<(), ScenixError> {
        let replace = self.post_source.as_ref().is_none_or(|target| {
            target.width() != self.config.width
                || target.height() != self.config.height
                || target.format() != format
        });
        if replace {
            self.post_source = Some(TextureTarget::new(
                &self.device,
                "scenix.post.source",
                self.config.width,
                self.config.height,
                format,
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            ));
        }
        Ok(())
    }

    fn render_scene_to_view(
        &mut self,
        view: &wgpu::TextureView,
        frame_context: FrameContext,
        visible_meshes: u32,
        opaque: &[crate::DrawSubmission],
        transparent: &[crate::DrawSubmission],
    ) {
        let frame_uniform = FrameUniform {
            view_projection: mat4_uniform(frame_context.view_projection),
            camera_position_frame: [
                frame_context.camera_position.x,
                frame_context.camera_position.y,
                frame_context.camera_position.z,
                frame_context.frame_index as f32,
            ],
            resolution: [
                frame_context.resolution.x,
                frame_context.resolution.y,
                1.0 / frame_context.resolution.x.max(1.0),
                1.0 / frame_context.resolution.y.max(1.0),
            ],
        };
        self.queue.write_buffer(
            &self.uniforms.frame_buffer,
            0,
            bytemuck::bytes_of(&frame_uniform),
        );
        let draw_count = opaque.len() + transparent.len();
        self.uniforms.ensure_draw_capacity(&self.device, draw_count);
        for (draw_index, draw) in opaque.iter().chain(transparent.iter()).enumerate() {
            let Some(material) = self.gpu_scene.material(draw.material_id) else {
                continue;
            };
            let object_uniform = ObjectUniform {
                world: mat4_uniform(draw.world_matrix),
            };
            let material_uniform = MaterialUniform::new(
                material.preview_color(),
                Vec3::ZERO,
                0.0,
                0.8,
                None,
                material.preview_shader_code(),
                material.pipeline_key().feature_bits,
            );
            self.queue.write_buffer(
                &self.uniforms.object_buffer,
                draw_index as u64 * self.uniforms.object_stride,
                bytemuck::bytes_of(&object_uniform),
            );
            self.queue.write_buffer(
                &self.uniforms.material_buffer,
                draw_index as u64 * self.uniforms.material_stride,
                bytemuck::bytes_of(&material_uniform),
            );
        }

        let clear = if visible_meshes > 0 {
            wgpu::Color {
                r: 0.12,
                g: 0.22,
                b: 0.34,
                a: 1.0,
            }
        } else {
            self.config.clear_color
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("scenix.frame.encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scenix.frame.clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.gbuffer.depth().view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if visible_meshes > 0 {
                pass.set_pipeline(&self.draw_pipeline);
                pass.set_bind_group(0, &self.uniforms.frame_bind_group, &[]);
                for (draw_index, draw) in opaque.iter().chain(transparent.iter()).enumerate() {
                    let Some(mesh) = self.gpu_scene.mesh(draw.mesh_id) else {
                        continue;
                    };
                    if self.gpu_scene.material(draw.material_id).is_none() {
                        continue;
                    }
                    pass.set_bind_group(
                        1,
                        &self.uniforms.object_bind_group,
                        &[(draw_index as u64 * self.uniforms.object_stride) as u32],
                    );
                    pass.set_bind_group(
                        2,
                        &self.uniforms.material_bind_group,
                        &[(draw_index as u64 * self.uniforms.material_stride) as u32],
                    );
                    pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                    pass.set_index_buffer(
                        mesh.index_buffer().slice(..),
                        mesh.packed().index_format.to_wgpu(),
                    );
                    pass.draw_indexed(0..mesh.packed().index_count, 0, 0..1);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
    }

    fn frame_context(&self, camera: &PerspectiveCamera) -> FrameContext {
        FrameContext {
            frame_index: self.frame_index,
            resolution: Vec2::new(self.config.width as f32, self.config.height as f32),
            view: camera.view_matrix(),
            projection: camera.projection_matrix(),
            view_projection: camera.view_projection(),
            camera_position: camera.position,
        }
    }
}

fn instance_from_config(config: &RendererConfig) -> wgpu::Instance {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = config.backends;
    wgpu::Instance::new(descriptor)
}

fn create_draw_pipeline(
    device: &wgpu::Device,
    color_format: wgpu::TextureFormat,
    uniforms: &UniformResources,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("scenix.mesh_color.shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mesh_color.wgsl").into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("scenix.mesh_color.layout"),
        bind_group_layouts: &[
            Some(&uniforms.frame_layout),
            Some(&uniforms.object_layout),
            Some(&uniforms.material_layout),
        ],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("scenix.mesh_color.pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[PackedVertex::layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn uniform_layout(
    device: &wgpu::Device,
    label: &'static str,
    visibility: wgpu::ShaderStages,
    dynamic: bool,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn uniform_buffer(device: &wgpu::Device, label: &'static str, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(1) as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn uniform_bind_group(
    device: &wgpu::Device,
    label: &'static str,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    binding_size: usize,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: wgpu::BufferSize::new(binding_size as u64),
            }),
        }],
    })
}

fn aligned_uniform_size<T>() -> u64 {
    let size = core::mem::size_of::<T>() as u64;
    (size + 255) & !255
}

fn mat4_uniform(matrix: Mat4) -> [[f32; 4]; 4] {
    [
        [
            matrix.cols[0].x,
            matrix.cols[0].y,
            matrix.cols[0].z,
            matrix.cols[0].w,
        ],
        [
            matrix.cols[1].x,
            matrix.cols[1].y,
            matrix.cols[1].z,
            matrix.cols[1].w,
        ],
        [
            matrix.cols[2].x,
            matrix.cols[2].y,
            matrix.cols[2].z,
            matrix.cols[2].w,
        ],
        [
            matrix.cols[3].x,
            matrix.cols[3].y,
            matrix.cols[3].z,
            matrix.cols[3].w,
        ],
    ]
}

async fn request_device(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), ScenixError> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: surface,
            force_fallback_adapter: false,
        })
        .await
        .map_err(|_| ScenixError::Gpu(GpuError::Init))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("scenix.renderer.device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        })
        .await
        .map_err(|_| ScenixError::Gpu(GpuError::Init))?;
    Ok((adapter, device, queue))
}

fn configure_surface(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    config: &RendererConfig,
) -> wgpu::SurfaceConfiguration {
    let caps = surface.get_capabilities(adapter);
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|format| *format == config.preferred_color_format())
        .or_else(|| caps.formats.first().copied())
        .unwrap_or_else(|| config.preferred_color_format());
    let present_mode = if caps.present_modes.contains(&config.present_mode) {
        config.present_mode
    } else {
        wgpu::PresentMode::Fifo
    };
    let alpha_mode = caps
        .alpha_modes
        .first()
        .copied()
        .unwrap_or(wgpu::CompositeAlphaMode::Auto);
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: config.width,
        height: config.height,
        present_mode,
        desired_maximum_frame_latency: 2,
        alpha_mode,
        view_formats: vec![],
    };
    surface.configure(device, &surface_config);
    surface_config
}
