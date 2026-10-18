#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side particle systems for scenix.
//!
//! This crate is the `v1.7.0` particle layer: emitters, keyframed curves,
//! stateless modifier modules, a deterministic CPU simulator, flipbook
//! atlas support, and the reusable render batch the renderer uploads per
//! frame. It contains **no GPU code** — drawing is `scenix-renderer`'s
//! job behind its `particles` feature (batched upload by default, an
//! optional compute simulation, and a documented WebGL/CPU fallback).
//!
//! # Quick start
//!
//! ```rust
//! use scenix_particles::{
//!     Drag, FlipbookConfig, Gravity, ParticleEmitter, ParticleSystem, SizeOverLifetime,
//! };
//! use scenix_math::Vec3;
//!
//! let emitter = ParticleEmitter::new()
//!     .with_rate(240.0)
//!     .with_lifetime(0.4, 1.2)
//!     .with_modifier(Gravity::new(Vec3::new(0.0, -9.8, 0.0)))
//!     .with_modifier(Drag::new(1.5))
//!     .with_modifier(SizeOverLifetime::from_to(1.0, 0.0));
//!
//! let mut system = ParticleSystem::new(emitter, 4096, 42);
//! system.set_flipbook(Some(FlipbookConfig::full(4, 4, 24.0)));
//!
//! // Per tick: simulate and take the reusable render batch.
//! let batch = system.update(1.0 / 60.0);
//! assert!(!batch.is_empty());
//! ```
//!
//! # Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `std` (default) | Enables the std allocator path shared workspace-wide |
//! | `serde` | Serializes the emitter description and batch data |
//!
//! # Renderer integration
//!
//! The CPU simulator emits [`ParticleRenderBatch`]es of world-space
//! quads. `scenix-renderer` (behind the `particles` feature) registers a
//! system by id, batches instanced-billboard draws in one call per system,
//! and — on devices with compute — can run the same emitter through a GPU
//! `ParticleSimMode::GpuCompute` path. See
//! docs/api/scenix-particles.md for the full guide.

extern crate alloc;

pub mod curve;
pub mod emitter;
pub mod flipbook;
pub mod modules;
pub mod particle;
pub mod rng;
pub mod system;

pub use curve::{ColorCurve, ColorKey, ScalarCurve, ScalarKey};
pub use emitter::{EmissionShape, ParticleEmitter};
pub use flipbook::FlipbookConfig;
pub use modules::{
    ColorOverLifetime, ConfineBox, Drag, Gravity, ParticleModifier, SizeOverLifetime, SpinOverLife,
};
pub use particle::Particle;
pub use rng::DeterministicRng;
pub use system::{PARTICLE_POOL_LIMIT, ParticleRenderBatch, ParticleRenderItem, ParticleSystem};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_surface_example_matches_the_simulation() {
        let emitter = ParticleEmitter::new()
            .with_rate(240.0)
            .with_lifetime(0.4, 1.2)
            .with_modifier(ConfineBox::elastic(scenix_math::Vec3::ONE));
        let mut system = ParticleSystem::new(emitter, 4096, 42);
        let batch = system.update(1.0 / 60.0);
        assert!(!batch.is_empty());
        assert_eq!(system.capacity(), 4096);
    }
}
