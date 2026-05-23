#![cfg_attr(not(feature = "std"), no_std)]

//! Facade crate for scenix Textures and Camera APIs.
//!
//! This release re-exports the Foundation crates, the GPU-free scene graph,
//! CPU-side geometry, materials, lights, textures, and cameras.

pub use scenix_core::*;
pub use scenix_input::*;
pub use scenix_math::*;

#[cfg(feature = "scene")]
pub use scenix_scene::*;

#[cfg(feature = "camera")]
pub use scenix_camera::*;

#[cfg(feature = "mesh")]
pub use scenix_mesh::*;

#[cfg(feature = "material")]
pub use scenix_material::*;

#[cfg(feature = "light")]
pub use scenix_light::*;

#[cfg(feature = "texture")]
pub use scenix_texture::*;
