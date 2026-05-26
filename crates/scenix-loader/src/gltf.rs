use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::{
    CameraId, Color, LightId, LoadError, MaterialId, MeshId, ScenixError, TextureId,
};
use scenix_light::{DirectionalLight, PointLight, SpotLight};
use scenix_material::{AlphaMode, PbrMaterial};
use scenix_math::{Quat, Transform, Vec2, Vec3, Vec4};
use scenix_mesh::Geometry;
use scenix_scene::{SceneGraph, SceneNode};
use scenix_texture::{AddressMode, FilterMode, Sampler, Texture2D, TextureFormat};

/// Loader behavior for glTF imports.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoaderOptions {
    /// Compute normals for triangle meshes that do not contain normals.
    pub generate_missing_normals: bool,
    /// Convert all decoded images to RGBA8 textures.
    pub decode_images: bool,
}

impl Default for LoaderOptions {
    #[inline]
    fn default() -> Self {
        Self {
            generate_missing_normals: true,
            decode_images: true,
        }
    }
}

/// glTF camera converted to scenix camera types.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadedCamera {
    /// Perspective projection.
    Perspective(PerspectiveCamera),
    /// Orthographic projection.
    Orthographic(OrthographicCamera),
}

/// glTF punctual light converted to scenix light types.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadedLight {
    /// Directional light.
    Directional(DirectionalLight),
    /// Point light.
    Point(PointLight),
    /// Spot light.
    Spot(SpotLight),
}

/// CPU-side scenix asset generated from a glTF file.
pub struct GltfAsset {
    /// Scene graph with mesh, camera, and light node references.
    pub scene: SceneGraph,
    /// Mesh primitives keyed by stable IDs from source order.
    pub meshes: BTreeMap<MeshId, Geometry>,
    /// PBR materials keyed by stable IDs from source order.
    pub materials: BTreeMap<MaterialId, PbrMaterial>,
    /// Decoded textures keyed by stable IDs from source texture order.
    pub textures: BTreeMap<TextureId, Texture2D>,
    /// Sampler state keyed by texture ID.
    pub samplers: BTreeMap<TextureId, Sampler>,
    /// Loaded lights keyed by stable IDs from source order.
    pub lights: BTreeMap<LightId, LoadedLight>,
    /// Loaded cameras keyed by stable IDs from source order.
    pub cameras: BTreeMap<CameraId, LoadedCamera>,
}

impl GltfAsset {
    /// Returns whether this asset contains no renderable meshes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.meshes.is_empty()
    }
}

/// glTF loader that produces CPU-side scenix assets.
#[derive(Clone, Debug, Default)]
pub struct GltfLoader {
    options: LoaderOptions,
}

impl GltfLoader {
    /// Creates a loader with default options.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a loader with explicit options.
    #[inline]
    pub const fn with_options(options: LoaderOptions) -> Self {
        Self { options }
    }

    /// Returns the loader options.
    #[inline]
    pub const fn options(&self) -> &LoaderOptions {
        &self.options
    }

    /// Loads a glTF or GLB file from disk.
    #[inline]
    pub fn load(&self, path: impl AsRef<Path>) -> Result<GltfAsset, ScenixError> {
        self.load_file(path)
    }

    /// Loads a glTF or GLB file from disk.
    pub fn load_file(&self, path: impl AsRef<Path>) -> Result<GltfAsset, ScenixError> {
        let path = path.as_ref();
        let (document, buffers, images) = gltf::import(path).map_err(|_| LoadError::Parse)?;
        let base_dir = path.parent().map(Path::to_path_buf);
        self.convert(document, buffers, images, base_dir)
    }

    /// Loads embedded glTF or GLB bytes.
    pub fn load_bytes(
        &self,
        bytes: &[u8],
        base_dir: impl Into<Option<PathBuf>>,
    ) -> Result<GltfAsset, ScenixError> {
        let (document, buffers, images) =
            gltf::import_slice(bytes).map_err(|_| LoadError::Parse)?;
        self.convert(document, buffers, images, base_dir.into())
    }

    /// Loads a glTF or GLB asset from a URL.
    #[cfg(feature = "http")]
    pub async fn load_url(&self, url: &str) -> Result<GltfAsset, ScenixError> {
        let bytes = reqwest::get(url)
            .await
            .map_err(|_| LoadError::Io)?
            .bytes()
            .await
            .map_err(|_| LoadError::Io)?;
        self.load_bytes(&bytes, None)
    }

