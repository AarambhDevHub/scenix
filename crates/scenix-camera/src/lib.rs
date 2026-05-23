#![cfg_attr(not(feature = "std"), no_std)]

//! GPU-free camera, frustum, and controller types for scenix.

pub mod controller;
pub mod cube_camera;
pub mod frustum;
pub mod orthographic;
pub mod perspective;

pub use controller::{FlyController, OrbitController};
pub use cube_camera::{CubeCamera, CubeFace};
pub use frustum::{Frustum, Visibility};
pub use orthographic::OrthographicCamera;
pub use perspective::PerspectiveCamera;

pub(crate) const EPSILON: f32 = 1.0e-6;

#[inline]
pub(crate) fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

#[inline]
pub(crate) fn sanitize_near_far(near: f32, far: f32) -> (f32, f32) {
    let near = if near > EPSILON { near } else { 0.1 };
    let far = if far > near { far } else { near + 1000.0 };
    (near, far)
}
