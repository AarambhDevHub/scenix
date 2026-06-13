use std::collections::HashMap;

use scenix_core::{Color, LightId, MaterialId, MeshId, TextureId, ValidationError};
use scenix_light::{
    AmbientLight, AreaLight, DirectionalLight, HemisphereLight, LightProbe, PointLight, SpotLight,
};
use scenix_material::{
    LambertMaterial, Material, NormalMaterial, PbrMaterial, PhysicalMaterial, PipelineKey,
    ToonMaterial, UnlitMaterial, WireframeMaterial,
};
use scenix_math::{Aabb, Mat4, Vec2, Vec3, Vec4};
use scenix_mesh::Geometry;
use scenix_texture::{
    AddressMode, CompareFunction, FilterMode, Sampler, Texture2D, Texture3D, TextureCube,
    TextureFormat,
};
use wgpu::util::DeviceExt;

/// Interleaved GPU vertex layout used by the v0.6 renderer.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedVertex {
    /// Vertex position.
    pub position: [f32; 3],
    /// Vertex normal.
    pub normal: [f32; 3],
    /// Primary texture coordinate.
    pub uv: [f32; 2],
    /// Vertex color.
    pub color: [f32; 4],
    /// Tangent vector and handedness.
    pub tangent: [f32; 4],
}

impl PackedVertex {
    /// Returns the wgpu vertex-buffer layout.
    pub const fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
            3 => Float32x4,
            4 => Float32x4
        ];
        wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<PackedVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

/// Index type chosen for packed geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GpuIndexFormat {
    /// 16-bit index buffer.
    Uint16,
    /// 32-bit index buffer.
    Uint32,
}

impl GpuIndexFormat {
    /// Returns the matching wgpu index format.
    #[inline]
    pub const fn to_wgpu(self) -> wgpu::IndexFormat {
        match self {
            Self::Uint16 => wgpu::IndexFormat::Uint16,
            Self::Uint32 => wgpu::IndexFormat::Uint32,
        }
    }
}

/// CPU-packed geometry ready for GPU upload.
#[derive(Clone, Debug, PartialEq)]
pub struct PackedGeometry {
    /// Interleaved vertices.
    pub vertices: Vec<PackedVertex>,
    /// Raw index bytes in `index_format`.
    pub index_bytes: Vec<u8>,
    /// Number of indices.
    pub index_count: u32,
    /// Index storage format.
    pub index_format: GpuIndexFormat,
    /// Local-space geometry bounds.
    pub aabb: Aabb,
}

/// Uploaded GPU mesh buffers.
#[derive(Debug)]
pub struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    packed: PackedGeometry,
}

impl GpuMesh {
    /// Returns the vertex buffer.
    #[inline]
    pub const fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// Returns the index buffer.
    #[inline]
    pub const fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    /// Returns packed geometry metadata.
    #[inline]
    pub const fn packed(&self) -> &PackedGeometry {
        &self.packed
    }
}

/// Renderer-side material registry entry.
#[derive(Clone, Debug, PartialEq)]
pub enum RendererMaterial {
    /// Metallic-roughness material.
    Pbr(PbrMaterial),
    /// Advanced physical material.
    Physical(PhysicalMaterial),
    /// Constant-color unlit material.
    Unlit(UnlitMaterial),
    /// Diffuse Lambert material.
    Lambert(LambertMaterial),
    /// Cel-shaded material.
    Toon(ToonMaterial),
    /// Wireframe/debug preview material.
    Wireframe(WireframeMaterial),
    /// Normal visualization material.
    Normal(NormalMaterial),
}

impl RendererMaterial {
    /// Returns the material pipeline key.
    #[inline]
    pub fn pipeline_key(&self) -> PipelineKey {
        match self {
            Self::Pbr(material) => material.pipeline_key(),
            Self::Physical(material) => material.pipeline_key(),
            Self::Unlit(material) => material.pipeline_key(),
            Self::Lambert(material) => material.pipeline_key(),
            Self::Toon(material) => material.pipeline_key(),
            Self::Wireframe(material) => material.pipeline_key(),
            Self::Normal(material) => material.pipeline_key(),
        }
    }

