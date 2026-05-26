#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side debug helper geometry for scenix.
//!
//! Helpers generate validated line segments for grids, axes, bounds, cameras,
//! lights, arrows, and simple skeletons. They do not depend on the renderer.

extern crate alloc;

pub mod arrow;
pub mod axes;
pub mod bounding_box;
pub mod camera_helper;
pub mod grid;
pub mod light_helper;
pub mod line_geometry;
pub mod skeleton_helper;

pub use arrow::ArrowHelper;
pub use axes::AxesHelper;
pub use bounding_box::BoundingBoxHelper;
pub use camera_helper::CameraHelper;
pub use grid::GridHelper;
pub use light_helper::{DirectionalLightHelper, PointLightHelper, SpotLightHelper};
pub use line_geometry::LineGeometry;
pub use skeleton_helper::SkeletonHelper;

const EPSILON: f32 = 1.0e-6;