    fn convert(
        &self,
        document: gltf::Document,
        buffers: Vec<gltf::buffer::Data>,
        images: Vec<gltf::image::Data>,
        _base_dir: Option<PathBuf>,
    ) -> Result<GltfAsset, ScenixError> {
        let mut asset = GltfAsset {
            scene: SceneGraph::new(),
            meshes: BTreeMap::new(),
            materials: BTreeMap::new(),
            textures: BTreeMap::new(),
            samplers: BTreeMap::new(),
            lights: BTreeMap::new(),
            cameras: BTreeMap::new(),
        };

        self.load_textures(&document, &images, &mut asset)?;
        self.load_materials(&document, &mut asset);
        self.load_cameras(&document, &mut asset);

        let default_material_id = if asset.materials.is_empty() {
            MaterialId::new(1)
        } else {
            MaterialId::new(asset.materials.len() as u64 + 1)
        };
        asset.materials.entry(default_material_id).or_default();

        let mut mesh_primitives = Vec::new();
        let mut next_mesh_id = 1_u64;
        for mesh in document.meshes() {
            let mut primitive_ids = Vec::new();
            for primitive in mesh.primitives() {
                if primitive.mode() != gltf::mesh::Mode::Triangles {
                    return Err(ScenixError::Load(LoadError::UnsupportedFeature));
                }

                let mesh_id = MeshId::new(next_mesh_id);
                next_mesh_id += 1;
                let material_id = material_id_for(&primitive.material(), default_material_id);
                let geometry = self.geometry_from_primitive(&primitive, &buffers)?;
                asset.meshes.insert(mesh_id, geometry);
                primitive_ids.push((mesh_id, material_id));
            }
            mesh_primitives.push(primitive_ids);
        }

        let scene = document
            .default_scene()
            .or_else(|| document.scenes().next())
            .ok_or(ScenixError::Load(LoadError::NotFound))?;
        for node in scene.nodes() {
            append_node(&mut asset.scene, node, None, &mesh_primitives)?;
        }
        asset.scene.update_world_transforms();

        Ok(asset)
    }

    fn load_textures(
        &self,
        document: &gltf::Document,
        images: &[gltf::image::Data],
        asset: &mut GltfAsset,
    ) -> Result<(), ScenixError> {
        if !self.options.decode_images {
            return Ok(());
        }

        for texture in document.textures() {
            let id = TextureId::new(texture.index() as u64 + 1);
            let source = texture.source().index();
            let image = images
                .get(source)
                .ok_or(ScenixError::Load(LoadError::NotFound))?;
            let texture_data = texture_from_gltf_image(image)?;
            asset.textures.insert(id, texture_data);
            asset
                .samplers
                .insert(id, sampler_from_gltf(texture.sampler()));
        }

        Ok(())
    }

    fn load_materials(&self, document: &gltf::Document, asset: &mut GltfAsset) {
        for material in document.materials() {
            let Some(index) = material.index() else {
                continue;
            };
            asset.materials.insert(
                MaterialId::new(index as u64 + 1),
                material_from_gltf(&material),
            );
        }
    }

    fn load_cameras(&self, document: &gltf::Document, asset: &mut GltfAsset) {
        for camera in document.cameras() {
            let index = camera.index();
            let loaded = match camera.projection() {
                gltf::camera::Projection::Perspective(perspective) => {
                    let aspect = perspective.aspect_ratio().unwrap_or(1.0);
                    let far = perspective.zfar().unwrap_or(1000.0);
                    LoadedCamera::Perspective(PerspectiveCamera::new(
                        perspective.yfov().to_degrees(),
                        aspect,
                        perspective.znear(),
                        far,
                    ))
                }
                gltf::camera::Projection::Orthographic(orthographic) => {
                    let half_x = orthographic.xmag() * 0.5;
                    let half_y = orthographic.ymag() * 0.5;
                    LoadedCamera::Orthographic(OrthographicCamera::new(
                        -half_x,
                        half_x,
                        -half_y,
                        half_y,
                        orthographic.znear(),
                        orthographic.zfar(),
                    ))
                }
            };
            asset
                .cameras
                .insert(CameraId::new(index as u64 + 1), loaded);
        }
    }

