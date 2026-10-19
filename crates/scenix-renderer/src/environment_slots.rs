//! Environment slots (scenix v1.8.0 "Terrain, Sky, And Water"), behind the
//! `environment` feature.
//!
//! Environment materials (`scenix-material`'s terrain/sky/water
//! descriptors) are registered through [`crate::Renderer::register_environment`]
//! into per-`MaterialId` slots. Each slot carries the pre-packed 128-byte
//! uniform the WGSL shader consumes, the two bindable textures, and the
//! renderer-side geometry (user-packed terrain chunk geometry, or an
//! internally generated sky dome / water patch).
//!
//! Slot uniforms use their own bind-group layout tuple
//! `[frame @0, object @1, environment @2 (uniform-dynamic + two texture
//! views + sampler), light @3]`; group 2 is broader than the shared
//! 64-byte material uniform, so environment draws never mix with the shared
//! material belt.

use scenix_core::TextureId;
use scenix_math::{Aabb, Vec3};

use wgpu::util::DeviceExt;

use crate::gpu_scene::GpuMesh;

/// Byte size of one packed environment draw uniform.
pub(crate) const ENV_UNIFORM_SIZE: usize = 128;
/// Default environment draw belt capacity.
pub(crate) const ENV_DRAW_CAPACITY: usize = 64;

/// Belt-slot sizing placeholder: env uniforms pack into 128-byte slots.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct EnvUniformRaw {
    /// 128 bytes of payload.
    pub payload: [[f32; 4]; 8],
}

/// WMGSL pipeline family selector for environment draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum EnvironmentShaderKind {
    /// Aerial-gradient sky background (`sky_dome`).
    Sky,
    /// Splat-blended terrain surface (`terrain_splat`).
    Terrain,
    /// Fresnel/foam water surface (`water_surface`).
    Water,
}

/// One registered environment draw source.
///
/// Uniforms are packed once at registration so every frame emits the exact
/// same bytes; dynamic belt offsets select the per-draw slot.
#[derive(Debug)]
pub(crate) struct EnvironmentSlot {
    kind: EnvironmentShaderKind,
    uniform: [u8; ENV_UNIFORM_SIZE],
    textures: [Option<TextureId>; 2],
    mesh: GpuMesh,
    aabb: Aabb,
    transparent: bool,
}

impl EnvironmentSlot {
    /// Whether the water draw uses alpha blending (diagnostic only).
    #[allow(dead_code)]
    pub(crate) const fn transparent_kind(&self) -> bool {
        self.transparent
    }
}

impl EnvironmentSlot {
    /// The WGSL pipeline family.
    #[inline]
    pub(crate) const fn kind(&self) -> EnvironmentShaderKind {
        self.kind
    }

    /// Packed uniform bytes.
    #[inline]
    pub(crate) fn uniform(&self) -> &[u8] {
        &self.uniform
    }

    /// Bindable textures `(primary, secondary)`; the layout picks the
    /// meaning: splat/detail for terrain, reflection/refraction for water,
    /// sky uses neither.
    #[inline]
    pub(crate) const fn textures(&self) -> [Option<TextureId>; 2] {
        self.textures
    }

    /// The packed renderer mesh for the draw.
    #[inline]
    pub(crate) fn mesh(&self) -> &GpuMesh {
        &self.mesh
    }

    /// World-neutral bounding box, transformed per node by culling.
    #[inline]
    pub(crate) const fn aabb(&self) -> Aabb {
        self.aabb
    }
}

/// The environment payload passed to
/// [`crate::Renderer::register_environment`].
#[derive(Debug)]
pub enum EnvironmentMaterial<'a> {
    /// A splat terrain: chunk + material state and its packed chunk
    /// geometry (built by `scenix-terrain`).
    Terrain(
        &'a scenix_material::TerrainMaterial,
        &'a scenix_mesh::Geometry,
    ),
    /// A sky background dome.
    Sky(&'a scenix_material::SkyMaterial),
    /// A water surface: material state plus the CPU `scenix-water`
    /// wave field whose parameters the GPU vertex stage mirrors.
    Water(
        &'a scenix_material::WaterMaterial,
        &'a scenix_water::WaveField,
    ),
}

/// Raw terrain uniform: packed layer colors + `(roughness, tiling,
/// has_detail, has_splat)` per layer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct TerrainUniformRaw {
    layer_colors: [[f32; 4]; 4],
    layer_params: [[f32; 4]; 4],
}

