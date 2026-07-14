#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side debug helper geometry for scenix.
//!
//! Helpers generate validated line segments for grids, axes, bounds, cameras,
//! lights, arrows, and simple skeletons. They do not depend on the renderer.

extern crate alloc;

pub mod animation_path;
pub mod arrow;
pub mod axes;
pub mod bounding_box;
pub mod camera_helper;
pub mod editor_helpers;
#[cfg(feature = "egui")]
pub mod egui_inspector;
pub mod gizmo;
pub mod grid;
pub mod light_helper;
pub mod line_geometry;
pub mod pose_helper;
pub mod skeleton_helper;

pub use animation_path::AnimationPathHelper;
pub use arrow::ArrowHelper;
pub use axes::AxesHelper;
pub use bounding_box::BoundingBoxHelper;
pub use camera_helper::CameraHelper;
pub use editor_helpers::{BoundsGizmoHelper, SelectionHelper, SnapGridHelper};
#[cfg(feature = "egui")]
pub use egui_inspector::{EguiInspectorResponse, show_inspector};
pub use gizmo::{GizmoGeometry, GizmoHandle, GizmoHandleId, GizmoHitShape, TransformGizmoHelper};
pub use grid::GridHelper;
pub use light_helper::{DirectionalLightHelper, PointLightHelper, SpotLightHelper};
pub use line_geometry::LineGeometry;
pub use pose_helper::PoseHelper;
pub use skeleton_helper::SkeletonHelper;

const EPSILON: f32 = 1.0e-6;