    fn geometry_from_primitive(
        &self,
        primitive: &gltf::Primitive<'_>,
        buffers: &[gltf::buffer::Data],
    ) -> Result<Geometry, ScenixError> {
        let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| &**data));
        let positions = reader
            .read_positions()
            .ok_or(ScenixError::Load(LoadError::Parse))?;

        let mut geometry = Geometry::new();
        geometry
            .positions
            .extend(positions.map(|p| Vec3::new(p[0], p[1], p[2])));

        if let Some(normals) = reader.read_normals() {
            geometry
                .normals
                .extend(normals.map(|n| Vec3::new(n[0], n[1], n[2]).normalize()));
        }
        if let Some(tangents) = reader.read_tangents() {
            geometry
                .tangents
                .extend(tangents.map(|t| Vec4::new(t[0], t[1], t[2], t[3])));
        }
        if let Some(uvs) = reader.read_tex_coords(0) {
            geometry
                .uvs
                .extend(uvs.into_f32().map(|uv| Vec2::new(uv[0], uv[1])));
        }
        if let Some(colors) = reader.read_colors(0) {
            geometry.colors.extend(
                colors
                    .into_rgba_f32()
                    .map(|c| Color::rgba(c[0], c[1], c[2], c[3])),
            );
        }
        if let Some(indices) = reader.read_indices() {
            geometry.indices.extend(indices.into_u32());
        } else {
            geometry
                .indices
                .extend((0..geometry.positions.len()).map(|index| index as u32));
        }

        if geometry.normals.is_empty() && self.options.generate_missing_normals {
            geometry.compute_normals();
        }
        geometry.validate()?;
        Ok(geometry)
    }
}

fn material_id_for(material: &gltf::Material<'_>, default_material_id: MaterialId) -> MaterialId {
    material
        .index()
        .map(|index| MaterialId::new(index as u64 + 1))
        .unwrap_or(default_material_id)
}

fn material_from_gltf(material: &gltf::Material<'_>) -> PbrMaterial {
    let pbr = material.pbr_metallic_roughness();
    let base = pbr.base_color_factor();
    let mut out = PbrMaterial::new()
        .named(material.name().unwrap_or_default())
        .albedo(Color::rgba(base[0], base[1], base[2], base[3]))
        .metallic_roughness(pbr.metallic_factor(), pbr.roughness_factor())
        .double_sided(material.double_sided());

    out.albedo_texture = pbr
        .base_color_texture()
        .map(|info| TextureId::new(info.texture().index() as u64 + 1));
    out.metallic_roughness_texture = pbr
        .metallic_roughness_texture()
        .map(|info| TextureId::new(info.texture().index() as u64 + 1));
    out.normal_texture = material
        .normal_texture()
        .map(|info| TextureId::new(info.texture().index() as u64 + 1));
    out.occlusion_texture = material
        .occlusion_texture()
        .map(|info| TextureId::new(info.texture().index() as u64 + 1));
    out.emissive_texture = material
        .emissive_texture()
        .map(|info| TextureId::new(info.texture().index() as u64 + 1));

    let emissive = material.emissive_factor();
    out.emissive = Vec3::new(emissive[0], emissive[1], emissive[2]);
    out.alpha_mode = match material.alpha_mode() {
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => AlphaMode::Mask(material.alpha_cutoff().unwrap_or(0.5)),
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
    };
    out
}

fn append_node(
    graph: &mut SceneGraph,
    node: gltf::Node<'_>,
    parent: Option<scenix_core::NodeId>,
    mesh_primitives: &[Vec<(MeshId, MaterialId)>],
) -> Result<(), ScenixError> {
    let name = node.name().unwrap_or("node");
    let transform = transform_from_gltf_node(&node);
    let current = if let Some(mesh) = node.mesh() {
        let primitives = mesh_primitives
            .get(mesh.index())
            .ok_or(ScenixError::Load(LoadError::Parse))?;
        if primitives.len() == 1 {
            let (mesh_id, material_id) = primitives[0];
            add_scene_node(
                graph,
                parent,
                SceneNode::mesh(name, mesh_id, material_id).transform(transform),
            )?
        } else {
            let group = add_scene_node(graph, parent, SceneNode::group(name).transform(transform))?;
            for (index, (mesh_id, material_id)) in primitives.iter().copied().enumerate() {
                graph.add_child(
                    group,
                    SceneNode::mesh(format!("{name}.primitive-{index}"), mesh_id, material_id),
                )?;
            }
            group
        }
    } else if let Some(camera) = node.camera() {
        add_scene_node(
            graph,
            parent,
            SceneNode::camera(name, CameraId::new(camera.index() as u64 + 1)).transform(transform),
        )?
    } else {
        add_scene_node(graph, parent, SceneNode::group(name).transform(transform))?
    };

    for child in node.children() {
        append_node(graph, child, Some(current), mesh_primitives)?;
    }

    Ok(())
}