/// Raw sky uniform: gradient bookends + sun + fade band, padded to 128
/// bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyUniformRaw {
    top_color: [f32; 4],
    horizon_color: [f32; 4],
    ground_color: [f32; 4],
    sun_params: [f32; 4],
    sun_color: [f32; 4],
    fade_params: [f32; 4],
    tail: [[f32; 4]; 2],
}

/// Raw water uniform: tints + Gerstner wave rows + foam/fresnel knobs.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct WaterUniformRaw {
    deep_color: [f32; 4],
    shallow_color: [f32; 4],
    waves: [[f32; 4]; 4],
    params0: [f32; 4],
    params1: [f32; 4],
}

/// Packs the terrain splat uniform.
fn pack_terrain_uniform(material: &scenix_material::TerrainMaterial) -> [u8; ENV_UNIFORM_SIZE] {
    let mut raw = TerrainUniformRaw::default();
    for (index, layer) in material.layers.iter().enumerate() {
        raw.layer_colors[index] = layer.albedo.to_array();
        raw.layer_params[index] = [layer.roughness, layer.tiling, 0.0, 0.0];
    }
    raw.layer_params[0][2] = u8::from(material.detail_texture.is_some()) as f32;
    raw.layer_params[0][3] = u8::from(material.splat_texture.is_some()) as f32;
    pack_raw(&raw)
}

/// Packs the sky uniform.
fn pack_sky_uniform(material: &scenix_material::SkyMaterial) -> [u8; ENV_UNIFORM_SIZE] {
    let direction = material.sun_direction.normalize();
    let raw = SkyUniformRaw {
        top_color: material.top_color.to_array(),
        horizon_color: material.horizon_color.to_array(),
        ground_color: material.ground_color.to_array(),
        sun_params: [
            direction.x,
            direction.y,
            direction.z,
            material.glare_strength,
        ],
        sun_color: material.sun_color.to_array(),
        fade_params: [
            material.fade_start,
            material.fade_range,
            u8::from(material.atmosphere) as f32,
            0.0,
        ],
        tail: [[0.0; 4]; 2],
    };
    pack_raw(&raw)
}

/// Packs the water uniform from material state and the CPU wave field.
fn pack_water_uniform(
    material: &scenix_material::WaterMaterial,
    waves: &scenix_water::WaveField,
) -> [u8; ENV_UNIFORM_SIZE] {
    let mut raw = WaterUniformRaw {
        deep_color: material.deep_color.to_array(),
        shallow_color: material.shallow_color.to_array(),
        waves: [[0.0; 4]; 4],
        params0: [0.0; 4],
        params1: [0.0; 4],
    };
    for (index, wave) in waves.waves().iter().take(4).enumerate() {
        let direction = wave.direction.normalize();
        raw.waves[index] = [direction.x, direction.y, wave.steepness, wave.wavelength];
    }
    let wave_count = waves
        .waves()
        .iter()
        .take(4)
        .filter(|wave| wave.wavelength > 1.0e-4)
        .count() as f32;
    let (foam_threshold, foam_sharpness) = match material.foam_threshold {
        Some(threshold) => (threshold.clamp(0.0, 1.0), material.foam_sharpness.max(0.0)),
        None => (1.5, 0.0),
    };
    let reflection_strength = if material.reflection_texture.is_some() {
        material.reflection_strength
    } else {
        0.0
    };
    let refraction_strength = if material.refraction_texture.is_some() {
        material.refraction_strength
    } else {
        0.0
    };
    raw.params0 = [
        foam_threshold,
        foam_sharpness,
        material.base_reflectance,
        wave_count,
    ];
    raw.params1 = [reflection_strength, refraction_strength, 0.0, 0.0];
    pack_raw(&raw)
}

#[inline]
fn pack_raw<T>(raw: &T) -> [u8; ENV_UNIFORM_SIZE]
where
    T: bytemuck::Pod,
{
    let bytes: &[u8] = bytemuck::bytes_of(raw);
    assert!(
        bytes.len() <= ENV_UNIFORM_SIZE,
        "environment uniform overflowed the belt slot"
    );
    let mut slot = [0_u8; ENV_UNIFORM_SIZE];
    slot[..bytes.len()].copy_from_slice(bytes);
    slot
}

