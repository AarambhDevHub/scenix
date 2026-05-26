//! CPU-side asset loading for scenix.
//!
//! This crate decodes common asset files into renderer-agnostic scenix data.
//! It does not create GPU buffers, bind groups, or renderer resources.

pub mod asset_cache;
pub mod gltf;
pub mod hdr;
pub mod image;
pub mod ktx2;
pub mod obj;
pub mod stl;

pub use asset_cache::AssetCache;
pub use gltf::{GltfAsset, GltfLoader, LoadedCamera, LoadedLight, LoaderOptions};
