//! GPU particle support: the billboard draw pipeline, the per-system
//! instance registry, and the optional compute simulation hook.
//!
//! CPU simulation (`scenix-particles`) stays the source of truth for the
//! draw path: the host runs `ParticleSystem::update`, then uploads render
//! items through `Renderer::update_particles`. The compute path is an
//! alternative where the renderer owns the pool: `register_particle_compute`
//! seeds a 64-byte pool and `simulate_particles_compute` advances it on the
//! GPU (gravity + drag + linear lifetime fades + deterministic ring
//! respawn), writing draw instances straight into the billboard buffer.
//!
//! Both paths share one 64-byte instance record:
//!
//! ```text
//! offset 0:  center.xyz + size (w)        billboard half-extent, world
//! offset 16: color.rgba                   tint premultiplied on CPU
//! offset 32: uv_rect (u0, v0, u1, v1)     scenix UV, v0 = top edge
//! offset 48: rotation.x (+ pad)           radians around the view axis
//! ```
//!
//! Rotation collapses to zero when an instance samples a sub-region
//! (flipbook): rotating a shrunken UV cell would sample outside the cell,
//! so v1.7.0 renders spin and flipbook strictly one at a time.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use scenix_core::{ParticleSystemId, ValidationError};
use scenix_math::Vec3;
use scenix_particles::{Particle, ParticleRenderItem};

/// Byte stride of one particle draw instance.
pub const PARTICLE_INSTANCE_STRIDE: u64 = 64;

/// Byte stride of one GPU compute pool record (four `vec4` fields).
pub const PARTICLE_POOL_STRIDE: u64 = 64;

/// Workgroup length for the integrate dispatch.
const SIM_WORKGROUP: u32 = 64;

/// Upper bound on instances one system can hold; mirrors the CPU pool cap
/// in `scenix-particles`.
pub const PARTICLE_DRAW_LIMIT: u32 = 1 << 18;

/// Identity UV rect sampling a whole texture.
pub const FULL_UV_RECT: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// Buffer usage shared between the CPU draw path and compute output.
#[allow(dead_code)]
fn particle_buffer_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE
}

/// One 64-byte draw instance.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ParticleInstanceUniform {
    /// World-space center + billboard half-extent.
    pub center_size: [f32; 4],
    /// Linear rgba tint (material tint × instance color).
    pub color: [f32; 4],
    /// Flipbook cell `[u0, v0, u1, v1]`, full rect without a grid.
    pub uv_rect: [f32; 4],
    /// Rotation around the view axis + padding.
    pub rotation_pad: [f32; 4],
}

impl ParticleInstanceUniform {
    /// Packs one render item, premultiplying `tint` into the color.
    #[must_use]
    pub fn pack(item: &ParticleRenderItem, tint: [f32; 4]) -> Self {
        let flipbooked = item.uv_rect != FULL_UV_RECT;
        let rotation = if flipbooked { 0.0 } else { item.rotation };
        Self {
            center_size: [
                item.position[0],
                item.position[1],
                item.position[2],
                item.size,
            ],
            color: [
                item.color[0] * tint[0],
                item.color[1] * tint[1],
                item.color[2] * tint[2],
                item.color[3] * tint[3],
            ],
            uv_rect: item.uv_rect,
            rotation_pad: [rotation, 0.0, 0.0, 0.0],
        }
    }
}

/// Per-frame billboard basis block fed to the render path.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ParticleParamsUniform {
    /// Camera right basis in world space (xyz, w unused).
    pub camera_right: [f32; 4],
    /// Camera up basis in world space (xyz, w unused).
    pub camera_up: [f32; 4],
    /// `(elapsed, aspect, dt, frame_index)`.
    pub misc: [f32; 4],
}

impl ParticleParamsUniform {
    /// Derives the camera-facing basis from a look-at camera.
    #[must_use]
    pub fn from_camera_basis(
        position: Vec3,
        target: Vec3,
        up: Vec3,
        elapsed: f32,
        aspect: f32,
        frame_index: u64,
    ) -> Self {
        let normalize = |v: Vec3| -> Vec3 {
            let length = v.length();
            if length > 1.0e-8 {
                v * (1.0 / length)
            } else {
                Vec3::ZERO
            }
        };
        let back = normalize(position - target);
        let right = normalize(up.cross(back));
        let camera_up = normalize(back.cross(right));
        let right = right.to_array();
        let up = camera_up.to_array();
        Self {
            camera_right: [right[0], right[1], right[2], 0.0],
            camera_up: [up[0], up[1], up[2], 0.0],
            misc: [elapsed, aspect, 0.0, frame_index as f32],
        }
    }
}

