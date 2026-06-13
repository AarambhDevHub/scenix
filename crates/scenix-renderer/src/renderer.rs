use std::collections::HashMap;

use scenix_camera::PerspectiveCamera;
use scenix_core::{GpuError, LightId, MaterialId, MeshId, ScenixError, TextureId};
use scenix_light::{
    AmbientLight, AreaLight, DirectionalLight, HemisphereLight, LightProbe, PointLight, SpotLight,
};
use scenix_material::{
    LambertMaterial, Material, NormalMaterial, PbrMaterial, PhysicalMaterial, ToonMaterial,
    UnlitMaterial, WireframeMaterial,
};
use scenix_math::{Mat4, Vec2, Vec3};
use scenix_mesh::Geometry;
use scenix_scene::SceneGraph;
use scenix_texture::{Sampler, Texture2D, Texture3D, TextureCube, TextureFormat};

use crate::gbuffer::TextureTarget;
use crate::pass::culling::collect_visible_draws;
use crate::pass::sort::{sort_opaque_front_to_back, sort_transparent_back_to_front};
use crate::{
    EnvironmentMap, FrameContext, FrameStats, GBuffer, GpuScene, MaterialUniform, PackedVertex,
    PipelineCache, PipelineCacheStats, RenderTargetDescriptor, RenderTargetKind, RenderTargetMode,
    RendererConfig, RendererDiagnostics, RendererLight, RendererMaterial, ResourceStats,
    ShadowMapAtlas,
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

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    ambient: [f32; 4],
    directional_direction: [f32; 4],
    directional_color: [f32; 4],
    point_position_range: [f32; 4],
    point_color: [f32; 4],
    spot_direction_angle: [f32; 4],
    spot_color: [f32; 4],
    environment: [f32; 4],
    counts: [f32; 4],
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            ambient: [0.08, 0.08, 0.08, 1.0],
            directional_direction: [-0.35, -0.8, -0.45, 0.0],
            directional_color: [1.0, 1.0, 1.0, 0.0],
            point_position_range: [0.0, 0.0, 0.0, 0.0],
            point_color: [0.0, 0.0, 0.0, 0.0],
            spot_direction_angle: [0.0, -1.0, 0.0, 0.0],
            spot_color: [0.0, 0.0, 0.0, 0.0],
            environment: [0.0, 0.0, 0.0, 0.0],
            counts: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct TextureGpuResource {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    material_bind_group: Option<wgpu::BindGroup>,
    material_bindable: bool,
    byte_len: u64,
}

#[derive(Debug)]
struct UniformResources {
    frame_layout: wgpu::BindGroupLayout,
    object_layout: wgpu::BindGroupLayout,
    material_layout: wgpu::BindGroupLayout,
    light_layout: wgpu::BindGroupLayout,
    frame_buffer: wgpu::Buffer,
    object_buffer: wgpu::Buffer,
    material_buffer: wgpu::Buffer,
    light_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    object_bind_group: wgpu::BindGroup,
    material_bind_group: wgpu::BindGroup,
    light_bind_group: wgpu::BindGroup,
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
        let material_layout = material_layout(device);
        let light_layout = uniform_layout(
            device,
            "scenix.light.layout",
            wgpu::ShaderStages::FRAGMENT,
            false,
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
        let light_buffer = uniform_buffer(
            device,
            "scenix.light.uniform",
            core::mem::size_of::<LightUniform>(),
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
        let material_bind_group = default_material_bind_group(
            device,
            "scenix.material.preview.bind_group",
            &material_layout,
            &material_buffer,
            core::mem::size_of::<MaterialUniform>(),
        );
        let light_bind_group = uniform_bind_group(
            device,
            "scenix.light.bind_group",
            &light_layout,
            &light_buffer,
            core::mem::size_of::<LightUniform>(),
        );
        Self {
            frame_layout,
            object_layout,
            material_layout,
            light_layout,
            frame_buffer,
            object_buffer,
            material_buffer,
            light_buffer,
            frame_bind_group,
            object_bind_group,
            material_bind_group,
            light_bind_group,
            object_stride,
            material_stride,
            draw_capacity,
        }
    }

    fn ensure_draw_capacity(&mut self, device: &wgpu::Device, needed: usize) -> bool {
        if needed <= self.draw_capacity {
            return false;
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
        self.material_bind_group = default_material_bind_group(
            device,
            "scenix.material.preview.bind_group",
            &self.material_layout,
            &self.material_buffer,
            core::mem::size_of::<MaterialUniform>(),
        );
        true
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
    texture_resources: HashMap<TextureId, TextureGpuResource>,
    render_targets: HashMap<TextureId, TextureTarget>,
    environment: Option<EnvironmentMap>,
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
            texture_resources: HashMap::new(),
            render_targets: HashMap::new(),
            environment: None,
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
            texture_resources: HashMap::new(),
            render_targets: HashMap::new(),
            environment: None,
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
        let (opaque, transparent, stats) = self.prepare_draws(scene, camera)?;

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
            .map_err(ScenixError::from)?;
        let resource = upload_texture2d(
            &self.device,
            &self.queue,
            &self.uniforms.material_layout,
            &self.uniforms.material_buffer,
            texture,
            sampler,
        )?;
        self.texture_resources.insert(texture_id, resource);
        Ok(())
    }

    /// Updates an existing 2D texture or registers it if the ID is new.
    #[inline]
    pub fn update_texture2d(
        &mut self,
        texture_id: TextureId,
        texture: &Texture2D,
        sampler: Sampler,
    ) -> Result<(), ScenixError> {
        self.register_texture2d(texture_id, texture, sampler)
    }

    /// Registers a cube texture.
    #[inline]
    pub fn register_texture_cube(
        &mut self,
        texture_id: TextureId,
        texture: &TextureCube,
        sampler: Sampler,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_texture_cube(texture_id, texture, sampler)
            .map_err(ScenixError::from)?;
        let resource = upload_texture_cube(&self.device, &self.queue, texture, sampler)?;
        self.texture_resources.insert(texture_id, resource);
        Ok(())
    }

    /// Registers a 3D texture.
    #[inline]
    pub fn register_texture3d(
        &mut self,
        texture_id: TextureId,
        texture: &Texture3D,
        sampler: Sampler,
    ) -> Result<(), ScenixError> {
        self.gpu_scene
            .register_texture3d(texture_id, texture, sampler)
            .map_err(ScenixError::from)?;
        let resource = upload_texture3d(&self.device, &self.queue, texture, sampler)?;
        self.texture_resources.insert(texture_id, resource);
        Ok(())
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

    /// Registers a hemisphere light.
    #[inline]
    pub fn register_hemisphere_light(
        &mut self,
        light_id: LightId,
        light: HemisphereLight,
    ) -> Result<(), ScenixError> {
        self.register_light(light_id, RendererLight::Hemisphere(light))
    }

    /// Registers an area light.
    #[inline]
    pub fn register_area_light(
        &mut self,
        light_id: LightId,
        light: AreaLight,
    ) -> Result<(), ScenixError> {
        self.register_light(light_id, RendererLight::Area(light))
    }

    /// Registers a light probe.
    #[inline]
    pub fn register_light_probe(
        &mut self,
        light_id: LightId,
        light: LightProbe,
    ) -> Result<(), ScenixError> {
        self.register_light(light_id, RendererLight::Probe(light))
    }

    /// Removes a mesh from renderer-owned resources.
    #[inline]
    pub fn unregister_mesh(&mut self, mesh_id: MeshId) -> bool {
        self.gpu_scene.unregister_mesh(mesh_id)
    }

    /// Removes a material from renderer-owned resources.
    #[inline]
    pub fn unregister_material(&mut self, material_id: MaterialId) -> bool {
        self.gpu_scene.unregister_material(material_id)
    }

    /// Removes a texture and any associated GPU resource.
    #[inline]
    pub fn unregister_texture(&mut self, texture_id: TextureId) -> bool {
        let metadata = self.gpu_scene.unregister_texture(texture_id);
        let resource = self.texture_resources.remove(&texture_id).is_some();
        metadata || resource
    }

    /// Removes a light from renderer-owned resources.
    #[inline]
    pub fn unregister_light(&mut self, light_id: LightId) -> bool {
        self.gpu_scene.unregister_light(light_id)
    }

    /// Clears all registered textures.
    #[inline]
    pub fn clear_textures(&mut self) {
        self.gpu_scene.clear_textures();
        self.texture_resources.clear();
    }

    /// Clears all registered render targets.
    #[inline]
    pub fn clear_render_targets(&mut self) {
        self.render_targets.clear();
    }

    /// Sets the active image-based-lighting environment.
    pub fn set_environment_map(&mut self, environment: EnvironmentMap) -> Result<(), ScenixError> {
        if environment.texture_id.is_null()
            || self.gpu_scene.texture(environment.texture_id).is_none()
        {
            return Err(ScenixError::Validation(
                scenix_core::ValidationError::InvalidId,
            ));
        }
        self.environment = Some(environment);
        Ok(())
    }

    /// Clears the active image-based-lighting environment.
    #[inline]
    pub fn clear_environment_map(&mut self) {
        self.environment = None;
    }

    /// Creates or replaces a renderer-owned render target keyed by `TextureId`.
    pub fn create_render_target(
        &mut self,
        texture_id: TextureId,
        descriptor: RenderTargetDescriptor,
    ) -> Result<(), ScenixError> {
        if texture_id.is_null() || descriptor.width == 0 || descriptor.height == 0 {
            return Err(ScenixError::Validation(
                scenix_core::ValidationError::OutOfRange,
            ));
        }
        if descriptor.sample_count != 1 {
            return Err(ScenixError::Gpu(GpuError::Unsupported));
        }
        let usage = match descriptor.kind {
            RenderTargetKind::Depth => {
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
            }
            RenderTargetKind::Color2D | RenderTargetKind::Hdr2D | RenderTargetKind::Cube => {
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
            }
        };
        let target = TextureTarget::new(
            &self.device,
            "scenix.render_target",
            descriptor.width,
            descriptor.height,
            descriptor.format,
            usage,
        );
        self.render_targets.insert(texture_id, target);
        Ok(())
    }

    /// Renders a scene into a registered render target.
    pub fn render_to_texture(
        &mut self,
        texture_id: TextureId,
        scene: &SceneGraph,
        camera: &PerspectiveCamera,
    ) -> Result<FrameStats, ScenixError> {
        let (view, format) = {
            let target = self
                .render_targets
                .get(&texture_id)
                .ok_or(ScenixError::Validation(
                    scenix_core::ValidationError::InvalidId,
                ))?;
            (
                target
                    .texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                target.format(),
            )
        };
        let frame_context = self.frame_context(camera);
        let (opaque, transparent, stats) = self.prepare_draws(scene, camera)?;
        self.render_to_final_view(
            &view,
            format,
            frame_context,
            stats.visible_meshes,
            &opaque,
            &transparent,
        )?;
        self.frame_index = self.frame_index.saturating_add(1);
        Ok(stats)
    }

    /// Reads one RGBA8 pixel from a registered color render target.
    pub fn read_texture_pixel(
        &self,
        texture_id: TextureId,
        x: u32,
        y: u32,
    ) -> Result<[u8; 4], ScenixError> {
        let target = self
            .render_targets
            .get(&texture_id)
            .ok_or(ScenixError::Validation(
                scenix_core::ValidationError::InvalidId,
            ))?;
        read_target_pixel(&self.device, &self.queue, target, x, y)
    }

    /// Returns renderer diagnostics.
    pub fn diagnostics(&self) -> RendererDiagnostics {
        RendererDiagnostics {
            frame_index: self.frame_index,
            meshes: self.gpu_scene.mesh_count() as u32,
            materials: self.gpu_scene.material_count() as u32,
            textures: self.gpu_scene.textures().len() as u32,
            lights: self.gpu_scene.light_count() as u32,
            render_targets: self.render_targets.len() as u32,
            texture_memory_bytes: self.texture_memory_bytes(),
            geometry_memory_bytes: self.geometry_memory_bytes(),
            uniform_memory_bytes: self.uniform_memory_bytes(),
            pipeline_cache_entries: self.pipeline_cache.len() as u32,
            shadow_slots: self.shadow_maps.layers(),
        }
    }

    /// Returns renderer resource memory counters.
    #[inline]
    pub fn resource_stats(&self) -> ResourceStats {
        ResourceStats {
            geometry_bytes: self.geometry_memory_bytes(),
            texture_bytes: self.texture_memory_bytes(),
            uniform_bytes: self.uniform_memory_bytes(),
        }
    }

    /// Returns pipeline cache counters.
    #[inline]
    pub fn pipeline_cache_stats(&self) -> PipelineCacheStats {
        PipelineCacheStats {
            entries: self.pipeline_cache.len() as u32,
        }
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
        read_target_pixel(&self.device, &self.queue, offscreen, 0, 0)
    }

    fn prepare_draws(
        &self,
        scene: &SceneGraph,
        camera: &PerspectiveCamera,
    ) -> Result<
        (
            Vec<crate::DrawSubmission>,
            Vec<crate::DrawSubmission>,
            FrameStats,
        ),
        ScenixError,
    > {
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

        Ok((opaque, transparent, stats))
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
        let light_uniform = self.light_uniform();
        self.queue.write_buffer(
            &self.uniforms.light_buffer,
            0,
            bytemuck::bytes_of(&light_uniform),
        );
        let draw_count = opaque.len() + transparent.len();
        if self.uniforms.ensure_draw_capacity(&self.device, draw_count) {
            self.rebuild_material_texture_bind_groups();
        }
        let mut object_bytes = vec![0_u8; self.uniforms.object_stride as usize * draw_count.max(1)];
        let mut material_bytes =
            vec![0_u8; self.uniforms.material_stride as usize * draw_count.max(1)];
        for (draw_index, draw) in opaque.iter().chain(transparent.iter()).enumerate() {
            let Some(material) = self.gpu_scene.material(draw.material_id) else {
                continue;
            };
            let object_uniform = ObjectUniform {
                world: mat4_uniform(draw.world_matrix),
            };
            let material_uniform = material_uniform(material);
            let object_offset = self.uniforms.object_stride as usize * draw_index;
            let material_offset = self.uniforms.material_stride as usize * draw_index;
            object_bytes[object_offset..object_offset + core::mem::size_of::<ObjectUniform>()]
                .copy_from_slice(bytemuck::bytes_of(&object_uniform));
            material_bytes
                [material_offset..material_offset + core::mem::size_of::<MaterialUniform>()]
                .copy_from_slice(bytemuck::bytes_of(&material_uniform));
        }
        if draw_count > 0 {
            self.queue
                .write_buffer(&self.uniforms.object_buffer, 0, &object_bytes);
            self.queue
                .write_buffer(&self.uniforms.material_buffer, 0, &material_bytes);
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
                pass.set_bind_group(3, &self.uniforms.light_bind_group, &[]);
                for (draw_index, draw) in opaque.iter().chain(transparent.iter()).enumerate() {
                    let Some(mesh) = self.gpu_scene.mesh(draw.mesh_id) else {
                        continue;
                    };
                    let Some(material) = self.gpu_scene.material(draw.material_id) else {
                        continue;
                    };
                    pass.set_bind_group(
                        1,
                        &self.uniforms.object_bind_group,
                        &[(draw_index as u64 * self.uniforms.object_stride) as u32],
                    );
                    pass.set_bind_group(
                        2,
                        self.material_bind_group(material),
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

    fn material_bind_group(&self, material: &RendererMaterial) -> &wgpu::BindGroup {
        material_albedo_texture(material)
            .and_then(|texture_id| self.texture_resources.get(&texture_id))
            .and_then(|resource| resource.material_bind_group.as_ref())
            .unwrap_or(&self.uniforms.material_bind_group)
    }

    fn rebuild_material_texture_bind_groups(&mut self) {
        for resource in self.texture_resources.values_mut() {
            if !resource.material_bindable {
                resource.material_bind_group = None;
                continue;
            }
            resource.material_bind_group = Some(material_bind_group(
                &self.device,
                "scenix.material.texture.bind_group",
                &self.uniforms.material_layout,
                &self.uniforms.material_buffer,
                core::mem::size_of::<MaterialUniform>(),
                &resource.sampler,
                &resource.view,
            ));
        }
    }

    fn light_uniform(&self) -> LightUniform {
        let mut uniform = LightUniform::default();
        let mut directional_count = 0.0_f32;
        let mut point_count = 0.0_f32;
        let mut spot_count = 0.0_f32;
        let mut extra_count = 0.0_f32;

        for (_, light) in self.gpu_scene.lights().take(32) {
            match light {
                RendererLight::Ambient(light) => {
                    uniform.ambient[0] += light.color.r * light.intensity;
                    uniform.ambient[1] += light.color.g * light.intensity;
                    uniform.ambient[2] += light.color.b * light.intensity;
                }
                RendererLight::Hemisphere(light) => {
                    uniform.ambient[0] +=
                        (light.sky_color.r + light.ground_color.r) * 0.5 * light.intensity;
                    uniform.ambient[1] +=
                        (light.sky_color.g + light.ground_color.g) * 0.5 * light.intensity;
                    uniform.ambient[2] +=
                        (light.sky_color.b + light.ground_color.b) * 0.5 * light.intensity;
                    extra_count += 1.0;
                }
                RendererLight::Directional(light) => {
                    if directional_count == 0.0 {
                        uniform.directional_direction = [
                            light.direction.x,
                            light.direction.y,
                            light.direction.z,
                            light.shadow.map_or(0.0, |_| 1.0),
                        ];
                        uniform.directional_color =
                            [light.color.r, light.color.g, light.color.b, light.intensity];
                    }
                    directional_count += 1.0;
                }
                RendererLight::Point(light) => {
                    if point_count == 0.0 {
                        uniform.point_position_range = [0.0, 2.0, 2.5, light.range];
                        uniform.point_color =
                            [light.color.r, light.color.g, light.color.b, light.intensity];
                    }
                    point_count += 1.0;
                }
                RendererLight::Spot(light) => {
                    if spot_count == 0.0 {
                        uniform.spot_direction_angle = [0.0, -1.0, 0.0, light.angle];
                        uniform.spot_color =
                            [light.color.r, light.color.g, light.color.b, light.intensity];
                    }
                    spot_count += 1.0;
                }
                RendererLight::Area(light) => {
                    uniform.ambient[0] += light.color.r * light.intensity * 0.08;
                    uniform.ambient[1] += light.color.g * light.intensity * 0.08;
                    uniform.ambient[2] += light.color.b * light.intensity * 0.08;
                    extra_count += 1.0;
                }
                RendererLight::Probe(light) => {
                    uniform.environment[0] += light.sh_coefficients[0].x * light.intensity;
                    uniform.environment[1] += light.sh_coefficients[0].y * light.intensity;
                    uniform.environment[2] += light.sh_coefficients[0].z * light.intensity;
                    extra_count += 1.0;
                }
            }
        }

        if let Some(environment) = self.environment {
            uniform.environment[0] += 0.08 * environment.intensity;
            uniform.environment[1] += 0.1 * environment.intensity;
            uniform.environment[2] += 0.14 * environment.intensity;
            uniform.environment[3] = environment.intensity;
            if let Some(RendererLight::Probe(probe)) = environment
                .light_probe
                .and_then(|light_id| self.gpu_scene.lights().find(|(id, _)| **id == light_id))
                .map(|(_, light)| light)
            {
                uniform.environment[0] += probe.sh_coefficients[0].x * probe.intensity;
                uniform.environment[1] += probe.sh_coefficients[0].y * probe.intensity;
                uniform.environment[2] += probe.sh_coefficients[0].z * probe.intensity;
            }
        }

        uniform.counts = [directional_count, point_count, spot_count, extra_count];
        uniform
    }

    fn geometry_memory_bytes(&self) -> u64 {
        self.gpu_scene.geometry_memory_bytes()
    }

    fn texture_memory_bytes(&self) -> u64 {
        let texture_bytes = self
            .texture_resources
            .values()
            .map(|resource| resource.byte_len)
            .sum::<u64>();
        let target_bytes = self
            .render_targets
            .values()
            .map(|target| texture_target_byte_len(target.width(), target.height(), target.format()))
            .sum::<u64>();
        texture_bytes + target_bytes
    }

    fn uniform_memory_bytes(&self) -> u64 {
        self.uniforms.object_stride * self.uniforms.draw_capacity as u64
            + self.uniforms.material_stride * self.uniforms.draw_capacity as u64
            + core::mem::size_of::<FrameUniform>() as u64
            + core::mem::size_of::<LightUniform>() as u64
    }
}

fn material_uniform(material: &RendererMaterial) -> MaterialUniform {
    match material {
        RendererMaterial::Pbr(material) => MaterialUniform::new(
            material.albedo,
            material.emissive,
            material.metallic,
            material.roughness,
            material.alpha_cutoff(),
            0.0,
            material.pipeline_key().feature_bits,
        ),
        RendererMaterial::Physical(material) => MaterialUniform::new(
            material.base.albedo,
            material.base.emissive,
            material.base.metallic,
            material.base.roughness,
            material.alpha_cutoff(),
            1.0,
            material.pipeline_key().feature_bits,
        ),
        RendererMaterial::Unlit(material) => MaterialUniform::new(
            material.color,
            Vec3::ZERO,
            0.0,
            1.0,
            material.alpha_cutoff(),
            2.0,
            material.pipeline_key().feature_bits,
        ),
        RendererMaterial::Lambert(material) => MaterialUniform::new(
            material.color,
            material.emissive,
            0.0,
            1.0,
            material.alpha_cutoff(),
            3.0,
            material.pipeline_key().feature_bits,
        ),
        RendererMaterial::Toon(material) => MaterialUniform::new(
            material.color,
            Vec3::ZERO,
            0.0,
            1.0,
            material.alpha_cutoff(),
            4.0,
            material.pipeline_key().feature_bits,
        ),
        RendererMaterial::Wireframe(material) => MaterialUniform::new(
            scenix_core::Color {
                a: material.opacity,
                ..material.color
            },
            Vec3::ZERO,
            0.0,
            1.0,
            material.alpha_cutoff(),
            5.0,
            material.pipeline_key().feature_bits,
        ),
        RendererMaterial::Normal(material) => MaterialUniform::new(
            scenix_core::Color::WHITE,
            Vec3::ZERO,
            0.0,
            1.0,
            material.alpha_cutoff(),
            6.0,
            material.pipeline_key().feature_bits,
        ),
    }
}

fn material_albedo_texture(material: &RendererMaterial) -> Option<TextureId> {
    match material {
        RendererMaterial::Pbr(material) => material.albedo_texture,
        RendererMaterial::Physical(material) => material.base.albedo_texture,
        RendererMaterial::Unlit(material) => material.color_texture,
        RendererMaterial::Toon(material) => material.color_texture.or(material.gradient_map),
        _ => None,
    }
}

fn upload_texture2d(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    material_buffer: &wgpu::Buffer,
    texture: &Texture2D,
    sampler: Sampler,
) -> Result<TextureGpuResource, ScenixError> {
    let format = to_supported_wgpu_format(device, texture.format)?;
    let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: texture.label.as_deref().or(Some("scenix.texture2d")),
        size: wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: texture.mip_levels.max(1),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    for level in 0..texture.mip_levels.max(1) {
        let range = texture.mip_level_range(level).map_err(ScenixError::from)?;
        let (width, height) = TextureFormat::mip_dimensions(texture.width, texture.height, level);
        write_texture_level(
            queue,
            &gpu_texture,
            TextureLevelUpload {
                format: texture.format,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                width,
                height,
                depth: 1,
                data: &texture.data[range],
            },
        )?;
    }
    let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let gpu_sampler = create_sampler(device, sampler);
    let material_bindable = texture.format != TextureFormat::Depth32Float;
    let material_bind_group = material_bindable.then(|| {
        material_bind_group(
            device,
            "scenix.material.texture.bind_group",
            layout,
            material_buffer,
            core::mem::size_of::<MaterialUniform>(),
            &gpu_sampler,
            &view,
        )
    });
    Ok(TextureGpuResource {
        texture: gpu_texture,
        view,
        sampler: gpu_sampler,
        material_bind_group,
        material_bindable,
        byte_len: texture.data.len() as u64,
    })
}

fn upload_texture_cube(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &TextureCube,
    sampler: Sampler,
) -> Result<TextureGpuResource, ScenixError> {
    let format = to_supported_wgpu_format(device, texture.format)?;
    let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: texture.label.as_deref().or(Some("scenix.texture_cube")),
        size: wgpu::Extent3d {
            width: texture.size,
            height: texture.size,
            depth_or_array_layers: 6,
        },
        mip_level_count: texture.mip_levels.max(1),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (face_index, face) in texture.faces.iter().enumerate() {
        for level in 0..texture.mip_levels.max(1) {
            let range = texture.mip_level_range(level).map_err(ScenixError::from)?;
            let (width, height) = TextureFormat::mip_dimensions(texture.size, texture.size, level);
            write_texture_level(
                queue,
                &gpu_texture,
                TextureLevelUpload {
                    format: texture.format,
                    mip_level: level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face_index as u32,
                    },
                    width,
                    height,
                    depth: 1,
                    data: &face[range],
                },
            )?;
        }
    }
    let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("scenix.texture_cube.view"),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });
    let gpu_sampler = create_sampler(device, sampler);
    Ok(TextureGpuResource {
        texture: gpu_texture,
        view,
        sampler: gpu_sampler,
        material_bind_group: None,
        material_bindable: false,
        byte_len: texture.faces.iter().map(Vec::len).sum::<usize>() as u64,
    })
}

fn upload_texture3d(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &Texture3D,
    sampler: Sampler,
) -> Result<TextureGpuResource, ScenixError> {
    let format = to_supported_wgpu_format(device, texture.format)?;
    let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: texture.label.as_deref().or(Some("scenix.texture3d")),
        size: wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: texture.depth,
        },
        mip_level_count: texture.mip_levels.max(1),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for level in 0..texture.mip_levels.max(1) {
        let range = texture.mip_level_range(level).map_err(ScenixError::from)?;
        let (width, height) = TextureFormat::mip_dimensions(texture.width, texture.height, level);
        let depth = (texture.depth >> level).max(1);
        write_texture_level(
            queue,
            &gpu_texture,
            TextureLevelUpload {
                format: texture.format,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                width,
                height,
                depth,
                data: &texture.data[range],
            },
        )?;
    }
    let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let gpu_sampler = create_sampler(device, sampler);
    Ok(TextureGpuResource {
        texture: gpu_texture,
        view,
        sampler: gpu_sampler,
        material_bind_group: None,
        material_bindable: false,
        byte_len: texture.data.len() as u64,
    })
}

struct TextureLevelUpload<'a> {
    format: TextureFormat,
    mip_level: u32,
    origin: wgpu::Origin3d,
    width: u32,
    height: u32,
    depth: u32,
    data: &'a [u8],
}

fn write_texture_level(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    upload: TextureLevelUpload<'_>,
) -> Result<(), ScenixError> {
    let bytes_per_row = texture_bytes_per_row(upload.format, upload.width)?;
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: upload.mip_level,
            origin: upload.origin,
            aspect: wgpu::TextureAspect::All,
        },
        upload.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(upload.height),
        },
        wgpu::Extent3d {
            width: upload.width,
            height: upload.height,
            depth_or_array_layers: upload.depth,
        },
    );
    Ok(())
}