    /// Returns whether the material should be depth-sorted with transparent draws.
    #[inline]
    pub fn is_transparent(&self) -> bool {
        match self {
            Self::Pbr(material) => material.is_transparent(),
            Self::Physical(material) => material.is_transparent(),
            Self::Unlit(material) => material.is_transparent(),
            Self::Lambert(material) => material.is_transparent(),
            Self::Toon(material) => material.is_transparent(),
            Self::Wireframe(material) => material.is_transparent(),
            Self::Normal(material) => material.is_transparent(),
        }
    }

    /// Returns the base preview color used by the stable v1 renderer path.
    #[inline]
    pub fn preview_color(&self) -> Color {
        match self {
            Self::Pbr(material) => material.albedo,
            Self::Physical(material) => material.base.albedo,
            Self::Unlit(material) => material.color,
            Self::Lambert(material) => material.color,
            Self::Toon(material) => material.color,
            Self::Wireframe(material) => Color {
                a: material.opacity,
                ..material.color
            },
            Self::Normal(_) => Color::WHITE,
        }
    }

    /// Returns a compact shader-family code for the shared v1 preview shader.
    #[inline]
    pub fn preview_shader_code(&self) -> f32 {
        match self {
            Self::Pbr(_) => 0.0,
            Self::Physical(_) => 1.0,
            Self::Unlit(_) => 2.0,
            Self::Lambert(_) => 3.0,
            Self::Toon(_) => 4.0,
            Self::Wireframe(_) => 5.0,
            Self::Normal(_) => 6.0,
        }
    }
}

/// Renderer-side texture metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuTexture {
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
    /// CPU texture format.
    pub format: TextureFormat,
    /// Matching wgpu texture format, when supported.
    pub wgpu_format: Option<wgpu::TextureFormat>,
    /// Sampler metadata.
    pub sampler: Sampler,
    /// Number of mip levels stored in the CPU texture.
    pub mip_levels: u32,
}

/// Texture metadata store used by `GpuMaterial`.
pub type TextureStore = HashMap<TextureId, GpuTexture>;

/// Renderer-side light registry entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RendererLight {
    /// Ambient light.
    Ambient(AmbientLight),
    /// Directional light.
    Directional(DirectionalLight),
    /// Point light.
    Point(PointLight),
    /// Spot light.
    Spot(SpotLight),
    /// Hemisphere gradient light.
    Hemisphere(HemisphereLight),
    /// Rectangular area light.
    Area(AreaLight),
    /// Spherical-harmonics light probe.
    Probe(LightProbe),
}

/// Visible draw submission generated from a scene node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawSubmission {
    /// Mesh resource ID.
    pub mesh_id: MeshId,
    /// Material resource ID.
    pub material_id: MaterialId,
    /// Node world transform.
    pub world_matrix: Mat4,
    /// World-space bounds used for culling.
    pub world_aabb: Aabb,
    /// Distance from camera position to bounds center.
    pub distance_to_camera: f32,
    /// Whether this draw needs transparent sorting.
    pub transparent: bool,
    /// Stable render order.
    pub render_order: u32,
}

/// Renderer-owned GPU scene resources and CPU metadata.
#[derive(Debug, Default)]
pub struct GpuScene {
    meshes: HashMap<MeshId, GpuMesh>,
    materials: HashMap<MaterialId, RendererMaterial>,
    textures: TextureStore,
    lights: HashMap<LightId, RendererLight>,
}