/// One 64-byte GPU compute pool record.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct PoolItemUniform {
    /// World-space position + age.
    pub pos_age: [f32; 4],
    /// Velocity + lifetime.
    pub vel_life: [f32; 4],
    /// Linear rgba color.
    pub color: [f32; 4],
    /// Size + rotation + padding.
    pub misc: [f32; 4],
}

impl PoolItemUniform {
    /// Seeds a pool record from a CPU-simulated particle.
    #[must_use]
    pub fn from_particle(particle: &Particle) -> Self {
        Self {
            pos_age: [
                particle.position.x,
                particle.position.y,
                particle.position.z,
                particle.age,
            ],
            vel_life: [
                particle.velocity.x,
                particle.velocity.y,
                particle.velocity.z,
                particle.lifetime,
            ],
            color: particle.color,
            misc: [particle.size, particle.rotation, 0.0, 0.0],
        }
    }
}

/// Tunables for the GPU compute simulation, one system at a time.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ParticleComputeParams {
    /// Gravity + `dt` in `w` (seconds).
    pub gravity_dt: [f32; 4],
    /// Birth color.
    pub color_from: [f32; 4],
    /// Death color.
    pub color_to: [f32; 4],
    /// `(drag, spawn_budget, hash_seed, size_from)`.
    pub knobs: [f32; 4],
    /// `(size_to, lifetime_min, lifetime_max, speed_from)`.
    pub fade: [f32; 4],
    /// Default instance UV rect.
    pub uv_rect: [f32; 4],
    /// Spawn origin + `speed_to` in `w`.
    pub origin: [f32; 4],
    /// Spawn direction (normalized), `w` unused.
    pub direction: [f32; 4],
}

impl Default for ParticleComputeParams {
    fn default() -> Self {
        Self {
            gravity_dt: [0.0, -9.8, 0.0, 1.0 / 60.0],
            color_from: [1.0, 1.0, 1.0, 1.0],
            color_to: [1.0, 1.0, 1.0, 0.0],
            knobs: [0.0, 0.0, 0.0, 1.0],
            fade: [1.0, 0.5, 1.5, 1.0],
            uv_rect: FULL_UV_RECT,
            origin: [0.0, 0.0, 0.0, 2.0],
            direction: [0.0, 1.0, 0.0, 0.0],
        }
    }
}

impl ParticleComputeParams {
    /// Returns params with a new `dt` for this simulation step.
    #[must_use]
    pub const fn with_dt(mut self, dt: f32) -> Self {
        self.gravity_dt[3] = dt;
        self
    }

    /// Sets how many dead slots the respawn pass may claim this step.
    #[must_use]
    pub const fn with_spawn_budget(mut self, budget: u32) -> Self {
        self.knobs[1] = budget as f32;
        self
    }
}

/// Sprite binding state of one slot; the bind group object rides along.
#[derive(Debug)]
pub(crate) enum SpriteGroup {
    /// White 1x1 fallback.
    Fallback(wgpu::BindGroup),
    /// A registered texture's view/sampler pair.
    Texture(wgpu::BindGroup),
}

impl SpriteGroup {
    pub(crate) fn group(&self) -> &wgpu::BindGroup {
        match self {
            Self::Fallback(group) | Self::Texture(group) => group,
        }
    }
}

pub(crate) struct ComputeSlot {
    pool: wgpu::Buffer,
    /// Persistent ring cursor for the respawn window (u32).
    ring: wgpu::Buffer,
    params: wgpu::Buffer,
    group: wgpu::BindGroup,
    pool_len: u32,
}

pub(crate) struct DrawSlot {
    material: scenix_material::ParticleMaterial,
    sprite: SpriteGroup,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
    compute: Option<ComputeSlot>,
}

/// Per-system particle registry owned by the renderer.
#[derive(Default)]
pub struct ParticleRegistry {
    slots: HashMap<ParticleSystemId, DrawSlot>,
}

