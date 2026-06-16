use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use scenix_core::{AssetId, LoadError, MaterialId, MeshId, ScenixError, TextureId};
use scenix_material::PbrMaterial;
use scenix_scene::{SceneGraph, SceneNode};
use scenix_texture::Sampler;

use crate::asset::{
    AssetDiagnostic, AssetFormatSupport, AssetLoadHandle, AssetLoadStatus, AssetPackage,
    AssetSource, AsyncAssetState, SharedAssetPackage, support_for_extension,
};
use crate::{GltfLoader, hdr, image, ktx2, obj, stl};

/// v1.3 asset manager with cache, background loading, diagnostics, and memory accounting.
#[derive(Default)]
pub struct AssetManager {
    next_id: u64,
    cache: BTreeMap<AssetSource, SharedAssetPackage>,
    memory_budget_bytes: Option<usize>,
}

impl AssetManager {
    /// Creates an empty asset manager.
    #[inline]
    pub const fn new() -> Self {
        Self {
            next_id: 1,
            cache: BTreeMap::new(),
            memory_budget_bytes: None,
        }
    }

    /// Returns number of cached packages.
    #[inline]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns whether no packages are cached.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Sets or clears the approximate CPU memory budget.
    #[inline]
    pub fn set_memory_budget_bytes(&mut self, budget: Option<usize>) {
        self.memory_budget_bytes = budget;
        self.enforce_memory_budget();
    }

    /// Returns approximate cached CPU memory usage.
    #[inline]
    pub fn memory_bytes(&self) -> usize {
        self.cache
            .values()
            .map(|package| package.memory_bytes())
            .sum()
    }

    /// Loads a local file, reusing a fresh cached package when possible.
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<SharedAssetPackage, ScenixError> {
        let path = path.as_ref();
        let key = AssetSource::Path(path.to_path_buf());
        if let Some(package) = self.cache.get(&key)
            && !package.dependency_graph.is_stale()
        {
            return Ok(Arc::clone(package));
        }

        let id = self.next_asset_id();
        let package = Arc::new(self.load_file_uncached(id, path)?);
        self.cache.insert(key, Arc::clone(&package));
        self.enforce_memory_budget();
        Ok(package)
    }

    /// Loads bytes using the label extension to choose a decoder.
    pub fn load_bytes(
        &mut self,
        label: impl Into<String>,
        bytes: &[u8],
    ) -> Result<SharedAssetPackage, ScenixError> {
        let label = label.into();
        let key = AssetSource::Bytes {
            label: label.clone(),
            len: bytes.len(),
        };
        if let Some(package) = self.cache.get(&key) {
            return Ok(Arc::clone(package));
        }
        let id = self.next_asset_id();
        let mut package = self.load_bytes_uncached(id, &label, bytes)?;
        package.source = Some(key.clone());
        package.dependency_graph.record_bytes(&label, bytes.len());
        let package = Arc::new(package);
        self.cache.insert(key, Arc::clone(&package));
        self.enforce_memory_budget();
        Ok(package)
    }

    /// Loads a glTF/GLB URL behind the `http` feature.
    #[cfg(feature = "http")]
    pub async fn load_url(&mut self, url: &str) -> Result<SharedAssetPackage, ScenixError> {
        let key = AssetSource::Url(url.to_owned());
        if let Some(package) = self.cache.get(&key) {
            return Ok(Arc::clone(package));
        }
        let mut package = GltfLoader::new().load_package_url(url).await?;
        package.id = self.next_asset_id();
        package.source = Some(key.clone());
        package.dependency_graph.record_url(url, 0);
        let package = Arc::new(package);
        self.cache.insert(key, Arc::clone(&package));
        self.enforce_memory_budget();
        Ok(package)
    }

