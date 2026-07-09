use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use scenix_core::{
    AnimationClipId, AssetId, CameraId, LightId, MaterialId, MeshId, ScenixError, SkinId, TextureId,
};
use scenix_material::{PbrMaterial, PhysicalMaterial, UnlitMaterial};
use scenix_math::{Mat4, Vec2, Vec3};
use scenix_mesh::{Geometry, MorphTarget};
use scenix_scene::SceneGraph;
use scenix_texture::{Sampler, Texture2D, TextureCube};

use crate::gltf::{GltfAsset, LoadedCamera, LoadedLight};

/// Source that produced an [`AssetPackage`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetSource {
    /// Local file path.
    Path(PathBuf),
    /// Remote URL.
    Url(String),
    /// Caller-owned byte buffer.
    Bytes { label: String, len: usize },
}

/// File or byte request for the asset manager.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetRequest {
    /// Stable request identifier.
    pub id: AssetId,
    /// Source to load.
    pub source: AssetSource,
}

impl AssetRequest {
    /// Creates a path request.
    #[inline]
    pub fn path(id: AssetId, path: impl Into<PathBuf>) -> Self {
        Self {
            id,
            source: AssetSource::Path(path.into()),
        }
    }

    /// Creates a URL request.
    #[inline]
    pub fn url(id: AssetId, url: impl Into<String>) -> Self {
        Self {
            id,
            source: AssetSource::Url(url.into()),
        }
    }

    /// Creates an embedded byte request.
    #[inline]
    pub fn bytes(id: AssetId, label: impl Into<String>, len: usize) -> Self {
        Self {
            id,
            source: AssetSource::Bytes {
                label: label.into(),
                len,
            },
        }
    }
}

/// Severity for import diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetDiagnosticSeverity {
    /// Informational metadata.
    Info,
    /// Feature was imported with a fallback or partial mapping.
    Warning,
    /// Feature cannot be imported by this build.
    Unsupported,
}

/// Structured import/export diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetDiagnostic {
    /// Severity.
    pub severity: AssetDiagnosticSeverity,
    /// Machine-readable code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl AssetDiagnostic {
    /// Creates an informational diagnostic.
    #[inline]
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(AssetDiagnosticSeverity::Info, code, message)
    }

    /// Creates a warning diagnostic.
    #[inline]
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(AssetDiagnosticSeverity::Warning, code, message)
    }

    /// Creates an unsupported-feature diagnostic.
    #[inline]
    pub fn unsupported(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(AssetDiagnosticSeverity::Unsupported, code, message)
    }

    /// Creates a diagnostic.
    #[inline]
    pub fn new(
        severity: AssetDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
        }
    }
}

/// One tracked asset dependency.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetDependency {
    /// Source path, URL, or byte label.
    pub source: AssetSource,
    /// Best-effort byte size.
    pub bytes: usize,
    /// Last modification timestamp for local paths.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub modified: Option<SystemTime>,
}

/// Dependency graph for invalidation and hot reload.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetDependencyGraph {
    /// Dependencies in deterministic source order.
    pub dependencies: Vec<AssetDependency>,
}

impl AssetDependencyGraph {
    /// Creates an empty graph.
    #[inline]
    pub const fn new() -> Self {
        Self {
            dependencies: Vec::new(),
        }
    }

    /// Records a local path.
    pub fn record_path(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path).ok();
        self.dependencies.push(AssetDependency {
            source: AssetSource::Path(path.to_path_buf()),
            bytes: metadata.as_ref().map_or(0, |m| m.len() as usize),
            modified: metadata.and_then(|m| m.modified().ok()),
        });
    }

    /// Records a remote URL.
    pub fn record_url(&mut self, url: impl Into<String>, bytes: usize) {
        self.dependencies.push(AssetDependency {
            source: AssetSource::Url(url.into()),
            bytes,
            modified: None,
        });
    }

    /// Records an embedded byte dependency.
    pub fn record_bytes(&mut self, label: impl Into<String>, bytes: usize) {
        self.dependencies.push(AssetDependency {
            source: AssetSource::Bytes {
                label: label.into(),
                len: bytes,
            },
            bytes,
            modified: None,
        });
    }

    /// Returns total tracked dependency bytes.
    #[inline]
    pub fn total_bytes(&self) -> usize {
        self.dependencies.iter().map(|dep| dep.bytes).sum()
    }

    /// Returns whether any local dependency has changed on disk.
    pub fn is_stale(&self) -> bool {
        self.dependencies.iter().any(|dep| {
            let AssetSource::Path(path) = &dep.source else {
                return false;
            };
            let Ok(metadata) = std::fs::metadata(path) else {
                return true;
            };
            metadata.modified().ok() != dep.modified
        })
    }
}

