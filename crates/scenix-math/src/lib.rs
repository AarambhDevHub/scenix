#![cfg_attr(not(feature = "std"), no_std)]

//! Custom scalar `f32` math primitives for scenix.
//!
//! The crate is intentionally dependency-light and can be used without `std`.
//! Enable the `libm` feature when targeting `no_std` platforms that need
//! portable trigonometric functions.

pub mod bounds;
pub mod cylindrical;
pub mod euler;
pub mod mat;
pub mod plane;
pub mod quat;
pub mod ray;
pub mod spherical;
pub mod transform;
pub mod vec;

pub use bounds::{Aabb, Sphere};
pub use cylindrical::Cylindrical;
pub use euler::{Euler, RotationOrder};
pub use mat::{Mat3, Mat4};
pub use plane::Plane;
pub use quat::Quat;
pub use ray::Ray3;
pub use spherical::Spherical;
pub use transform::Transform;
pub use vec::{Vec2, Vec3, Vec4};

pub(crate) const EPSILON: f32 = 1.0e-6;

#[inline]
pub(crate) fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Portable `f32` intrinsics (`sqrt`, trig, inverse trig, cube root) with
/// `libm`/`std`/fallback selection. Preferred public path for scenix crates
/// that must stay `no_std`-friendly.
pub mod float;

// Legacy crate-internal paths (e.g. `use crate::{cos, sin}`) keep resolving.
pub(crate) use self::float::{acos, asin, atan2, cos, sin, sqrt, tan};

#[cfg(test)]
pub(crate) fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-4, "{a} != {b}");
}
