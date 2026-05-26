#![cfg_attr(not(feature = "std"), no_std)]

//! Facade crate for scenix Renderer APIs.
//!
//! This release re-exports the Foundation crates, the GPU-free scene graph,
//! CPU-side geometry, materials, lights, textures, cameras, optional loaders,
//! optional post-processing, and optional renderer APIs.

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

#[cfg(feature = "loader")]
pub use scenix_loader::*;

#[cfg(feature = "post")]
pub use scenix_post::*;

#[cfg(feature = "renderer")]
pub use scenix_renderer::*;
