#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side BVH raycasting and scene picking for scenix.
//!
//! This crate is renderer-agnostic. Callers provide a `SceneGraph` and a mesh
//! geometry store, then raycasts return world-space intersections against mesh
//! triangles.

extern crate alloc;

pub mod bvh;
pub mod intersection;
pub mod raycaster;

pub use bvh::{Bvh, BvhEntry, BvhNode};
pub use intersection::Intersection;
pub use raycaster::{GeometryProvider, Raycaster};
