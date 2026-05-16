#![cfg_attr(not(feature = "std"), no_std)]

//! GPU-free scene graph types for scenix.
//!
//! This crate owns scene node hierarchy, local transforms, cached world
//! transforms, traversal, fog settings, sprites, and LOD helpers. It does not
//! depend on renderer, mesh, material, loader, or platform crates.

extern crate alloc;

pub mod fog;
pub mod graph;
pub mod iter;
pub mod lod;
pub mod node;
pub mod sprite;

pub use fog::Fog;
pub use graph::SceneGraph;
pub use iter::{BreadthFirstIter, DepthFirstIter};
pub use lod::LodGroup;
pub use node::{NodeKind, SceneNode};
pub use sprite::{BillboardMode, Sprite};