fn to_supported_wgpu_format(
    device: &wgpu::Device,
    format: TextureFormat,
) -> Result<wgpu::TextureFormat, ScenixError> {
    let Some(wgpu_format) = crate::to_wgpu_texture_format(format) else {
        return Err(ScenixError::Gpu(GpuError::Unsupported));
    };
    let required = required_texture_features(format);
    if !device.features().contains(required) {
        return Err(ScenixError::Gpu(GpuError::Unsupported));
    }
    Ok(wgpu_format)
}

fn required_texture_features(format: TextureFormat) -> wgpu::Features {
    match format {
        TextureFormat::Bc7RgbaUnorm => wgpu::Features::TEXTURE_COMPRESSION_BC,
        TextureFormat::Astc4x4RgbaUnorm => wgpu::Features::TEXTURE_COMPRESSION_ASTC,
        TextureFormat::Etc2Rgba8Unorm => wgpu::Features::TEXTURE_COMPRESSION_ETC2,
        _ => wgpu::Features::empty(),
    }
}

fn texture_bytes_per_row(format: TextureFormat, width: u32) -> Result<u32, ScenixError> {
    if let Some(bytes_per_pixel) = format.bytes_per_pixel() {
        Ok(width.saturating_mul(bytes_per_pixel as u32))
    } else {
        let (block_width, _) = format
            .block_dimensions()
            .ok_or(ScenixError::Gpu(GpuError::Unsupported))?;
        let bytes_per_block = format
            .bytes_per_block()
            .ok_or(ScenixError::Gpu(GpuError::Unsupported))?;
        Ok(width.div_ceil(block_width) * bytes_per_block as u32)
    }
}