impl GpuScene {
    /// Creates an empty GPU scene registry.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Packs geometry into the renderer interleaved vertex/index layout.
    pub fn pack_geometry(geometry: &Geometry) -> Result<PackedGeometry, ValidationError> {
        geometry.validate()?;
        let vertex_count = geometry.positions.len();
        let mut vertices = Vec::with_capacity(vertex_count);
        for index in 0..vertex_count {
            let position = geometry.positions[index];
            let normal = geometry.normals.get(index).copied().unwrap_or(Vec3::Y);
            let uv = geometry.uvs.get(index).copied().unwrap_or(Vec2::ZERO);
            let color = geometry.colors.get(index).copied().unwrap_or(Color::WHITE);
            let tangent = geometry
                .tangents
                .get(index)
                .copied()
                .unwrap_or(Vec4::new(1.0, 0.0, 0.0, 1.0));
            vertices.push(PackedVertex {
                position: [position.x, position.y, position.z],
                normal: [normal.x, normal.y, normal.z],
                uv: [uv.x, uv.y],
                color: color.to_array(),
                tangent: [tangent.x, tangent.y, tangent.z, tangent.w],
            });
        }

        let source_indices: Vec<u32> = if geometry.indices.is_empty() {
            (0..vertex_count as u32).collect()
        } else {
            geometry.indices.clone()
        };
        let can_use_u16 = vertex_count <= u16::MAX as usize
            && source_indices.iter().all(|index| *index <= u16::MAX as u32);
        let (index_bytes, index_format) = if can_use_u16 {
            let indices: Vec<u16> = source_indices.iter().map(|index| *index as u16).collect();
            (
                bytemuck::cast_slice(&indices).to_vec(),
                GpuIndexFormat::Uint16,
            )
        } else {
            (
                bytemuck::cast_slice(source_indices.as_slice()).to_vec(),
                GpuIndexFormat::Uint32,
            )
        };

        Ok(PackedGeometry {
            vertices,
            index_bytes,
            index_count: source_indices.len() as u32,
            index_format,
            aabb: geometry.aabb(),
        })
    }

