#![cfg_attr(not(feature = "std"), no_std)]

//! GPU-free light data types for scenix.
//!
//! This crate stores CPU-side light descriptions and shadow configuration. It
//! does not allocate GPU resources and does not depend on the texture or
//! renderer crates.

pub mod ambient;
pub mod area;
pub mod directional;
pub mod hemisphere;
pub mod point;
pub mod probe;
pub mod shadow;
pub mod spot;

pub use ambient::AmbientLight;
pub use area::AreaLight;
pub use directional::DirectionalLight;
pub use hemisphere::HemisphereLight;
pub use point::PointLight;
pub use probe::LightProbe;
pub use shadow::ShadowConfig;
pub use spot::SpotLight;

pub(crate) const EPSILON: f32 = 1.0e-6;

#[inline]
pub(crate) fn positive(value: f32, fallback: f32) -> f32 {
    if value > EPSILON { value } else { fallback }
}

#[inline]
pub(crate) fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}