impl ParticleRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    /// Registered system count.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Whether any system is registered.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Total draw instances across systems; compute systems draw the whole
    /// pool (dead lanes carry zero size and vanish in rasterization).
    pub fn instance_count(&self) -> u32 {
        self.slots
            .values()
            .map(|slot| match &slot.compute {
                Some(compute) => compute.pool_len,
                None => slot.instance_count,
            })
            .sum()
    }

    pub(crate) fn compute(&self, system_id: ParticleSystemId) -> Option<&ComputeSlot> {
        self.slots
            .get(&system_id)
            .and_then(|slot| slot.compute.as_ref())
    }

    pub(crate) fn take(&mut self, system_id: ParticleSystemId) -> Option<DrawSlot> {
        self.slots.remove(&system_id)
    }

    pub(crate) fn insert(&mut self, system_id: ParticleSystemId, slot: DrawSlot) {
        self.slots.insert(system_id, slot);
    }

    /// Deterministic register order (ascending system id) used when
    /// encoding draws.
    pub(crate) fn sorted(&self) -> Vec<(ParticleSystemId, &DrawSlot)> {
        let mut pairs: Vec<(ParticleSystemId, &DrawSlot)> =
            self.slots.iter().map(|(id, slot)| (*id, slot)).collect();
        pairs.sort_by_key(|(id, _)| id.get());
        pairs
    }
}

/// Everything GPU-owned for the particle subsystem.
pub(crate) struct ParticleResources {
    sprite_layout: wgpu::BindGroupLayout,
    registry: ParticleRegistry,
    frame_params_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,
    fallback_view: wgpu::TextureView,
    fallback_sampler: wgpu::Sampler,
    alpha_pipeline: wgpu::RenderPipeline,
    additive_pipeline: wgpu::RenderPipeline,
    compute_layout: wgpu::BindGroupLayout,
    compute_pipeline: wgpu::ComputePipeline,
    respawn_pipeline: wgpu::ComputePipeline,
}

impl ParticleResources {
    /// Creates pipelines, layouts, and the white fallback sprite.
    ///
    /// `frame_buffer` is the renderer's shared frame uniform buffer, bound
    /// at group 0 slot 0 so camera state stays in one place across passes.
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        frame_buffer: &wgpu::Buffer,
    ) -> Self {
        let frame_params_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("scenix.particles.group0.layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scenix.particles.group1.layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.particles.params"),
            size: core::mem::size_of::<ParticleParamsUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let fallback = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scenix.particles.default_sprite"),
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
        let fallback_view = fallback.create_view(&wgpu::TextureViewDescriptor::default());
        let fallback_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scenix.particles.billboard.shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/particles_billboard.wgsl").into(),
            ),
        });
        let alpha_pipeline = particle_pipeline(
            device,
            "scenix.particles.alpha.pipeline",
            color_format,
            &frame_params_layout,
            &sprite_layout,
            &shader,
            wgpu::BlendState::ALPHA_BLENDING,
        );
        let additive_pipeline = particle_pipeline(
            device,
            "scenix.particles.additive.pipeline",
            color_format,
            &frame_params_layout,
            &sprite_layout,
            &shader,
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::REPLACE,
            },
        );

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scenix.particles.compute.shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particles_compute.wgsl").into()),
        });
        let compute_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scenix.particles.compute.layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("scenix.particles.compute.pipeline_layout"),
                bind_group_layouts: &[Some(&compute_layout)],
                immediate_size: 0,
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("scenix.particles.compute.pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let respawn_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("scenix.particles.respawn.pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("cs_respawn"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        // The frame+params group binds the shared frame uniform buffer
        // (group 0 slot 0) plus the particle params block (slot 1).
        let frame_params_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scenix.particles.group0"),
            layout: &frame_params_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: frame_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        Self {
            sprite_layout,
            registry: ParticleRegistry::new(),
            frame_params_group,
            params_buffer,
            fallback_view,
            fallback_sampler,
            alpha_pipeline,
            additive_pipeline,
            compute_layout,
            compute_pipeline,
            respawn_pipeline,
        }
    }

    /// Builds the sprite bind group for a registered texture resource.
    pub(crate) fn texture_group(
        &self,
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scenix.particles.sprite.group"),
            layout: &self.sprite_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    /// Builds the white fallback sprite group.
    pub(crate) fn fallback_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        self.texture_group(device, &self.fallback_sampler, &self.fallback_view)
    }

    /// Writes the per-frame camera basis block; called once per frame.
    pub(crate) fn write_params(&mut self, queue: &wgpu::Queue, params: &ParticleParamsUniform) {
        queue.write_buffer(
            &self.params_buffer,
            0,
            bytemuck::bytes_of::<ParticleParamsUniform>(params),
        );
    }

    /// Binds and draws every live particle system; returns instance draw
    /// count for stats.
    pub(crate) fn encode_frame(&self, pass: &mut wgpu::RenderPass<'_>) -> u32 {
        let mut drawn = 0_u32;
        for (_, slot) in self.registry.sorted() {
            let count = match &slot.compute {
                Some(compute) => compute.pool_len,
                None => slot.instance_count,
            };
            if count == 0 {
                continue;
            }
            let pipeline = match slot.material.blend {
                scenix_material::ParticleBlend::Additive => &self.additive_pipeline,
                scenix_material::ParticleBlend::Alpha => &self.alpha_pipeline,
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.frame_params_group, &[]);
            pass.set_bind_group(1, slot.sprite.group(), &[]);
            pass.set_vertex_buffer(0, slot.instance_buffer.slice(..));
            pass.draw(0..6, 0..count);
            drawn += count;
        }
        drawn
    }
}

