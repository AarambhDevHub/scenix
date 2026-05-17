#![cfg_attr(not(feature = "std"), no_std)]

//! Facade crate for scenix Geometry APIs.
//!
//! This release re-exports the Foundation crates, the GPU-free scene graph, and
//! with the default `mesh` feature, CPU-side geometry and mesh primitives.

pub use scenix_core::*;
pub use scenix_input::*;
pub use scenix_math::*;

#[cfg(feature = "scene")]
pub use scenix_scene::*;

#[cfg(feature = "mesh")]
pub use scenix_mesh::*;
