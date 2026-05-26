use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use scenix_core::{LoadError, ScenixError};

/// Path-keyed asset cache that reuses decoded CPU-side assets.
#[derive(Debug)]
pub struct AssetCache<T> {
    assets: BTreeMap<PathBuf, Arc<T>>,
}

impl<T> AssetCache<T> {
    /// Creates an empty cache.
    #[inline]
    pub const fn new() -> Self {
        Self {
            assets: BTreeMap::new(),
        }
    }

    /// Returns the number of cached assets.
    #[inline]
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Returns whether the cache has no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Returns whether `path` is cached after canonical path normalization.
    pub fn contains(&self, path: impl AsRef<Path>) -> bool {
        canonical_cache_key(path.as_ref())
            .ok()
            .is_some_and(|key| self.assets.contains_key(&key))
    }

    /// Loads an asset once and returns shared handles on subsequent requests.
    pub fn get_or_load(
        &mut self,
        path: impl AsRef<Path>,
        load: impl FnOnce(&Path) -> Result<T, ScenixError>,
    ) -> Result<Arc<T>, ScenixError> {
        let key = canonical_cache_key(path.as_ref())?;
        if let Some(asset) = self.assets.get(&key) {
            return Ok(Arc::clone(asset));
        }

        let asset = Arc::new(load(&key)?);
        self.assets.insert(key, Arc::clone(&asset));
        Ok(asset)
    }

    /// Removes a cached asset if present.
    pub fn invalidate(&mut self, path: impl AsRef<Path>) -> bool {
        canonical_cache_key(path.as_ref())
            .ok()
            .and_then(|key| self.assets.remove(&key))
            .is_some()
    }

    /// Clears all cached handles.
    #[inline]
    pub fn clear(&mut self) {
        self.assets.clear();
    }
}

impl<T> Default for AssetCache<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn canonical_cache_key(path: &Path) -> Result<PathBuf, ScenixError> {
    path.canonicalize().map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            ScenixError::Load(LoadError::NotFound)
        } else {
            ScenixError::Load(LoadError::Io)
        }
    })
}
