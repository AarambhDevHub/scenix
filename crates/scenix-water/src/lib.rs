#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side water systems for scenix.
//!
//! This crate is the `v1.8.0` water layer: Gerstner wave fields, deformable
//! surface geometry, Schlick fresnel reflectance, crest foam masks,
//! underwater view tinting, and planar reflection helpers. It is
//! **GPU-free** — the renderer `environment` feature packs wave/tint state
//! into its `water_surface` WGSL pipeline and samples registered
//! reflection/refraction render targets.
//!
//! # Quick start
//!
//! ```rust
//! use scenix_water::{ReflectionSetup, WaveField, WaterSurface, mirror_transform, reflected_camera};
//! use scenix_camera::PerspectiveCamera;
//! use scenix_math::Vec3;
//!
//! let surface = WaterSurface::new(WaveField::ocean_shore())
//!     .with_config(scenix_water::WaterConfig::new(64.0, 64.0, 32, 0.0));
//!
//! // CPU height preview (physics/collision reuse):
//! let crest = surface.height_at(2.0, -3.0, 0.75);
//! assert!(crest.is_finite());
//!
//! // Reflection pass plumbing (renderer feature samples these targets):
//! let setup = ReflectionSetup::minimal(11);
//! let mirror = mirror_transform(surface.config.plane_height);
//! let camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 400.0)
//!     .position(Vec3::new(0.0, 8.0, 14.0))
//!     .target(Vec3::new(0.0, 2.0, 0.0));
//! let mirrored = reflected_camera(&camera, surface.config.plane_height);
//! assert!((mirrored.position.y + camera.position.y).abs() < 1.0e-5);
//! ```
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `std` (default) | Enables the std allocator path shared workspace-wide |
//! | `serde` | Serializes wave fields, foam, underwater views, and surfaces |
//!
//! # Renderer integration
//!
//! `register_water_surface` (renderer `environment` feature) consumes a
//! [`WaterSurface`]: wave parameters, tint/fresnel/foam state, and an
//! optional reflection/refraction target pair. See docs/api/scenix-water.md.

extern crate alloc;

pub mod foam;
pub mod fresnel;
pub mod reflection;
pub mod surface;
pub mod underwater;
pub mod waves;

pub use foam::FoamParams;
pub use fresnel::FresnelParams;
pub use reflection::{ReflectionSetup, mirror_transform, reflected_camera};
pub use surface::{WaterConfig, WaterSurface};
pub use underwater::UnderwaterView;
pub use waves::{GerstnerWave, MAX_WAVES, WaveField, WaveSample};

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_math::Vec2;

    #[test]
    fn crate_surface_routes_from_waves_to_foam() {
        let surface = WaterSurface::new(WaveField::ocean_shore());
        let sample = surface.waves.evaluate(Vec2::new(1.0, 2.0), 0.4);
        let coverage = surface.foam.mask(sample.crest);
        assert!((0.0..=1.0).contains(&coverage));
        assert_eq!(MAX_WAVES, 4);
    }
}