/// Texture transform imported from `KHR_texture_transform`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextureTransform {
    /// UV offset.
    pub offset: Vec2,
    /// UV rotation in radians.
    pub rotation: f32,
    /// UV scale.
    pub scale: Vec2,
    /// Optional overriding texcoord set.
    pub tex_coord: Option<u32>,
}

impl Default for TextureTransform {
    #[inline]
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            tex_coord: None,
        }
    }
}

/// Material variant metadata imported from `KHR_materials_variants`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialVariant {
    /// Source variant index.
    pub index: u32,
    /// Source variant name if available.
    pub name: String,
}

/// Renderer-agnostic material imported by the asset pipeline.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadedMaterial {
    /// Metallic-roughness PBR material.
    Pbr(PbrMaterial),
    /// Advanced physical material mapped from glTF extensions.
    Physical(PhysicalMaterial),
    /// Unlit material mapped from `KHR_materials_unlit`.
    Unlit(UnlitMaterial),
}

impl LoadedMaterial {
    /// Returns the PBR-compatible base material.
    #[inline]
    pub fn base_pbr(&self) -> PbrMaterial {
        match self {
            Self::Pbr(material) => material.clone(),
            Self::Physical(material) => material.base.clone(),
            Self::Unlit(material) => PbrMaterial::new()
                .named(material.name.clone())
                .albedo(material.color)
                .alpha_mode(material.alpha_mode)
                .double_sided(material.double_sided),
        }
    }
}

/// Imported skeleton/skin metadata.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadedSkin {
    /// Skin ID.
    pub id: SkinId,
    /// Human-readable name.
    pub name: String,
    /// Source joint node indices.
    pub joints: Vec<usize>,
    /// Source skeleton root node index.
    pub skeleton_root: Option<usize>,
    /// Inverse bind matrices in joint order.
    pub inverse_bind_matrices: Vec<Mat4>,
}

/// Per-vertex skin attributes imported for one mesh primitive.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadedMeshSkinAttributes {
    /// Joint indices in the first glTF joint set.
    pub joints: Vec<[u16; 4]>,
    /// Joint weights in the first glTF weight set.
    pub weights: Vec<[f32; 4]>,
    /// Skin used by the node instantiating this mesh, when known.
    pub skin: Option<SkinId>,
}

/// Imported animation target property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadedAnimationProperty {
    /// Node translation.
    Translation,
    /// Node rotation.
    Rotation,
    /// Node scale.
    Scale,
    /// Morph target weights.
    MorphTargetWeights,
}

/// Imported animation interpolation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadedAnimationInterpolation {
    /// Linear interpolation.
    Linear,
    /// Step interpolation.
    Step,
    /// Cubic spline interpolation.
    CubicSpline,
}

/// One imported animation channel.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadedAnimationChannel {
    /// Source node index.
    pub node_index: usize,
    /// Target property.
    pub property: LoadedAnimationProperty,
    /// Interpolation mode.
    pub interpolation: LoadedAnimationInterpolation,
    /// Keyframe times in seconds.
    pub times: Vec<f32>,
    /// Decoded output values packed as `output_components` per keyframe (v1.4.0).
    ///
    /// For `CubicSpline` interpolation the layout per keyframe is
    /// `[in_tangent, value, out_tangent]`, matching the glTF spec.
    pub output: Vec<f32>,
    /// Output component count for one keyframe value.
    pub output_components: usize,
}

/// Imported animation clip metadata and keyframe times.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadedAnimationClip {
    /// Clip ID.
    pub id: AnimationClipId,
    /// Human-readable name.
    pub name: String,
    /// Duration in seconds.
    pub duration: f32,
    /// Channels in source order.
    pub channels: Vec<LoadedAnimationChannel>,
}

