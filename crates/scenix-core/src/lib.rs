#![cfg_attr(not(feature = "std"), no_std)]

//! Shared Foundation types for scenix.

#[cfg(feature = "inspector")]
extern crate alloc;

pub mod color;
pub mod error;
pub mod ids;
#[cfg(feature = "inspector")]
pub mod inspector;
pub mod traits;

pub use color::{Color, ColorSpace};
pub use error::{GpuError, LoadError, ScenixError, ValidationError};
pub use ids::{
    AnimationClipId, AssetId, CameraId, LightId, MaterialId, MeshId, NodeId, SkinId, TextureId,
};
#[cfg(feature = "inspector")]
pub use inspector::{
    Inspectable, InspectorField, InspectorId, InspectorItem, InspectorSnapshot, InspectorValue,
};
pub use traits::{Bounded, Renderable};

#[cfg(feature = "gpu")]
pub use traits::GpuUpload;
#[cfg(feature = "std")]
pub use traits::Named;
