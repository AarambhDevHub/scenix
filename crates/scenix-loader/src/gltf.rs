use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::{
    AnimationClipId, AssetId, CameraId, Color, LightId, LoadError, MaterialId, MeshId, ScenixError,
    SkinId, TextureId,
};
use scenix_light::{DirectionalLight, PointLight, SpotLight};
use scenix_material::{AlphaMode, PbrMaterial, PhysicalMaterial, UnlitMaterial};
use scenix_math::{Mat4, Quat, Transform, Vec2, Vec3, Vec4};
use scenix_mesh::{Geometry, MorphTarget};
use scenix_scene::{SceneGraph, SceneNode};
use scenix_texture::{AddressMode, FilterMode, Sampler, Texture2D, TextureFormat};

use crate::asset::{
    AssetDependencyGraph, AssetDiagnostic, AssetPackage, AssetSource, LoadedAnimationChannel,
    LoadedAnimationClip, LoadedAnimationInterpolation, LoadedAnimationProperty, LoadedMaterial,
    LoadedMeshSkinAttributes, LoadedSkin, MaterialVariant, TextureTransform,
};

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

    /// Loads a glTF or GLB file into a v1.3 asset package with extension metadata.
    pub fn load_package_file(&self, path: impl AsRef<Path>) -> Result<AssetPackage, ScenixError> {
        let path = path.as_ref();
        let (document, buffers, images) = gltf::import(path).map_err(|_| LoadError::Parse)?;
        let base_dir = path.parent().map(Path::to_path_buf);
        let mut graph = AssetDependencyGraph::new();
        graph.record_path(path);
        let mut package = self.convert_package(
            document,
            buffers,
            images,
            base_dir,
            AssetId::new(1),
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("gltf"),
        )?;
        package.source = Some(AssetSource::Path(path.to_path_buf()));
        package.dependency_graph = graph;
        Ok(package)
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

    /// Loads embedded glTF or GLB bytes into a v1.3 asset package.
    pub fn load_package_bytes(
        &self,
        bytes: &[u8],
        base_dir: impl Into<Option<PathBuf>>,
    ) -> Result<AssetPackage, ScenixError> {
        let (document, buffers, images) =
            gltf::import_slice(bytes).map_err(|_| LoadError::Parse)?;
        let mut package = self.convert_package(
            document,
            buffers,
            images,
            base_dir.into(),
            AssetId::new(1),
            "embedded-gltf",
        )?;
        package.source = Some(AssetSource::Bytes {
            label: String::from("embedded-gltf"),
            len: bytes.len(),
        });
        package
            .dependency_graph
            .record_bytes("embedded-gltf", bytes.len());
        Ok(package)
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

    /// Loads a glTF or GLB asset package from a URL.
    #[cfg(feature = "http")]
    pub async fn load_package_url(&self, url: &str) -> Result<AssetPackage, ScenixError> {
        let bytes = reqwest::get(url)
            .await
            .map_err(|_| LoadError::Io)?
            .bytes()
            .await
            .map_err(|_| LoadError::Io)?;
        let mut package = self.load_package_bytes(&bytes, None)?;
        package.source = Some(AssetSource::Url(url.to_owned()));
        package.dependency_graph = AssetDependencyGraph::new();
        package.dependency_graph.record_url(url, bytes.len());
        Ok(package)
    }

    fn convert_package(
        &self,
        document: gltf::Document,
        buffers: Vec<gltf::buffer::Data>,
        images: Vec<gltf::image::Data>,
        base_dir: Option<PathBuf>,
        asset_id: AssetId,
        label: impl Into<String>,
    ) -> Result<AssetPackage, ScenixError> {
        let metadata = collect_package_metadata(&document, &buffers);
        let asset = self.convert(document, buffers, images, base_dir)?;
        let mut package = AssetPackage::from_gltf_asset(asset_id, label, asset);
        package.loaded_materials = metadata.loaded_materials;
        for (id, material) in &package.loaded_materials {
            package.materials.insert(*id, material.base_pbr());
        }
        package.morph_targets = metadata.morph_targets;
        package.mesh_skin_attributes = metadata.mesh_skin_attributes;
        package.skins = metadata.skins;
        package.animations = metadata.animations;
        package.texture_transforms = metadata.texture_transforms;
        package.material_variants = metadata.material_variants;
        package.diagnostics = metadata.diagnostics;
        Ok(package)
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

struct GltfPackageMetadata {
    loaded_materials: BTreeMap<MaterialId, LoadedMaterial>,
    morph_targets: BTreeMap<MeshId, Vec<MorphTarget>>,
    mesh_skin_attributes: BTreeMap<MeshId, LoadedMeshSkinAttributes>,
    skins: BTreeMap<SkinId, LoadedSkin>,
    animations: BTreeMap<AnimationClipId, LoadedAnimationClip>,
    texture_transforms: BTreeMap<(MaterialId, String), TextureTransform>,
    material_variants: Vec<MaterialVariant>,
    diagnostics: Vec<AssetDiagnostic>,
}

fn collect_package_metadata(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> GltfPackageMetadata {
    let mut metadata = GltfPackageMetadata {
        loaded_materials: BTreeMap::new(),
        morph_targets: BTreeMap::new(),
        mesh_skin_attributes: BTreeMap::new(),
        skins: BTreeMap::new(),
        animations: BTreeMap::new(),
        texture_transforms: BTreeMap::new(),
        material_variants: Vec::new(),
        diagnostics: Vec::new(),
    };

    collect_extension_diagnostics(document, &mut metadata.diagnostics);
    collect_material_metadata(document, &mut metadata);
    collect_skin_metadata(document, buffers, &mut metadata);
    collect_mesh_metadata(document, buffers, &mut metadata);
    collect_animation_metadata(document, buffers, &mut metadata);
    metadata
}

fn collect_extension_diagnostics(
    document: &gltf::Document,
    diagnostics: &mut Vec<AssetDiagnostic>,
) {
    for extension in document.extensions_used() {
        match extension {
            "KHR_draco_mesh_compression" => diagnostics.push(AssetDiagnostic::unsupported(
                "gltf.draco",
                "Draco-compressed meshes are detected but not decoded by the default v1.3 loader",
            )),
            "EXT_meshopt_compression" => diagnostics.push(AssetDiagnostic::unsupported(
                "gltf.meshopt",
                "meshopt-compressed buffers are detected but not decoded by the default v1.3 loader",
            )),
            "KHR_texture_basisu" => diagnostics.push(AssetDiagnostic::warning(
                "gltf.basisu",
                "BasisU textures are represented through KTX2 metadata when the container is accessible",
            )),
            "KHR_materials_clearcoat" | "KHR_materials_sheen" | "KHR_materials_iridescence" => {
                diagnostics.push(AssetDiagnostic::warning(
                    format!("gltf.{extension}"),
                    format!("{extension} is detected and preserved as material metadata diagnostics"),
                ));
            }
            _ => {}
        }
    }
    for extension in document.extensions_required() {
        if matches!(
            extension,
            "KHR_draco_mesh_compression" | "EXT_meshopt_compression"
        ) {
            diagnostics.push(AssetDiagnostic::unsupported(
                format!("gltf.required.{extension}"),
                format!("required extension {extension} needs an external pre-processing step"),
            ));
        }
    }
}

fn collect_material_metadata(document: &gltf::Document, metadata: &mut GltfPackageMetadata) {
    for material in document.materials() {
        let Some(index) = material.index() else {
            continue;
        };
        let id = MaterialId::new(index as u64 + 1);
        let base = material_from_gltf(&material);
        let loaded = if material.unlit() {
            let mut unlit = UnlitMaterial::new()
                .color(base.albedo)
                .alpha_mode(base.alpha_mode);
            unlit.name = base.name.clone();
            unlit.color_texture = base.albedo_texture;
            unlit.double_sided = base.double_sided;
            LoadedMaterial::Unlit(unlit)
        } else if has_physical_extensions(&material) {
            LoadedMaterial::Physical(physical_material_from_gltf(&material, base.clone()))
        } else {
            LoadedMaterial::Pbr(base)
        };
        metadata.loaded_materials.insert(id, loaded);
        record_material_texture_transforms(id, &material, &mut metadata.texture_transforms);
    }

    if let Some(variants) = document.variants() {
        for (index, variant) in variants.enumerate() {
            metadata.material_variants.push(MaterialVariant {
                index: index as u32,
                name: variant.name().to_owned(),
            });
        }
    }
}

fn collect_skin_metadata(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    metadata: &mut GltfPackageMetadata,
) {
    for skin in document.skins() {
        let id = SkinId::new(skin.index() as u64 + 1);
        let joints: Vec<_> = skin.joints().map(|joint| joint.index()).collect();
        let inverse_bind_matrices = skin
            .reader(|buffer| buffers.get(buffer.index()).map(|data| &**data))
            .read_inverse_bind_matrices()
            .map(|matrices| matrices.map(mat4_from_gltf).collect())
            .unwrap_or_else(|| vec![Mat4::IDENTITY; joints.len()]);
        metadata.skins.insert(
            id,
            LoadedSkin {
                id,
                name: skin.name().unwrap_or_default().to_owned(),
                joints,
                skeleton_root: skin.skeleton().map(|node| node.index()),
                inverse_bind_matrices,
            },
        );
    }
}

fn collect_mesh_metadata(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    metadata: &mut GltfPackageMetadata,
) {
    let mut primitive_ids_by_mesh: Vec<Vec<MeshId>> = Vec::new();
    let mut next_mesh_id = 1_u64;
    for mesh in document.meshes() {
        let mut primitive_ids = Vec::new();
        for primitive in mesh.primitives() {
            let mesh_id = MeshId::new(next_mesh_id);
            next_mesh_id += 1;
            primitive_ids.push(mesh_id);
            let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| &**data));
            let morphs = morph_targets_from_reader(&reader);
            if !morphs.is_empty() {
                metadata.morph_targets.insert(mesh_id, morphs);
            }
            let joints: Vec<[u16; 4]> = reader
                .read_joints(0)
                .map(|joints| joints.into_u16().collect())
                .unwrap_or_default();
            let weights: Vec<[f32; 4]> = reader
                .read_weights(0)
                .map(|weights| weights.into_f32().collect())
                .unwrap_or_default();
            if !joints.is_empty() || !weights.is_empty() {
                metadata.mesh_skin_attributes.insert(
                    mesh_id,
                    LoadedMeshSkinAttributes {
                        joints,
                        weights,
                        skin: None,
                    },
                );
            }
        }
        primitive_ids_by_mesh.push(primitive_ids);
    }

    for node in document.nodes() {
        let (Some(mesh), Some(skin)) = (node.mesh(), node.skin()) else {
            continue;
        };
        let skin_id = SkinId::new(skin.index() as u64 + 1);
        if let Some(ids) = primitive_ids_by_mesh.get(mesh.index()) {
            for mesh_id in ids {
                metadata
                    .mesh_skin_attributes
                    .entry(*mesh_id)
                    .or_default()
                    .skin = Some(skin_id);
            }
        }
    }
}

fn collect_animation_metadata(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    metadata: &mut GltfPackageMetadata,
) {
    for animation in document.animations() {
        let id = AnimationClipId::new(animation.index() as u64 + 1);
        let mut duration = 0.0_f32;
        let mut channels = Vec::new();
        for channel in animation.channels() {
            let reader = channel.reader(|buffer| buffers.get(buffer.index()).map(|data| &**data));
            let times: Vec<f32> = reader
                .read_inputs()
                .map(|inputs| inputs.collect())
                .unwrap_or_default();
            if let Some(last) = times.last().copied() {
                duration = duration.max(last);
            }
            let sampler = channel.sampler();
            let output = sampler.output();
            let input_count = sampler.input().count().max(1);
            let output_components = match channel.target().property() {
                gltf::animation::Property::Translation => 3,
                gltf::animation::Property::Rotation => 4,
                gltf::animation::Property::Scale => 3,
                gltf::animation::Property::MorphTargetWeights => output.count() / input_count,
            };

            // v1.4.0: decode the output accessor bytes into flat `f32` values
            // so the runtime `clip_from_loaded` bridge can build keyframe
            // tracks without re-reading the glTF accessor.
            let output_values: Vec<f32> = match reader.read_outputs() {
                Some(gltf::animation::util::ReadOutputs::Translations(ts)) => {
                    ts.flat_map(|v| v.into_iter()).collect()
                }
                Some(gltf::animation::util::ReadOutputs::Scales(sc)) => {
                    sc.flat_map(|v| v.into_iter()).collect()
                }
                Some(gltf::animation::util::ReadOutputs::Rotations(rs)) => {
                    rs.into_f32().flat_map(|v| v.into_iter()).collect()
                }
                Some(gltf::animation::util::ReadOutputs::MorphTargetWeights(ws)) => {
                    ws.into_f32().collect()
                }
                None => Vec::new(),
            };

            channels.push(LoadedAnimationChannel {
                node_index: channel.target().node().index(),
                property: animation_property(channel.target().property()),
                interpolation: animation_interpolation(sampler.interpolation()),
                times,
                output: output_values,
                output_components,
            });
        }
        metadata.animations.insert(
            id,
            LoadedAnimationClip {
                id,
                name: animation.name().unwrap_or_default().to_owned(),
                duration,
                channels,
            },
        );
    }
}

fn morph_targets_from_reader<'a, 's, F>(reader: &gltf::mesh::Reader<'a, 's, F>) -> Vec<MorphTarget>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_morph_targets()
        .enumerate()
        .map(|(index, (positions, normals, _tangents))| {
            let mut target = MorphTarget::new(format!("morph-{index}"));
            if let Some(positions) = positions {
                target.positions_delta = positions
                    .map(|position| Vec3::new(position[0], position[1], position[2]))
                    .collect();
            }
            if let Some(normals) = normals {
                target.normals_delta = normals
                    .map(|normal| Vec3::new(normal[0], normal[1], normal[2]).normalize())
                    .collect();
            }
            target
        })
        .collect()
}

fn has_physical_extensions(material: &gltf::Material<'_>) -> bool {
    material.transmission().is_some()
        || material.volume().is_some()
        || material.specular().is_some()
        || material.ior().is_some()
        || material.emissive_strength().is_some()
        || material
            .extension_value("KHR_materials_clearcoat")
            .is_some()
        || material.extension_value("KHR_materials_sheen").is_some()
        || material
            .extension_value("KHR_materials_iridescence")
            .is_some()
}

fn physical_material_from_gltf(
    material: &gltf::Material<'_>,
    mut base: PbrMaterial,
) -> PhysicalMaterial {
    if let Some(strength) = material.emissive_strength() {
        base.emissive *= strength;
    }
    let mut physical = PhysicalMaterial::new().base(base);
    if let Some(transmission) = material.transmission() {
        physical.transmission = transmission.transmission_factor().clamp(0.0, 1.0);
        physical.base.alpha_mode = AlphaMode::Blend;
    }
    if let Some(volume) = material.volume() {
        physical.thickness = volume.thickness_factor().max(0.0);
    }
    if let Some(ior) = material.ior() {
        physical.ior = ior.max(1.0);
    }
    if let Some(specular) = material.specular() {
        physical.sheen = specular.specular_factor().clamp(0.0, 1.0);
        let color = specular.specular_color_factor();
        physical.sheen_color = Color::rgb(color[0], color[1], color[2]);
    }
    physical
}

fn record_material_texture_transforms(
    material_id: MaterialId,
    material: &gltf::Material<'_>,
    transforms: &mut BTreeMap<(MaterialId, String), TextureTransform>,
) {
    let pbr = material.pbr_metallic_roughness();
    if let Some(info) = pbr.base_color_texture() {
        record_texture_transform(material_id, "albedo", info, transforms);
    }
    if let Some(info) = pbr.metallic_roughness_texture() {
        record_texture_transform(material_id, "metallic_roughness", info, transforms);
    }
    // glTF's normal/occlusion wrappers expose extension maps but not the shared
    // texture::Info helper, so v1.3 records typed transforms for slots exposed
    // through texture::Info and leaves these slots as diagnostics/future work.
    if let Some(info) = material.emissive_texture() {
        record_texture_transform(material_id, "emissive", info, transforms);
    }
    if let Some(transmission) = material.transmission()
        && let Some(info) = transmission.transmission_texture()
    {
        record_texture_transform(material_id, "transmission", info, transforms);
    }
    if let Some(volume) = material.volume()
        && let Some(info) = volume.thickness_texture()
    {
        record_texture_transform(material_id, "thickness", info, transforms);
    }
    if let Some(specular) = material.specular() {
        if let Some(info) = specular.specular_texture() {
            record_texture_transform(material_id, "specular", info, transforms);
        }
        if let Some(info) = specular.specular_color_texture() {
            record_texture_transform(material_id, "specular_color", info, transforms);
        }
    }
}

fn record_texture_transform(
    material_id: MaterialId,
    slot: &str,
    info: gltf::texture::Info<'_>,
    transforms: &mut BTreeMap<(MaterialId, String), TextureTransform>,
) {
    if let Some(transform) = info.texture_transform() {
        let offset = transform.offset();
        let scale = transform.scale();
        transforms.insert(
            (material_id, slot.to_owned()),
            TextureTransform {
                offset: Vec2::new(offset[0], offset[1]),
                rotation: transform.rotation(),
                scale: Vec2::new(scale[0], scale[1]),
                tex_coord: transform.tex_coord(),
            },
        );
    }
}

fn animation_property(property: gltf::animation::Property) -> LoadedAnimationProperty {
    match property {
        gltf::animation::Property::Translation => LoadedAnimationProperty::Translation,
        gltf::animation::Property::Rotation => LoadedAnimationProperty::Rotation,
        gltf::animation::Property::Scale => LoadedAnimationProperty::Scale,
        gltf::animation::Property::MorphTargetWeights => {
            LoadedAnimationProperty::MorphTargetWeights
        }
    }
}

fn animation_interpolation(
    interpolation: gltf::animation::Interpolation,
) -> LoadedAnimationInterpolation {
    match interpolation {
        gltf::animation::Interpolation::Linear => LoadedAnimationInterpolation::Linear,
        gltf::animation::Interpolation::Step => LoadedAnimationInterpolation::Step,
        gltf::animation::Interpolation::CubicSpline => LoadedAnimationInterpolation::CubicSpline,
    }
}

fn mat4_from_gltf(values: [[f32; 4]; 4]) -> Mat4 {
    Mat4::from_cols_array([
        values[0][0],
        values[0][1],
        values[0][2],
        values[0][3],
        values[1][0],
        values[1][1],
        values[1][2],
        values[1][3],
        values[2][0],
        values[2][1],
        values[2][2],
        values[2][3],
        values[3][0],
        values[3][1],
        values[3][2],
        values[3][3],
    ])
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