    /// Uploads and stores a mesh.
    pub fn register_mesh(
        &mut self,
        device: &wgpu::Device,
        mesh_id: MeshId,
        geometry: &Geometry,
    ) -> Result<(), ValidationError> {
        if mesh_id.is_null() {
            return Err(ValidationError::InvalidId);
        }
        let packed = Self::pack_geometry(geometry)?;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("scenix.mesh.vertices"),
            contents: bytemuck::cast_slice(packed.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("scenix.mesh.indices"),
            contents: packed.index_bytes.as_slice(),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.meshes.insert(
            mesh_id,
            GpuMesh {
                vertex_buffer,
                index_buffer,
                packed,
            },
        );
        Ok(())
    }

    /// Registers a PBR material.
    pub fn register_pbr_material(
        &mut self,
        material_id: MaterialId,
        material: &PbrMaterial,
    ) -> Result<(), ValidationError> {
        self.register_material(material_id, RendererMaterial::Pbr(material.clone()))
    }

    /// Registers a physical material.
    pub fn register_physical_material(
        &mut self,
        material_id: MaterialId,
        material: &PhysicalMaterial,
    ) -> Result<(), ValidationError> {
        self.register_material(material_id, RendererMaterial::Physical(material.clone()))
    }

    /// Registers an unlit material.
    pub fn register_unlit_material(
        &mut self,
        material_id: MaterialId,
        material: &UnlitMaterial,
    ) -> Result<(), ValidationError> {
        self.register_material(material_id, RendererMaterial::Unlit(material.clone()))
    }

    /// Registers a Lambert material.
    pub fn register_lambert_material(
        &mut self,
        material_id: MaterialId,
        material: &LambertMaterial,
    ) -> Result<(), ValidationError> {
        self.register_material(material_id, RendererMaterial::Lambert(material.clone()))
    }

    /// Registers a toon material.
    pub fn register_toon_material(
        &mut self,
        material_id: MaterialId,
        material: &ToonMaterial,
    ) -> Result<(), ValidationError> {
        self.register_material(material_id, RendererMaterial::Toon(material.clone()))
    }

    /// Registers a wireframe preview material.
    pub fn register_wireframe_material(
        &mut self,
        material_id: MaterialId,
        material: &WireframeMaterial,
    ) -> Result<(), ValidationError> {
        self.register_material(material_id, RendererMaterial::Wireframe(*material))
    }

    /// Registers a normal visualization material.
    pub fn register_normal_material(
        &mut self,
        material_id: MaterialId,
        material: &NormalMaterial,
    ) -> Result<(), ValidationError> {
        self.register_material(material_id, RendererMaterial::Normal(*material))
    }

    /// Registers a renderer material.
    pub fn register_material(
        &mut self,
        material_id: MaterialId,
        material: RendererMaterial,
    ) -> Result<(), ValidationError> {
        if material_id.is_null() {
            return Err(ValidationError::InvalidId);
        }
        self.materials.insert(material_id, material);
        Ok(())
    }

    /// Registers validated 2D texture metadata and sampler state.
    pub fn register_texture2d(
        &mut self,
        texture_id: TextureId,
        texture: &Texture2D,
        sampler: Sampler,
    ) -> Result<(), ValidationError> {
        if texture_id.is_null() {
            return Err(ValidationError::InvalidId);
        }
        texture.validate()?;
        self.textures.insert(
            texture_id,
            GpuTexture {
                width: texture.width,
                height: texture.height,
                format: texture.format,
                wgpu_format: to_wgpu_texture_format(texture.format),
                sampler,
                mip_levels: texture.mip_levels.max(1),
            },
        );
        Ok(())
    }

    /// Registers validated cube texture metadata and sampler state.
    pub fn register_texture_cube(
        &mut self,
        texture_id: TextureId,
        texture: &TextureCube,
        sampler: Sampler,
    ) -> Result<(), ValidationError> {
        if texture_id.is_null() {
            return Err(ValidationError::InvalidId);
        }
        texture.validate()?;
        self.textures.insert(
            texture_id,
            GpuTexture {
                width: texture.size,
                height: texture.size,
                format: texture.format,
                wgpu_format: to_wgpu_texture_format(texture.format),
                sampler,
                mip_levels: texture.mip_levels.max(1),
            },
        );
        Ok(())
    }

    /// Registers validated 3D texture metadata and sampler state.
    pub fn register_texture3d(
        &mut self,
        texture_id: TextureId,
        texture: &Texture3D,
        sampler: Sampler,
    ) -> Result<(), ValidationError> {
        if texture_id.is_null() {
            return Err(ValidationError::InvalidId);
        }
        texture.validate()?;
        self.textures.insert(
            texture_id,
            GpuTexture {
                width: texture.width,
                height: texture.height,
                format: texture.format,
                wgpu_format: to_wgpu_texture_format(texture.format),
                sampler,
                mip_levels: texture.mip_levels.max(1),
            },
        );
        Ok(())
    }

    /// Registers a light.
    pub fn register_light(
        &mut self,
        light_id: LightId,
        light: RendererLight,
    ) -> Result<(), ValidationError> {
        if light_id.is_null() {
            return Err(ValidationError::InvalidId);
        }
        self.lights.insert(light_id, light);
        Ok(())
    }

    /// Removes one mesh from the registry.
    #[inline]
    pub fn unregister_mesh(&mut self, mesh_id: MeshId) -> bool {
        self.meshes.remove(&mesh_id).is_some()
    }

    /// Removes one material from the registry.
    #[inline]
    pub fn unregister_material(&mut self, material_id: MaterialId) -> bool {
        self.materials.remove(&material_id).is_some()
    }

    /// Removes one texture metadata entry from the registry.
    #[inline]
    pub fn unregister_texture(&mut self, texture_id: TextureId) -> bool {
        self.textures.remove(&texture_id).is_some()
    }

    /// Removes one light from the registry.
    #[inline]
    pub fn unregister_light(&mut self, light_id: LightId) -> bool {
        self.lights.remove(&light_id).is_some()
    }

    /// Clears all mesh resources.
    #[inline]
    pub fn clear_meshes(&mut self) {
        self.meshes.clear();
    }

    /// Clears all material resources.
    #[inline]
    pub fn clear_materials(&mut self) {
        self.materials.clear();
    }

    /// Clears all texture metadata.
    #[inline]
    pub fn clear_textures(&mut self) {
        self.textures.clear();
    }

    /// Clears all light resources.
    #[inline]
    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }

