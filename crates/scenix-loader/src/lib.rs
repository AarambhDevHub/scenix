//! CPU-side asset loading for scenix.
//!
//! This crate decodes common asset files into renderer-agnostic scenix data.
//! It does not create GPU buffers, bind groups, or renderer resources.

pub mod asset;
pub mod asset_cache;
pub mod asset_manager;
pub mod export;
pub mod gltf;
pub mod hdr;
pub mod image;
pub mod ktx2;
pub mod obj;
pub mod stl;

pub use asset::{
    ASSET_FORMATS, AssetDependency, AssetDependencyGraph, AssetDiagnostic, AssetDiagnosticSeverity,
    AssetFormatInfo, AssetFormatSupport, AssetLoadHandle, AssetLoadStatus, AssetPackage,
    AssetRequest, AssetSource, LoadedAnimationChannel, LoadedAnimationClip,
    LoadedAnimationInterpolation, LoadedAnimationProperty, LoadedMaterial,
    LoadedMeshSkinAttributes, LoadedSkin, MaterialVariant, SharedAssetPackage, TextureTransform,
    support_for_extension,
};
pub use asset_cache::AssetCache;
pub use asset_manager::AssetManager;
pub use gltf::{GltfAsset, GltfLoader, LoadedCamera, LoadedLight, LoaderOptions};