fn add_scene_node(
    graph: &mut SceneGraph,
    parent: Option<scenix_core::NodeId>,
    node: SceneNode,
) -> Result<scenix_core::NodeId, ScenixError> {
    if let Some(parent) = parent {
        graph.add_child(parent, node).map_err(ScenixError::from)
    } else {
        Ok(graph.add(node))
    }
}

fn transform_from_gltf_node(node: &gltf::Node<'_>) -> Transform {
    let (translation, rotation, scale) = node.transform().decomposed();
    Transform::new(
        Vec3::new(translation[0], translation[1], translation[2]),
        Quat::new(rotation[0], rotation[1], rotation[2], rotation[3]).normalize(),
        Vec3::new(scale[0], scale[1], scale[2]),
    )
}

fn texture_from_gltf_image(image: &gltf::image::Data) -> Result<Texture2D, ScenixError> {
    let data = rgba8_from_gltf_image(image)?;
    Texture2D::new(
        image.width,
        image.height,
        TextureFormat::Rgba8UnormSrgb,
        data,
    )
    .map_err(ScenixError::from)
}

fn rgba8_from_gltf_image(image: &gltf::image::Data) -> Result<Vec<u8>, ScenixError> {
    use gltf::image::Format;

    let pixel_count = image
        .width
        .checked_mul(image.height)
        .ok_or(ScenixError::Load(LoadError::Parse))? as usize;
    let mut rgba = Vec::with_capacity(pixel_count * 4);

    match image.format {
        Format::R8 => {
            for pixel in image.pixels.iter().copied() {
                rgba.extend_from_slice(&[pixel, pixel, pixel, 255]);
            }
        }
        Format::R8G8 => {
            for pixel in image.pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
            }
        }
        Format::R8G8B8 => {
            for pixel in image.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
        }
        Format::R8G8B8A8 => rgba.extend_from_slice(&image.pixels),
        Format::R16 | Format::R16G16 | Format::R16G16B16 | Format::R16G16B16A16 => {
            let channels = match image.format {
                Format::R16 => 1,
                Format::R16G16 => 2,
                Format::R16G16B16 => 3,
                Format::R16G16B16A16 => 4,
                _ => unreachable!(),
            };
            for pixel in image.pixels.chunks_exact(channels * 2) {
                let r = pixel[1];
                let g = if channels > 1 { pixel[3] } else { r };
                let b = if channels > 2 { pixel[5] } else { r };
                let a = if channels > 3 { pixel[7] } else { 255 };
                rgba.extend_from_slice(&[r, g, b, a]);
            }
        }
        _ => return Err(ScenixError::Load(LoadError::UnsupportedFormat)),
    }

    Ok(rgba)
}

fn sampler_from_gltf(sampler: gltf::texture::Sampler<'_>) -> Sampler {
    Sampler::new()
        .filters(
            match sampler.mag_filter() {
                Some(gltf::texture::MagFilter::Nearest) => FilterMode::Nearest,
                Some(gltf::texture::MagFilter::Linear) | None => FilterMode::Linear,
            },
            match sampler.min_filter() {
                Some(gltf::texture::MinFilter::Nearest)
                | Some(gltf::texture::MinFilter::NearestMipmapNearest)
                | Some(gltf::texture::MinFilter::NearestMipmapLinear) => FilterMode::Nearest,
                Some(gltf::texture::MinFilter::Linear)
                | Some(gltf::texture::MinFilter::LinearMipmapNearest)
                | Some(gltf::texture::MinFilter::LinearMipmapLinear)
                | None => FilterMode::Linear,
            },
            match sampler.min_filter() {
                Some(gltf::texture::MinFilter::NearestMipmapNearest)
                | Some(gltf::texture::MinFilter::LinearMipmapNearest) => FilterMode::Nearest,
                _ => FilterMode::Linear,
            },
        )
        .address_modes(
            address_mode_from_gltf(sampler.wrap_s()),
            address_mode_from_gltf(sampler.wrap_t()),
            AddressMode::ClampToEdge,
        )
}

fn address_mode_from_gltf(mode: gltf::texture::WrappingMode) -> AddressMode {
    match mode {
        gltf::texture::WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
        gltf::texture::WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
        gltf::texture::WrappingMode::Repeat => AddressMode::Repeat,
    }
}