// Registry service helpers (called from Renderer methods).

/// Device/queue pair bundle keeping signatures under the argument lint cap.
pub(crate) struct GpuIo<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
}

impl ParticleResources {
    /// Installs (or merges over) a CPU-driven draw slot and uploads the
    /// instance bytes.
    pub(crate) fn store_draw_slot(
        &mut self,
        gpu: &GpuIo<'_>,
        system_id: ParticleSystemId,
        material: &scenix_material::ParticleMaterial,
        sprite: SpriteGroup,
        instances: &[ParticleInstanceUniform],
    ) -> Result<(), ValidationError> {
        let count = instances.len() as u32;
        if count > PARTICLE_DRAW_LIMIT {
            return Err(ValidationError::OutOfRange);
        }
        let bytes = core::mem::size_of_val(instances) as u64;
        let reusable = self
            .registry
            .take(system_id)
            .filter(|existing| bytes <= existing.instance_buffer.size());
        if let Some(existing) = reusable {
            gpu.queue.write_buffer(
                &existing.instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
            self.registry.insert(
                system_id,
                DrawSlot {
                    material: material.clone(),
                    sprite,
                    instance_buffer: existing.instance_buffer,
                    instance_count: count,
                    compute: None,
                },
            );
            return Ok(());
        }
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.particles.instances"),
            size: bytes.max(PARTICLE_INSTANCE_STRIDE),
            usage: particle_buffer_usage(),
            mapped_at_creation: false,
        });
        gpu.queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(instances));
        self.registry.insert(
            system_id,
            DrawSlot {
                material: material.clone(),
                sprite,
                instance_buffer: buffer,
                instance_count: count,
                compute: None,
            },
        );
        Ok(())
    }

    /// Installs a compute-owned slot seeded from a CPU pool snapshot.
    pub(crate) fn store_compute_slot(
        &mut self,
        gpu: &GpuIo<'_>,
        system_id: ParticleSystemId,
        material: &scenix_material::ParticleMaterial,
        sprite: SpriteGroup,
        source: &scenix_particles::ParticleSystem,
        params: &ParticleComputeParams,
    ) -> Result<(), ValidationError> {
        let source_pool = source.particles();
        if source_pool.len() as u32 > PARTICLE_DRAW_LIMIT {
            return Err(ValidationError::OutOfRange);
        }
        let pool_len = source_pool.len() as u32;
        let pool_records: Vec<PoolItemUniform> = source_pool
            .iter()
            .map(PoolItemUniform::from_particle)
            .collect();
        let pool = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.particles.compute.pool"),
            size: pool_len as u64 * PARTICLE_POOL_STRIDE,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        gpu.queue
            .write_buffer(&pool, 0, bytemuck::cast_slice(&pool_records));
        let ring = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.particles.compute.ring"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let cursor_zero: [u32; 1] = [0];
        gpu.queue
            .write_buffer(&ring, 0, bytemuck::bytes_of(&cursor_zero));
        let sim_params = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.particles.compute.params"),
            size: core::mem::size_of::<ParticleComputeParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(
            &sim_params,
            0,
            bytemuck::bytes_of::<ParticleComputeParams>(params),
        );
        let instance_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.particles.compute.instances"),
            size: pool_len as u64 * PARTICLE_INSTANCE_STRIDE,
            usage: particle_buffer_usage(),
            mapped_at_creation: false,
        });
        let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scenix.particles.compute.group"),
            layout: &self.compute_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &pool,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &sim_params,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ring,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &instance_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        // Reuse a previously wider instance buffer when one exists.
        let instance_buffer = match self.registry.take(system_id) {
            Some(existing)
                if existing.instance_buffer.size()
                    >= pool_len as u64 * PARTICLE_INSTANCE_STRIDE =>
            {
                existing.instance_buffer
            }
            _ => instance_buffer,
        };
        self.registry.insert(
            system_id,
            DrawSlot {
                material: material.clone(),
                sprite,
                instance_buffer,
                instance_count: 0,
                compute: Some(ComputeSlot {
                    pool,
                    ring,
                    params: sim_params,
                    group,
                    pool_len,
                }),
            },
        );
        Ok(())
    }

    /// Byte copies of the GPU compute pool for inspection and tests.
    pub(crate) fn read_pool(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        system_id: ParticleSystemId,
        output: &mut Vec<PoolItemUniform>,
    ) -> Result<(), ValidationError> {
        output.clear();
        let pool = self
            .registry
            .compute(system_id)
            .ok_or(ValidationError::InvalidId)?;
        let bytes = pool.pool_len as u64 * PARTICLE_POOL_STRIDE;
        if bytes == 0 {
            return Ok(());
        }
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.particles.compute.pool.readback"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("scenix.particles.compute.pool.readback.encoder"),
        });
        encoder.copy_buffer_to_buffer(&pool.pool, 0, &staging, 0, bytes);
        queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|_| ValidationError::InvalidState)?;
        receiver
            .recv()
            .map_err(|_| ValidationError::InvalidState)?
            .map_err(|_| ValidationError::InvalidState)?;

        let mapped = slice.get_mapped_range();
        let records: &[PoolItemUniform] = bytemuck::cast_slice(mapped.as_ref());
        output.extend(records.iter().copied());
        drop(mapped);
        staging.unmap();
        Ok(())
    }

    /// Reads the respawn ring cursor (`u32`) of a compute-owned slot.
    ///
    /// Exists for verification (tests, editors) — the simulation never needs
    /// a CPU copy of the cursor.
    pub(crate) fn read_ring(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        system_id: ParticleSystemId,
        cursor: &mut u32,
    ) -> Result<(), ValidationError> {
        let pool = self
            .registry
            .compute(system_id)
            .ok_or(ValidationError::InvalidId)?;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.particles.compute.ring.readback"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("scenix.particles.compute.ring.readback.encoder"),
        });
        encoder.copy_buffer_to_buffer(&pool.ring, 0, &staging, 0, 4);
        queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|_| ValidationError::InvalidState)?;
        receiver
            .recv()
            .map_err(|_| ValidationError::InvalidState)?
            .map_err(|_| ValidationError::InvalidState)?;
        let mapped = slice.get_mapped_range();
        let bytes: [u8; 4] = [mapped[0], mapped[1], mapped[2], mapped[3]];
        drop(mapped);
        staging.unmap();
        *cursor = u32::from_le_bytes(bytes);
        Ok(())
    }

    /// Dispatches the integrate and ring-respawn passes for one system.
    pub(crate) fn encode_simulation(
        &self,
        queue: &wgpu::Queue,
        system_id: ParticleSystemId,
        params: &ParticleComputeParams,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), ValidationError> {
        let compute = self
            .registry
            .compute(system_id)
            .ok_or(ValidationError::InvalidId)?;
        queue.write_buffer(
            &compute.params,
            0,
            bytemuck::bytes_of::<ParticleComputeParams>(params),
        );
        let workgroups = compute.pool_len.div_ceil(SIM_WORKGROUP);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("scenix.particles.sim.pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.compute_pipeline);
        pass.set_bind_group(0, &compute.group, &[]);
        pass.dispatch_workgroups(workgroups.max(1), 1, 1);
        pass.set_pipeline(&self.respawn_pipeline);
        pass.dispatch_workgroups(1, 1, 1);
        Ok(())
    }

    /// Removes a system; returns whether it existed.
    pub(crate) fn remove(&mut self, system_id: ParticleSystemId) -> bool {
        self.registry.take(system_id).is_some()
    }
}