/// Packs and uploads the slot mesh for the given geometry.
fn build_slot(
    device: &wgpu::Device,
    kind: EnvironmentShaderKind,
    uniform: [u8; ENV_UNIFORM_SIZE],
    textures: [Option<TextureId>; 2],
    geometry: scenix_mesh::Geometry,
    transparent: bool,
) -> Result<EnvironmentSlot, scenix_core::ValidationError> {
    let packed = crate::gpu_scene::GpuScene::pack_geometry(&geometry)?;
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("scenix.environment.vertices"),
        contents: bytemuck::cast_slice(packed.vertices.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("scenix.environment.indices"),
        contents: packed.index_bytes.as_slice(),
        usage: wgpu::BufferUsages::INDEX,
    });
    let aabb = packed.aabb;
    let mesh = GpuMesh::from_buffers(vertex_buffer, index_buffer, packed);
    Ok(EnvironmentSlot {
        kind,
        uniform,
        textures,
        mesh,
        aabb,
        transparent,
    })
}

/// Builds a terrain slot from a splat material and its chunk geometry.
///
/// # Errors
/// Returns the geometry validation error when packing fails.
pub(crate) fn build_terrain_slot(
    device: &wgpu::Device,
    material: &scenix_material::TerrainMaterial,
    geometry: &scenix_mesh::Geometry,
) -> Result<EnvironmentSlot, scenix_core::ValidationError> {
    let textures = [material.splat_texture, material.detail_texture];
    build_slot(
        device,
        EnvironmentShaderKind::Terrain,
        pack_terrain_uniform(material),
        textures,
        geometry.clone(),
        false,
    )
}

/// Builds a sky slot around the internally generated dome geometry.
///
/// # Errors
/// Returns the geometry validation error when packing fails.
pub(crate) fn build_sky_slot(
    device: &wgpu::Device,
    material: &scenix_material::SkyMaterial,
) -> Result<EnvironmentSlot, scenix_core::ValidationError> {
    build_slot(
        device,
        EnvironmentShaderKind::Sky,
        pack_sky_uniform(material),
        [None, None],
        sky_dome_geometry(32, 16),
        false,
    )
}

/// Builds a water slot around the internally generated flat patch; the GPU
/// vertex stage displaces it with the CPU wave-field parameters.
///
/// # Errors
/// Returns the geometry validation error when packing fails.
pub(crate) fn build_water_slot(
    device: &wgpu::Device,
    material: &scenix_material::WaterMaterial,
    waves: &scenix_water::WaveField,
) -> Result<EnvironmentSlot, scenix_core::ValidationError> {
    let textures = [material.reflection_texture, material.refraction_texture];
    build_slot(
        device,
        EnvironmentShaderKind::Water,
        pack_water_uniform(material, waves),
        textures,
        water_patch_geometry(32),
        true,
    )
}

/// Generates a unit-radius equirectangular dome (row 0 at the zenith pole,
/// longitude from `+x` toward `+z`); scale it through the node transform.
pub(crate) fn sky_dome_geometry(columns: u32, rows: u32) -> scenix_mesh::Geometry {
    use scenix_core::Color;
    use scenix_math::float::{cos, sin};
    use scenix_mesh::Geometry;

    let columns = columns.max(4);
    let rows = rows.max(3);
    let mut positions = Vec::with_capacity(((columns + 1) * (rows + 1)) as usize);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    for row in 0..=rows {
        let v = row as f32 / rows as f32;
        let latitude = core::f32::consts::FRAC_PI_2 - v * core::f32::consts::PI;
        let horizontal = cos(latitude).max(0.0);
        for column in 0..=columns {
            let u = column as f32 / columns as f32;
            let longitude = u * core::f32::consts::TAU;
            let direction = Vec3::new(
                horizontal * cos(longitude),
                sin(latitude),
                horizontal * sin(longitude),
            );
            positions.push(direction);
            normals.push(direction);
            uvs.push(scenix_math::Vec2::new(u, v));
        }
    }
    let mut indices = Vec::with_capacity((columns * rows * 6) as usize);
    let stride = (columns + 1) as i32;
    for row in 0..rows as i32 {
        for column in 0..columns as i32 {
            let top_left = row * stride + column;
            let top_right = top_left + 1;
            let bottom_left = top_left + stride;
            let bottom_right = bottom_left + 1;
            indices.extend([
                top_left as u32,
                bottom_left as u32,
                top_right as u32,
                top_right as u32,
                bottom_left as u32,
                bottom_right as u32,
            ]);
        }
    }
    let count = positions.len();
    Geometry {
        positions,
        normals,
        uvs,
        uvs2: Vec::new(),
        colors: vec![Color::WHITE; count],
        indices,
        tangents: Vec::new(),
    }
}

