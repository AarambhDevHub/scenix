#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side geometry and mesh helpers for scenix.
//!
//! This crate is renderer-agnostic. It owns triangle geometry, primitive
//! generation, morph target data, instancing metadata, and batched geometry
//! assembly, but it does not allocate GPU resources.

extern crate alloc;

pub mod batched;
pub mod buffer;
pub mod geometry;
pub mod instanced;
pub mod mesh;
pub mod morph;
pub mod primitives;
pub mod shape;

pub use batched::{BatchedGeometryRange, BatchedMesh};
pub use buffer::{
    BufferLayout, BufferStepMode, IndexFormat, VertexAttribute, VertexFormat, VertexSemantic,
};
pub use geometry::Geometry;
pub use instanced::InstancedMesh;
pub use mesh::Mesh;
pub use morph::MorphTarget;
pub use primitives::{
    box_geometry, capsule_geometry, circle_geometry, cone_geometry, cylinder_geometry,
    extrude_geometry, icosphere_geometry, lathe_geometry, plane_geometry, ring_geometry,
    shape_geometry, sphere_geometry, torus_geometry, torus_knot_geometry, tube_geometry,
};
pub use shape::Shape;

pub(crate) const EPSILON: f32 = 1.0e-6;

#[inline]
pub(crate) fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

#[inline]
pub(crate) fn sin_cos(value: f32) -> (f32, f32) {
    #[cfg(feature = "std")]
    {
        value.sin_cos()
    }
    #[cfg(not(feature = "std"))]
    {
        let sin = sin_approx(value);
        let cos = cos_approx(value);
        (sin, cos)
    }
}

#[cfg(not(feature = "std"))]
fn reduce_pi(mut value: f32) -> f32 {
    const PI: f32 = core::f32::consts::PI;
    const TAU: f32 = core::f32::consts::TAU;
    while value > PI {
        value -= TAU;
    }
    while value < -PI {
        value += TAU;
    }
    value
}

#[cfg(not(feature = "std"))]
fn sin_approx(value: f32) -> f32 {
    const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;
    const PI: f32 = core::f32::consts::PI;
    let mut x = reduce_pi(value);
    let sign = if x > FRAC_PI_2 {
        x = PI - x;
        1.0
    } else if x < -FRAC_PI_2 {
        x = -PI - x;
        1.0
    } else {
        1.0
    };
    let x2 = x * x;
    sign * x
        * (1.0 - x2 / 6.0 + (x2 * x2) / 120.0 - (x2 * x2 * x2) / 5040.0
            + (x2 * x2 * x2 * x2) / 362_880.0)
}

#[cfg(not(feature = "std"))]
fn cos_approx(value: f32) -> f32 {
    const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;
    const PI: f32 = core::f32::consts::PI;
    let mut x = reduce_pi(value);
    let mut sign = 1.0;
    if x > FRAC_PI_2 {
        x = PI - x;
        sign = -1.0;
    } else if x < -FRAC_PI_2 {
        x = -PI - x;
        sign = -1.0;
    }
    let x2 = x * x;
    sign * (1.0 - x2 / 2.0 + (x2 * x2) / 24.0 - (x2 * x2 * x2) / 720.0
        + (x2 * x2 * x2 * x2) / 40_320.0)
}