    /// Starts background loading of a local file on a standard thread.
    pub fn load_file_async(&mut self, path: impl Into<PathBuf>) -> AssetLoadHandle {
        let id = self.next_asset_id();
        let path = path.into();
        let state = Arc::new(Mutex::new(AsyncAssetState {
            status: AssetLoadStatus::Pending,
            package: None,
            cancel_requested: false,
        }));
        let thread_state = Arc::clone(&state);
        thread::spawn(move || {
            if let Ok(mut state) = thread_state.lock() {
                if state.cancel_requested {
                    state.status = AssetLoadStatus::Cancelled;
                    return;
                }
                state.status = AssetLoadStatus::Loading { progress: 0.1 };
            }

            let result = load_file_for_thread(id, &path).map(Arc::new);
            if let Ok(mut state) = thread_state.lock() {
                if state.cancel_requested {
                    state.status = AssetLoadStatus::Cancelled;
                    return;
                }
                match result {
                    Ok(package) => {
                        state.package = Some(package);
                        state.status = AssetLoadStatus::Loaded;
                    }
                    Err(err) => state.status = AssetLoadStatus::Failed(err),
                }
            }
        });
        AssetLoadHandle::new(id, state)
    }

    /// Invalidates a cached path or byte/URL source.
    pub fn invalidate(&mut self, source: &AssetSource) -> bool {
        self.cache.remove(source).is_some()
    }

    /// Invalidates local packages whose dependencies changed on disk.
    pub fn invalidate_stale(&mut self) -> usize {
        let stale: Vec<_> = self
            .cache
            .iter()
            .filter_map(|(source, package)| {
                package
                    .dependency_graph
                    .is_stale()
                    .then_some(source.clone())
            })
            .collect();
        let count = stale.len();
        for source in stale {
            self.cache.remove(&source);
        }
        count
    }

    /// Clears all cached packages.
    #[inline]
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    fn next_asset_id(&mut self) -> AssetId {
        let id = AssetId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    fn load_file_uncached(&self, id: AssetId, path: &Path) -> Result<AssetPackage, ScenixError> {
        let extension = extension(path)?;
        let mut package = match extension.as_str() {
            "gltf" | "glb" => GltfLoader::new().load_package_file(path)?,
            "obj" => package_from_obj(id, path)?,
            "stl" => package_from_stl(id, path)?,
            "ktx2" => package_from_texture(id, path, ktx2::load(path)?)?,
            "hdr" => package_from_cube(id, path, hdr::load(path)?)?,
            "png" | "jpg" | "jpeg" | "webp" | "tga" | "tif" | "tiff" | "exr" => {
                package_from_texture(id, path, image::load(path)?)?
            }
            other => diagnostic_package_for_extension(id, path, other)?,
        };
        package.id = id;
        package.source = Some(AssetSource::Path(path.to_path_buf()));
        package.dependency_graph.record_path(path);
        Ok(package)
    }

    fn load_bytes_uncached(
        &self,
        id: AssetId,
        label: &str,
        bytes: &[u8],
    ) -> Result<AssetPackage, ScenixError> {
        let extension = extension(Path::new(label))?;
        match extension.as_str() {
            "gltf" | "glb" => GltfLoader::new().load_package_bytes(bytes, None),
            "ktx2" => package_from_texture_label(id, label, ktx2::load_bytes(bytes)?),
            "png" | "jpg" | "jpeg" | "webp" | "tga" | "tif" | "tiff" | "exr" => {
                package_from_texture_label(id, label, image::load_bytes(bytes)?)
            }
            other => {
                let mut package = AssetPackage::empty(id, label);
                package
                    .diagnostics
                    .push(unsupported_format_diagnostic(other));
                Ok(package)
            }
        }
    }

    fn enforce_memory_budget(&mut self) {
        let Some(budget) = self.memory_budget_bytes else {
            return;
        };
        while self.memory_bytes() > budget {
            let Some(source) = self.cache.keys().next().cloned() else {
                break;
            };
            self.cache.remove(&source);
        }
    }
}

fn load_file_for_thread(id: AssetId, path: &Path) -> Result<AssetPackage, ScenixError> {
    AssetManager {
        next_id: id.get(),
        cache: BTreeMap::new(),
        memory_budget_bytes: None,
    }
    .load_file_uncached(id, path)
}

fn extension(path: &Path) -> Result<String, ScenixError> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .ok_or(ScenixError::Load(LoadError::UnsupportedFormat))
}