fn texture_target_byte_len(width: u32, height: u32, format: wgpu::TextureFormat) -> u64 {
    let bytes_per_pixel = match format {
        wgpu::TextureFormat::Rgba16Float => 8,
        wgpu::TextureFormat::Depth32Float
        | wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
        _ => 4,
    };
    width as u64 * height as u64 * bytes_per_pixel
}

fn create_sampler(device: &wgpu::Device, sampler: Sampler) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("scenix.texture.sampler"),
        address_mode_u: crate::to_wgpu_address_mode(sampler.address_u),
        address_mode_v: crate::to_wgpu_address_mode(sampler.address_v),
        address_mode_w: crate::to_wgpu_address_mode(sampler.address_w),
        mag_filter: crate::to_wgpu_filter_mode(sampler.mag_filter),
        min_filter: crate::to_wgpu_filter_mode(sampler.min_filter),
        mipmap_filter: to_wgpu_mipmap_filter_mode(sampler.mip_filter),
        compare: crate::to_wgpu_compare(sampler.compare),
        anisotropy_clamp: sampler.anisotropy as u16,
        ..Default::default()
    })
}

fn to_wgpu_mipmap_filter_mode(filter: scenix_texture::FilterMode) -> wgpu::MipmapFilterMode {
    match filter {
        scenix_texture::FilterMode::Nearest => wgpu::MipmapFilterMode::Nearest,
        scenix_texture::FilterMode::Linear => wgpu::MipmapFilterMode::Linear,
    }
}

