#![cfg_attr(not(feature = "std"), no_std)]

//! Facade crate for scenix Scene Graph APIs.
//!
//! This release re-exports the Foundation crates and, with the default `scene`
//! feature, the GPU-free scene graph.

pub use scenix_core::*;
pub use scenix_input::*;
pub use scenix_math::*;

#[cfg(feature = "scene")]
pub use scenix_scene::*;
