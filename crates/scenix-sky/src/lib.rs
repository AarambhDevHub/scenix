#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side procedural sky systems for scenix.
//!
//! This crate is the `v1.8.0` sky layer: a procedural Rayleigh/Mie sky
//! model, sun/-elevation light helpers backed by `scenix-light`'s
//! [`scenix_light::SunLight`], environment-map baking, and grounded skybox
//! gradients. Evaluation is **GPU-free** — the renderer `environment`
//! feature packs [`SkyModel`]/[`GroundedSkybox`] state into its WGSL sky
//! pipeline.
//!
//! # Quick start
//!
//! ```rust
//! use scenix_sky::{SkyModel, SunPosition, bake_ambient, bake_equirectangular};
//! use scenix_math::Vec3;
//!
//! let model = SkyModel::clear_day();
//! let sun = SunPosition::degrees(32.0, 46.0);
//!
//! // Evaluate the dome directly:
//! let glow = model.rgb_at(Vec3::Y, sun.direction());
//! assert!(glow[2] > glow[0]);
//!
//! // Bake for scene lighting + IBL previews:
//! let samples = bake_equirectangular(&model, &sun, 8, 8);
//! let ambient = bake_ambient(&samples, 8, 8);
//! assert!(ambient.sky.iter().sum::<f32>() > 0.0);
//! ```
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `std` (default) | Enables the std allocator path shared workspace-wide |
//! | `serde` | Serializes sky/sun/grounded-skybox descriptors |
//!
//! # Renderer and light integration
//!
//! [`SunPosition::directional_light`] turns one sun elevation into a
//! registered-able [`scenix_light::DirectionalLight`];
//! `bake_light_probe` produces an `scenix_light::LightProbe`; and the
//! renderer `environment` feature samples the same parameters through its
//! `sky_dome` WGSL pipeline. See docs/api/scenix-sky.md.

extern crate alloc;

pub mod bake;
pub mod sky;
pub mod skybox;
pub mod sun;

pub use bake::{SkyAmbient, bake_ambient, bake_equirectangular, bake_light_probe};
pub use sky::SkyModel;
pub use skybox::GroundedSkybox;
pub use sun::SunPosition;

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_math::Vec3;

    #[test]
    fn crate_surface_routes_from_model_to_light() {
        let model = SkyModel::clear_day();
        let sun = SunPosition::degrees(40.0, 20.0);
        let radiance = model.rgb_at(Vec3::Y, sun.direction());
        assert!(radiance.iter().all(|item| item.is_finite() && *item > 0.0));

        let sun_light = sun.sun_light(4.0);
        assert!(sun_light.intensity > 0.0);
        let directional = sun.directional_light(4.0);
        assert!((directional.direction.length() - 1.0).abs() < 1.0e-5);
    }
}