/// Creates one billboard render pipeline in the given blend mode.
fn particle_pipeline(
    device: &wgpu::Device,
    label: &'static str,
    color_format: wgpu::TextureFormat,
    frame_params_layout: &wgpu::BindGroupLayout,
    sprite_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    blend: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("scenix.particles.pipeline_layout"),
        bind_group_layouts: &[Some(frame_params_layout), Some(sprite_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: PARTICLE_INSTANCE_STRIDE,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        offset: 16,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        offset: 32,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 2,
                    },
                    wgpu::VertexAttribute {
                        offset: 48,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 3,
                    },
                ],
            }],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            // Particles layer over the lit scene; they never write depth so
            // billboard order stays irrelevant.
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        FULL_UV_RECT, PARTICLE_DRAW_LIMIT, PARTICLE_INSTANCE_STRIDE, PARTICLE_POOL_STRIDE,
        ParticleComputeParams, ParticleInstanceUniform, PoolItemUniform,
    };
    use scenix_particles::{
        ColorCurve, ColorOverLifetime, ParticleEmitter, ParticleSystem, SizeOverLifetime,
    };

    #[test]
    fn pack_premultiplies_tint_and_keeps_uv() {
        let item = scenix_particles::ParticleRenderItem {
            position: [1.0, 2.0, 3.0],
            size: 0.5,
            color: [1.0, 0.5, 0.25, 1.0],
            uv_rect: FULL_UV_RECT,
            rotation: 0.25,
        };
        let packed = ParticleInstanceUniform::pack(&item, [0.5, 1.0, 2.0, 0.5]);
        assert_eq!(packed.center_size, [1.0, 2.0, 3.0, 0.5]);
        assert_eq!(packed.color, [0.5, 0.5, 0.5, 0.5]);
        assert_eq!(packed.uv_rect, FULL_UV_RECT);
        assert_eq!(packed.rotation_pad[0], 0.25);
        assert_eq!(
            core::mem::size_of::<ParticleInstanceUniform>() as u64,
            PARTICLE_INSTANCE_STRIDE
        );
        assert_eq!(
            core::mem::size_of::<PoolItemUniform>() as u64,
            PARTICLE_POOL_STRIDE
        );
    }

    #[test]
    fn flipbook_cells_disable_billboard_rotation() {
        let item = scenix_particles::ParticleRenderItem {
            position: [0.0, 0.0, 0.0],
            size: 1.0,
            color: [1.0; 4],
            uv_rect: [0.0, 0.0, 0.5, 0.5],
            rotation: 0.6,
        };
        let packed = ParticleInstanceUniform::pack(&item, [1.0; 4]);
        assert_eq!(packed.rotation_pad[0], 0.0, "spin collapses on flipbook");
    }

    #[test]
    fn pool_seeding_round_trips_cpu_state() {
        let emitter = ParticleEmitter::new()
            .with_rate(240.0)
            .with_lifetime(2.0, 2.0)
            .with_modifier(ColorOverLifetime::new(ColorCurve::from_to(
                [1.0, 0.9, 0.7, 1.0],
                [0.9, 0.2, 0.0, 0.0],
            )))
            .with_modifier(SizeOverLifetime::from_to(1.0, 0.2));
        let mut system = ParticleSystem::new(emitter, 8, 11);
        system.update(1.0 / 16.0);
        assert!(!system.particles().is_empty());
        let first = system.particles()[0];
        let pool = PoolItemUniform::from_particle(&first);
        assert_eq!(pool.pos_age[..3], first.position.to_array()[..3]);
        assert_eq!(pool.pos_age[3], first.age);
        assert_eq!(pool.vel_life[3], first.lifetime);
        assert_eq!(pool.misc[0], first.size);
    }

    #[test]
    fn compute_params_bear_budget_and_dt() {
        let params = ParticleComputeParams::default()
            .with_dt(1.0 / 30.0)
            .with_spawn_budget(24);
        assert_eq!(params.gravity_dt[3], 1.0 / 30.0);
        assert_eq!(params.knobs[1], 24.0);
        assert_eq!(PARTICLE_DRAW_LIMIT.checked_mul(0), Some(0));
    }
}