fn material_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("scenix.material.preview.layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}

fn material_bind_group(
    device: &wgpu::Device,
    label: &'static str,
    layout: &wgpu::BindGroupLayout,
    material_buffer: &wgpu::Buffer,
    binding_size: usize,
    sampler: &wgpu::Sampler,
    view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: material_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(binding_size as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(view),
            },
        ],
    })
}

fn default_material_bind_group(
    device: &wgpu::Device,
    label: &'static str,
    layout: &wgpu::BindGroupLayout,
    material_buffer: &wgpu::Buffer,
    binding_size: usize,
) -> wgpu::BindGroup {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("scenix.material.default_white_texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    material_bind_group(
        device,
        label,
        layout,
        material_buffer,
        binding_size,
        &sampler,
        &view,
    )
}

fn read_target_pixel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &TextureTarget,
    x: u32,
    y: u32,
) -> Result<[u8; 4], ScenixError> {
    if x >= target.width() || y >= target.height() {
        return Err(ScenixError::Validation(
            scenix_core::ValidationError::OutOfRange,
        ));
    }
    let padded_bytes_per_row = 256_u32;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scenix.target.readback"),
        size: padded_bytes_per_row as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("scenix.target.readback.encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: target.texture(),
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
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
    queue.submit(Some(encoder.finish()));

    let slice = buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
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
            Some(&uniforms.light_layout),
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
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
    let optional_features = wgpu::Features::TEXTURE_COMPRESSION_BC
        | wgpu::Features::TEXTURE_COMPRESSION_ETC2
        | wgpu::Features::TEXTURE_COMPRESSION_ASTC;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("scenix.renderer.device"),
            required_features: adapter.features() & optional_features,
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