/// CPU-side package produced by the v1.3 asset pipeline.
pub struct AssetPackage {
    /// Stable package ID.
    pub id: AssetId,
    /// Human-readable label.
    pub label: String,
    /// Source that produced this package.
    pub source: Option<AssetSource>,
    /// Renderable scene graph.
    pub scene: SceneGraph,
    /// Imported mesh geometries.
    pub meshes: BTreeMap<MeshId, Geometry>,
    /// PBR-compatible materials for existing renderer paths.
    pub materials: BTreeMap<MaterialId, PbrMaterial>,
    /// Full imported material variants.
    pub loaded_materials: BTreeMap<MaterialId, LoadedMaterial>,
    /// 2D textures.
    pub textures: BTreeMap<TextureId, Texture2D>,
    /// Cube textures.
    pub texture_cubes: BTreeMap<TextureId, TextureCube>,
    /// Samplers keyed by texture ID.
    pub samplers: BTreeMap<TextureId, Sampler>,
    /// Imported lights.
    pub lights: BTreeMap<LightId, LoadedLight>,
    /// Imported cameras.
    pub cameras: BTreeMap<CameraId, LoadedCamera>,
    /// Morph targets keyed by mesh ID.
    pub morph_targets: BTreeMap<MeshId, Vec<MorphTarget>>,
    /// Skinning attributes keyed by mesh ID.
    pub mesh_skin_attributes: BTreeMap<MeshId, LoadedMeshSkinAttributes>,
    /// Imported skins.
    pub skins: BTreeMap<SkinId, LoadedSkin>,
    /// Imported animation clips.
    pub animations: BTreeMap<AnimationClipId, LoadedAnimationClip>,
    /// Texture transforms keyed by material ID and material slot name.
    pub texture_transforms: BTreeMap<(MaterialId, String), TextureTransform>,
    /// Material variants.
    pub material_variants: Vec<MaterialVariant>,
    /// Dependency graph.
    pub dependency_graph: AssetDependencyGraph,
    /// Import diagnostics.
    pub diagnostics: Vec<AssetDiagnostic>,
}

impl AssetPackage {
    /// Creates an empty package.
    pub fn empty(id: AssetId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            source: None,
            scene: SceneGraph::new(),
            meshes: BTreeMap::new(),
            materials: BTreeMap::new(),
            loaded_materials: BTreeMap::new(),
            textures: BTreeMap::new(),
            texture_cubes: BTreeMap::new(),
            samplers: BTreeMap::new(),
            lights: BTreeMap::new(),
            cameras: BTreeMap::new(),
            morph_targets: BTreeMap::new(),
            mesh_skin_attributes: BTreeMap::new(),
            skins: BTreeMap::new(),
            animations: BTreeMap::new(),
            texture_transforms: BTreeMap::new(),
            material_variants: Vec::new(),
            dependency_graph: AssetDependencyGraph::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Converts the stable v1 glTF asset into a package.
    pub fn from_gltf_asset(id: AssetId, label: impl Into<String>, asset: GltfAsset) -> Self {
        let loaded_materials = asset
            .materials
            .iter()
            .map(|(id, material)| (*id, LoadedMaterial::Pbr(material.clone())))
            .collect();
        Self {
            id,
            label: label.into(),
            source: None,
            scene: asset.scene,
            meshes: asset.meshes,
            materials: asset.materials,
            loaded_materials,
            textures: asset.textures,
            texture_cubes: BTreeMap::new(),
            samplers: asset.samplers,
            lights: asset.lights,
            cameras: asset.cameras,
            morph_targets: BTreeMap::new(),
            mesh_skin_attributes: BTreeMap::new(),
            skins: BTreeMap::new(),
            animations: BTreeMap::new(),
            texture_transforms: BTreeMap::new(),
            material_variants: Vec::new(),
            dependency_graph: AssetDependencyGraph::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Converts this package back into the stable glTF asset shape.
    pub fn into_gltf_asset(self) -> GltfAsset {
        GltfAsset {
            scene: self.scene,
            meshes: self.meshes,
            materials: self.materials,
            textures: self.textures,
            samplers: self.samplers,
            lights: self.lights,
            cameras: self.cameras,
        }
    }

    /// Returns the scene graph.
    #[inline]
    pub const fn scene(&self) -> &SceneGraph {
        &self.scene
    }

    /// Returns whether this package has unsupported diagnostics.
    #[inline]
    pub fn has_unsupported_features(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == AssetDiagnosticSeverity::Unsupported)
    }

    /// Returns approximate CPU memory used by package-owned asset buffers.
    pub fn memory_bytes(&self) -> usize {
        let meshes = self.meshes.values().map(geometry_bytes).sum::<usize>();
        let morphs = self
            .morph_targets
            .values()
            .flatten()
            .map(morph_bytes)
            .sum::<usize>();
        let textures = self
            .textures
            .values()
            .map(|texture| texture.data.len())
            .sum::<usize>();
        let cubes = self
            .texture_cubes
            .values()
            .flat_map(|cube| cube.faces.iter())
            .map(Vec::len)
            .sum::<usize>();
        meshes + morphs + textures + cubes + self.dependency_graph.total_bytes()
    }

    /// Adds a diagnostic and returns the package for chaining.
    #[inline]
    pub fn with_diagnostic(mut self, diagnostic: AssetDiagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }
}

/// Shared package handle returned by [`AssetManager`](crate::AssetManager).
pub type SharedAssetPackage = Arc<AssetPackage>;

/// Async load status.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetLoadStatus {
    /// Queued but not started.
    Pending,
    /// In progress with best-effort `0.0..=1.0` progress.
    Loading { progress: f32 },
    /// Finished successfully.
    Loaded,
    /// Cancelled by the caller.
    Cancelled,
    /// Failed with a scenix error.
    Failed(ScenixError),
}

pub(crate) struct AsyncAssetState {
    pub status: AssetLoadStatus,
    pub package: Option<SharedAssetPackage>,
    pub cancel_requested: bool,
}

/// Handle for background asset loading.
#[derive(Clone)]
pub struct AssetLoadHandle {
    id: AssetId,
    pub(crate) state: Arc<Mutex<AsyncAssetState>>,
}

impl AssetLoadHandle {
    pub(crate) fn new(id: AssetId, state: Arc<Mutex<AsyncAssetState>>) -> Self {
        Self { id, state }
    }