    /// Returns a mesh by ID.
    #[inline]
    pub fn mesh(&self, mesh_id: MeshId) -> Option<&GpuMesh> {
        self.meshes.get(&mesh_id)
    }

    /// Returns a material by ID.
    #[inline]
    pub fn material(&self, material_id: MaterialId) -> Option<&RendererMaterial> {
        self.materials.get(&material_id)
    }

    /// Returns texture metadata by ID.
    #[inline]
    pub fn texture(&self, texture_id: TextureId) -> Option<&GpuTexture> {
        self.textures.get(&texture_id)
    }

    /// Returns registered texture metadata.
    #[inline]
    pub const fn textures(&self) -> &TextureStore {
        &self.textures
    }

    /// Returns registered lights.
    #[inline]
    pub fn lights(&self) -> impl Iterator<Item = (&LightId, &RendererLight)> {
        self.lights.iter()
    }

    /// Returns the number of registered lights.
    #[inline]
    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    /// Returns the number of registered meshes.
    #[inline]
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// Returns the number of registered materials.
    #[inline]
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// Returns approximate GPU bytes used by mesh vertex and index buffers.
    #[inline]
    pub fn geometry_memory_bytes(&self) -> u64 {
        self.meshes
            .values()
            .map(|mesh| {
                bytemuck::cast_slice::<PackedVertex, u8>(mesh.packed.vertices.as_slice()).len()
                    + mesh.packed.index_bytes.len()
            })
            .sum::<usize>() as u64
    }
}

/// Converts scenix texture format metadata to a wgpu format.
pub const fn to_wgpu_texture_format(format: TextureFormat) -> Option<wgpu::TextureFormat> {
    match format {
        TextureFormat::Rgba8Unorm => Some(wgpu::TextureFormat::Rgba8Unorm),
        TextureFormat::Rgba8UnormSrgb => Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        TextureFormat::Rgba16Float => Some(wgpu::TextureFormat::Rgba16Float),
        TextureFormat::Depth32Float => Some(wgpu::TextureFormat::Depth32Float),
        TextureFormat::Bc7RgbaUnorm => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        TextureFormat::Astc4x4RgbaUnorm => Some(wgpu::TextureFormat::Astc {
            block: wgpu::AstcBlock::B4x4,
            channel: wgpu::AstcChannel::Unorm,
        }),
        TextureFormat::Etc2Rgba8Unorm => Some(wgpu::TextureFormat::Etc2Rgba8Unorm),
    }
}

/// Converts sampler filter modes to wgpu filter modes.
pub const fn to_wgpu_filter_mode(filter: FilterMode) -> wgpu::FilterMode {
    match filter {
        FilterMode::Nearest => wgpu::FilterMode::Nearest,
        FilterMode::Linear => wgpu::FilterMode::Linear,
    }
}

/// Converts sampler address modes to wgpu address modes.
pub const fn to_wgpu_address_mode(address: AddressMode) -> wgpu::AddressMode {
    match address {
        AddressMode::Repeat => wgpu::AddressMode::Repeat,
        AddressMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        AddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
    }
}

/// Converts optional compare state to wgpu compare state.
pub const fn to_wgpu_compare(compare: Option<CompareFunction>) -> Option<wgpu::CompareFunction> {
    match compare {
        Some(CompareFunction::Less) => Some(wgpu::CompareFunction::Less),
        Some(CompareFunction::LessEqual) => Some(wgpu::CompareFunction::LessEqual),
        Some(CompareFunction::Greater) => Some(wgpu::CompareFunction::Greater),
        Some(CompareFunction::GreaterEqual) => Some(wgpu::CompareFunction::GreaterEqual),
        Some(CompareFunction::Equal) => Some(wgpu::CompareFunction::Equal),
        Some(CompareFunction::NotEqual) => Some(wgpu::CompareFunction::NotEqual),
        Some(CompareFunction::Always) => Some(wgpu::CompareFunction::Always),
        Some(CompareFunction::Never) => Some(wgpu::CompareFunction::Never),
        None => None,
    }
}