/// Generates the flat `[−0.5, 0.5]²` water patch the vertex shader
/// displaces; the node transform provides position, scale (world extents),
/// and wave amplitude scaling.
pub(crate) fn water_patch_geometry(segments: u32) -> scenix_mesh::Geometry {
    use scenix_core::Color;
    use scenix_mesh::Geometry;
    let segments = segments.max(2);
    let mut positions = Vec::with_capacity((segments * segments) as usize);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    for row in 0..segments {
        for column in 0..segments {
            let u = column as f32 / (segments - 1) as f32;
            let v = row as f32 / (segments - 1) as f32;
            positions.push(Vec3::new(u - 0.5, 0.0, v - 0.5));
            normals.push(Vec3::Y);
            uvs.push(scenix_math::Vec2::new(u, v));
        }
    }
    let mut indices = Vec::with_capacity(((segments - 1) * (segments - 1) * 6) as usize);
    let stride = segments;
    for row in 0..segments - 1 {
        for column in 0..segments - 1 {
            let top_left = row * stride + column;
            let top_right = top_left + 1;
            let bottom_left = top_left + stride;
            let bottom_right = bottom_left + 1;
            indices.extend([
                top_left,
                bottom_left,
                top_right,
                top_right,
                bottom_left,
                bottom_right,
            ]);
        }
    }
    let count = positions.len();
    Geometry {
        positions,
        normals,
        uvs,
        uvs2: Vec::new(),
        colors: vec![Color::WHITE; count],
        indices,
        tangents: Vec::new(),
    }
}