    /// Returns this load's ID.
    #[inline]
    pub const fn id(&self) -> AssetId {
        self.id
    }

    /// Requests cancellation.
    pub fn cancel(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.cancel_requested = true;
            state.status = AssetLoadStatus::Cancelled;
        }
    }

    /// Returns current status.
    pub fn status(&self) -> AssetLoadStatus {
        self.state
            .lock()
            .map(|state| state.status.clone())
            .unwrap_or(AssetLoadStatus::Failed(ScenixError::Load(
                scenix_core::LoadError::Io,
            )))
    }

    /// Returns the finished package, if loading completed successfully.
    pub fn package(&self) -> Option<SharedAssetPackage> {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.package.as_ref().map(Arc::clone))
    }
}

/// Loader support level for a file family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetFormatSupport {
    /// Fully decoded into scenix CPU data.
    Full,
    /// Metadata or a focused subset is decoded.
    Partial,
    /// Recognized but not decoded by this build.
    DiagnosticOnly,
}

/// One row in the asset format support matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetFormatInfo {
    /// Human-readable format family.
    pub name: &'static str,
    /// Common file extensions.
    pub extensions: &'static [&'static str],
    /// Support level.
    pub support: AssetFormatSupport,
}

/// v1.3 asset pipeline support matrix.
pub const ASSET_FORMATS: &[AssetFormatInfo] = &[
    AssetFormatInfo {
        name: "glTF / GLB",
        extensions: &["gltf", "glb"],
        support: AssetFormatSupport::Full,
    },
    AssetFormatInfo {
        name: "OBJ / MTL",
        extensions: &["obj"],
        support: AssetFormatSupport::Full,
    },
    AssetFormatInfo {
        name: "STL",
        extensions: &["stl"],
        support: AssetFormatSupport::Full,
    },
    AssetFormatInfo {
        name: "Images",
        extensions: &["png", "jpg", "jpeg", "webp", "tga", "tif", "tiff", "exr"],
        support: AssetFormatSupport::Full,
    },
    AssetFormatInfo {
        name: "KTX2 / DDS / HDR",
        extensions: &["ktx2", "dds", "hdr"],
        support: AssetFormatSupport::Partial,
    },
    AssetFormatInfo {
        name: "PLY / VOX / SVG / IES / LUT",
        extensions: &["ply", "vox", "svg", "ies", "cube", "3dl"],
        support: AssetFormatSupport::Partial,
    },
    AssetFormatInfo {
        name: "Collada / 3MF / VTK / LDraw / TTF",
        extensions: &["dae", "3mf", "vtk", "ldr", "dat", "ttf", "otf"],
        support: AssetFormatSupport::DiagnosticOnly,
    },
    AssetFormatInfo {
        name: "FBX / USD / USDZ / Rhino / UltraHDR",
        extensions: &["fbx", "usd", "usdz", "3dm", "uhdr"],
        support: AssetFormatSupport::DiagnosticOnly,
    },
];

/// Returns support information for a path extension.
pub fn support_for_extension(extension: &str) -> Option<AssetFormatInfo> {
    let normalized = extension.trim_start_matches('.').to_ascii_lowercase();
    ASSET_FORMATS
        .iter()
        .copied()
        .find(|info| info.extensions.iter().any(|ext| *ext == normalized))
}

fn geometry_bytes(geometry: &Geometry) -> usize {
    geometry.positions.len() * core::mem::size_of::<Vec3>()
        + geometry.normals.len() * core::mem::size_of::<Vec3>()
        + geometry.uvs.len() * core::mem::size_of::<Vec2>()
        + geometry.uvs2.len() * core::mem::size_of::<Vec2>()
        + geometry.colors.len() * core::mem::size_of::<scenix_core::Color>()
        + geometry.indices.len() * core::mem::size_of::<u32>()
        + geometry.tangents.len() * core::mem::size_of::<scenix_math::Vec4>()
}

fn morph_bytes(morph: &MorphTarget) -> usize {
    morph.positions_delta.len() * core::mem::size_of::<Vec3>()
        + morph.normals_delta.len() * core::mem::size_of::<Vec3>()
}