fn package_from_obj(id: AssetId, path: &Path) -> Result<AssetPackage, ScenixError> {
    let asset = obj::load_with_materials(path)?;
    let mut package = AssetPackage::empty(id, path_label(path));
    let mut scene = SceneGraph::new();
    let default_material = MaterialId::new(1);
    package
        .materials
        .insert(default_material, PbrMaterial::new().named("obj-default"));
    for (index, material) in asset.materials.iter().enumerate() {
        let id = MaterialId::new(index as u64 + 1);
        package
            .materials
            .entry(id)
            .or_insert_with(|| PbrMaterial::new().named(material.name.clone()));
    }
    for (index, geometry) in asset.geometries.into_iter().enumerate() {
        let mesh_id = MeshId::new(index as u64 + 1);
        let material_id = asset
            .geometry_materials
            .get(index)
            .and_then(|material| material.map(|value| MaterialId::new(value as u64 + 1)))
            .unwrap_or(default_material);
        package.meshes.insert(mesh_id, geometry);
        scene.add(SceneNode::mesh(
            format!("obj.mesh-{index}"),
            mesh_id,
            material_id,
        ));
    }
    scene.update_world_transforms();
    package.scene = scene;
    for (id, material) in &package.materials {
        package
            .loaded_materials
            .insert(*id, crate::LoadedMaterial::Pbr(material.clone()));
    }
    Ok(package)
}

fn package_from_stl(id: AssetId, path: &Path) -> Result<AssetPackage, ScenixError> {
    let geometry = stl::load(path)?;
    let mut package = AssetPackage::empty(id, path_label(path));
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    package.meshes.insert(mesh_id, geometry);
    package
        .materials
        .insert(material_id, PbrMaterial::new().named("stl-default"));
    package.loaded_materials.insert(
        material_id,
        crate::LoadedMaterial::Pbr(package.materials[&material_id].clone()),
    );
    package
        .scene
        .add(SceneNode::mesh("stl.mesh", mesh_id, material_id));
    package.scene.update_world_transforms();
    Ok(package)
}

fn package_from_texture(
    id: AssetId,
    path: &Path,
    texture: scenix_texture::Texture2D,
) -> Result<AssetPackage, ScenixError> {
    package_from_texture_label(id, &path_label(path), texture)
}

fn package_from_texture_label(
    id: AssetId,
    label: &str,
    texture: scenix_texture::Texture2D,
) -> Result<AssetPackage, ScenixError> {
    let mut package = AssetPackage::empty(id, label);
    package.textures.insert(TextureId::new(1), texture);
    package.samplers.insert(TextureId::new(1), Sampler::new());
    Ok(package)
}

fn package_from_cube(
    id: AssetId,
    path: &Path,
    texture: scenix_texture::TextureCube,
) -> Result<AssetPackage, ScenixError> {
    let mut package = AssetPackage::empty(id, path_label(path));
    package.texture_cubes.insert(TextureId::new(1), texture);
    package.samplers.insert(TextureId::new(1), Sampler::new());
    Ok(package)
}

fn diagnostic_package_for_extension(
    id: AssetId,
    path: &Path,
    extension: &str,
) -> Result<AssetPackage, ScenixError> {
    if let Some(info) = support_for_extension(extension) {
        let mut package = AssetPackage::empty(id, path_label(path));
        let diagnostic = match info.support {
            AssetFormatSupport::Full => AssetDiagnostic::warning(
                "asset.loader.unrouted",
                format!("{} is recognized but not routed by this build", info.name),
            ),
            AssetFormatSupport::Partial => AssetDiagnostic::warning(
                "asset.loader.partial",
                format!("{} has partial v1.3 metadata support", info.name),
            ),
            AssetFormatSupport::DiagnosticOnly => AssetDiagnostic::unsupported(
                "asset.loader.diagnostic_only",
                format!(
                    "{} is recognized but requires an external converter",
                    info.name
                ),
            ),
        };
        package.diagnostics.push(diagnostic);
        Ok(package)
    } else {
        Err(ScenixError::Load(LoadError::UnsupportedFormat))
    }
}

fn unsupported_format_diagnostic(extension: &str) -> AssetDiagnostic {
    AssetDiagnostic::unsupported(
        "asset.loader.unsupported_format",
        format!("no v1.3 byte loader is available for .{extension}"),
    )
}

fn path_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset")
        .to_owned()
}