/// The environment bind-group layout: `[uniform-dynamic, texture, texture,
/// sampler]` at group 2 of each environment draw.
pub(crate) fn create_env_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("scenix.environment.layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
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
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Builds a bind group for one environment slot: the belt buffer plus two
/// texture views (missing views bind the neutral texture) and the shared
/// environment sampler.
pub(crate) fn create_env_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    sampler: &wgpu::Sampler,
    view_a: &wgpu::TextureView,
    view_b: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("scenix.environment.bind_group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(ENV_UNIFORM_SIZE as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(view_a),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(view_b),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

/// Builds the wrap-group render pipeline for an environment kind.
///
/// Terrain and water use `LessEqual` depth; the sky dome uses `Always` with
/// depth writes disabled so meshes land in front of it.
pub(crate) fn create_env_pipeline(
    device: &wgpu::Device,
    uniforms: &crate::renderer::UniformResources,
    kind: EnvironmentShaderKind,
    color_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let (source, label, depth_write, depth_compare) = match kind {
        EnvironmentShaderKind::Sky => (
            include_str!("shaders/sky_dome.wgsl"),
            "scenix.environment.sky",
            false,
            wgpu::CompareFunction::Always,
        ),
        EnvironmentShaderKind::Terrain => (
            include_str!("shaders/terrain_splat.wgsl"),
            "scenix.environment.terrain",
            true,
            wgpu::CompareFunction::LessEqual,
        ),
        EnvironmentShaderKind::Water => (
            include_str!("shaders/water_surface.wgsl"),
            "scenix.environment.water",
            true,
            wgpu::CompareFunction::LessEqual,
        ),
    };
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[
            Some(&uniforms.frame_layout),
            Some(&uniforms.object_layout),
            Some(uniforms.environment_layout()),
            Some(&uniforms.light_layout),
        ],
        immediate_size: 0,
    });
    let blend = match kind {
        EnvironmentShaderKind::Water => wgpu::BlendState::ALPHA_BLENDING,
        _ => wgpu::BlendState::REPLACE,
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[crate::gpu_scene::PackedVertex::layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(depth_write),
            depth_compare: Some(depth_compare),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
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
        EnvironmentShaderKind, pack_raw, pack_sky_uniform, pack_terrain_uniform, pack_water_uniform,
    };
    use scenix_core::{Color, TextureId};
    use scenix_material::{SkyMaterial, SplatLayerDesc, TerrainMaterial, WaterMaterial};
    use scenix_math::Vec3;
    use scenix_water::WaveField;

    /// Reads one little-endian f32 out of a packed uniform.
    fn f32_at(bytes: &[u8], byte_offset: usize) -> f32 {
        f32::from_le_bytes([
            bytes[byte_offset],
            bytes[byte_offset + 1],
            bytes[byte_offset + 2],
            bytes[byte_offset + 3],
        ])
    }

    /// Asserts a packed RGBA color matches bit-for-bit.
    fn assert_color(bytes: &[u8], byte_offset: usize, color: Color) {
        let [r, g, b, a] = color.to_array();
        assert_eq!(f32_at(bytes, byte_offset), r);
        assert_eq!(f32_at(bytes, byte_offset + 4), g);
        assert_eq!(f32_at(bytes, byte_offset + 8), b);
        assert_eq!(f32_at(bytes, byte_offset + 12), a);
    }

    // TerrainUniformRaw: layer_colors 0..64, layer_params 64..128
    // (layer 0 = roughness@64, tiling@68, has_detail@72, has_splat@76).
    #[test]
    fn terrain_uniform_packs_layers_and_flags() {
        let material = TerrainMaterial::default_palette();
        let terrain = pack_terrain_uniform(&material);
        assert_eq!(terrain.len(), super::ENV_UNIFORM_SIZE);
        assert_color(&terrain, 0, material.layers[0].albedo);
        assert_eq!(f32_at(&terrain, 64), material.layers[0].roughness);
        assert_eq!(f32_at(&terrain, 68), material.layers[0].tiling);
        assert_eq!(f32_at(&terrain, 72), 0.0, "no detail texture");
        assert_eq!(f32_at(&terrain, 76), 0.0, "no splat texture");

        let textured = material
            .with_splat_texture(TextureId::new(4))
            .with_detail_texture(TextureId::new(9));
        let packed = pack_terrain_uniform(&textured);
        assert_eq!(f32_at(&packed, 72), 1.0, "detail flag set");
        assert_eq!(f32_at(&packed, 76), 1.0, "splat flag set");
    }

    // SkyUniformRaw: top@0, horizon@16, ground@32, sun_params@48 (glare@60),
    // sun_color@64, fade_params@80 (atmosphere@88), tail 96..128.
    #[test]
    fn sky_uniform_packs_gradient_and_sun() {
        let material = SkyMaterial::clear_day();
        let sky = pack_sky_uniform(&material);
        assert_eq!(sky.len(), super::ENV_UNIFORM_SIZE);
        assert_color(&sky, 0, material.top_color);
        assert_color(&sky, 16, material.horizon_color);
        assert_color(&sky, 64, material.sun_color);
        assert_eq!(f32_at(&sky, 60), material.glare_strength);
        assert_eq!(
            f32_at(&sky, 88),
            f32::from(material.atmosphere),
            "atmosphere flag rides fade_params.z"
        );
        assert_eq!(
            f32_at(&sky, 80),
            material.fade_start,
            "fade band start packs verbatim"
        );
    }

    // WaterUniformRaw: deep@0, shallow@16, waves 32..96 (row i: dirx@0,
    // dirz@4, steepness@8, wavelength@12), params0@96 (threshold@96,
    // sharpness@100, base_reflectance@104, wave_count@108), params1@112.
    #[test]
    fn water_uniform_packs_waves_and_foam() {
        let material = WaterMaterial::clear();
        let waves = WaveField::ocean_shore();
        let water = pack_water_uniform(&material, &waves);
        assert_eq!(water.len(), super::ENV_UNIFORM_SIZE);
        assert_color(&water, 0, material.deep_color);
        assert_eq!(
            f32_at(&water, 108),
            3.0,
            "ocean-shore preset packs three waves"
        );
        assert_eq!(
            f32_at(&water, 32 + 12),
            waves.waves()[0].wavelength,
            "first wave wavelength round-trips"
        );
        assert_eq!(
            f32_at(&water, 96),
            1.5,
            "foam disabled parks the threshold above one"
        );

        let foamy = pack_water_uniform(
            &WaterMaterial::clear().with_foam(0.45, 6.0),
            &WaveField::calm(),
        );
        assert_eq!(f32_at(&foamy, 96), 0.45);
        assert_eq!(f32_at(&foamy, 100), 6.0);
    }

    #[test]
    fn pack_raw_tolerates_smaller_payloads() {
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        #[repr(C)]
        struct Tiny {
            value: f32,
        }
        let packed = pack_raw(&Tiny { value: 7.0 });
        assert_eq!(packed.len(), 128);
        let mut kinds = [false; 3];
        for kind in [
            EnvironmentShaderKind::Sky,
            EnvironmentShaderKind::Terrain,
            EnvironmentShaderKind::Water,
        ] {
            kinds[match kind {
                EnvironmentShaderKind::Sky => 0,
                EnvironmentShaderKind::Terrain => 1,
                EnvironmentShaderKind::Water => 2,
            }] = true;
        }
        assert!(kinds.iter().all(|present| *present));
        // Splat layer defaults keep distinct palette entries.
        let palette = [
            SplatLayerDesc::default_grass(),
            SplatLayerDesc::default_rock(),
        ];
        assert_ne!(palette[0].albedo, palette[1].albedo);
        let _ = (Color::WHITE, Vec3::ZERO);
    }
}
