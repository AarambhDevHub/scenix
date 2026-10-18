//! The CPU particle simulator and its render batch output.
//!
//! [`ParticleSystem`] owns a compact pool of [`Particle`]s, applies the
//! emitter rate and bursts, integrates modifier modules, and folds the
//! survivors into a reusable [`ParticleRenderBatch`]. Simulation is fully
//! deterministic for a given seed and tick sequence.

use alloc::vec::Vec;

use scenix_math::Vec3;

use crate::emitter::ParticleEmitter;
use crate::flipbook::FlipbookConfig;
use crate::particle::Particle;
use crate::rng::DeterministicRng;

/// Hard ceiling for a single particle system's pool.
///
/// The sim runs on the CPU inside the render loop; beyond this floor
/// authors should shard across systems.
pub const PARTICLE_POOL_LIMIT: usize = 262_144;

/// One render-ready particle folded out of the simulator.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParticleRenderItem {
    /// World-space position.
    pub position: [f32; 3],
    /// World-space half-extent of the billboard.
    pub size: f32,
    /// Current tint color, linear rgba.
    pub color: [f32; 4],
    /// Flipbook uv rect `[u0, v0, u1, v1]`; `[0, 0, 1, 1]` without a grid.
    pub uv_rect: [f32; 4],
    /// Billboard rotation in radians.
    pub rotation: f32,
}

/// Reusable render output of one particle system.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParticleRenderBatch {
    /// Render items in pool order (stable across frames when nothing dies).
    pub items: Vec<ParticleRenderItem>,
}

impl ParticleRenderBatch {
    /// Returns the number of particles in this batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the batch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clears the batch while keeping the allocation.
    #[inline]
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// Deterministic CPU particle system.
///
/// The system couples one emitter with a fixed pool capacity. `update`
/// advances the simulation and returns the reusable render batch for the
/// frame; bursts add immediate particles on top of the rate counter.
#[derive(Debug)]
pub struct ParticleSystem {
    emitter: ParticleEmitter,
    capacity: usize,
    particles: Vec<Particle>,
    rate_accumulator: f32,
    elapsed: f32,
    total_spawned: u64,
    rng: DeterministicRng,
    flipbook: Option<FlipbookConfig>,
    batch: ParticleRenderBatch,
    max_alive: usize,
}

impl ParticleSystem {
    /// Creates a system with the given pool capacity and seed.
    ///
    /// Capacities above [`PARTICLE_POOL_LIMIT`] clamp to the limit.
    pub fn new(emitter: ParticleEmitter, capacity: usize, seed: u64) -> Self {
        let capacity = capacity.min(PARTICLE_POOL_LIMIT);
        Self {
            emitter,
            capacity,
            particles: Vec::with_capacity(capacity.min(1024)),
            rate_accumulator: 0.0,
            elapsed: 0.0,
            total_spawned: 0,
            rng: DeterministicRng::new(seed),
            flipbook: None,
            batch: ParticleRenderBatch::default(),
            max_alive: 0,
        }
    }

    /// Returns the system emitter.
    #[inline]
    pub fn emitter(&self) -> &ParticleEmitter {
        &self.emitter
    }

    /// Returns a mutable reference to the emitter.
    #[inline]
    pub fn emitter_mut(&mut self) -> &mut ParticleEmitter {
        &mut self.emitter
    }

    /// Replaces the emitter entirely.
    #[inline]
    pub fn set_emitter(&mut self, emitter: ParticleEmitter) {
        self.emitter = emitter;
    }

    /// Sets the flipbook grid used when emitting uv rects.
    #[inline]
    pub fn set_flipbook(&mut self, config: Option<FlipbookConfig>) {
        self.flipbook = config;
    }

    /// Returns the configured flipbook, if any.
    #[inline]
    pub fn flipbook(&self) -> Option<&FlipbookConfig> {
        self.flipbook.as_ref()
    }

    /// Returns the pool capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of alive particles.
    #[inline]
    pub fn alive(&self) -> usize {
        self.particles.len()
    }

    /// Returns the peak alive count since creation.
    #[inline]
    pub fn max_alive(&self) -> usize {
        self.max_alive
    }

    /// Returns the total number of particles spawned since creation.
    #[inline]
    pub fn total_spawned(&self) -> u64 {
        self.total_spawned
    }

    /// Returns the simulated seconds since creation.
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Spawns `count` particles immediately (throttled only by capacity).
    pub fn spawn_burst(&mut self, count: u32) -> u32 {
        let mut spawned = 0_u32;
        for _ in 0..count {
            if self.particles.len() >= self.capacity {
                break;
            }
            self.push_particle();
            spawned += 1;
        }
        spawned
    }

    /// Clears all alive particles and the batch (allocations are kept).
    pub fn clear(&mut self) {
        self.particles.clear();
        self.batch.clear();
    }

    /// Advances the simulation by `dt` seconds and returns the render batch.
    ///
    /// Falls back for paused frames (`dt <= 0`) and returns the previous
    /// batch untouched so paused systems keep rendering their last state.
    pub fn update(&mut self, dt: f32) -> &ParticleRenderBatch {
        if dt > 0.0 {
            self.elapsed += dt;
            self.spawn_from_rate(dt);
            self.integrate(dt);
            self.fold_batch();
        }
        &self.batch
    }

    /// Returns the last render batch without advancing the simulation.
    #[inline]
    pub fn render_batch(&self) -> &ParticleRenderBatch {
        &self.batch
    }

    /// Read-only view over the live simulation pool.
    ///
    /// Exists so the renderer's GPU-compute path (scenix v1.7.0) can seed
    /// its 48-byte pool layout directly from CPU simulation state without
    /// exposing the pool for mutation.
    #[inline]
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Moves the system pool into the emitter's world space such that the
    /// emitter origin sits at `origin` (convenience for scene-attached
    /// systems that moved this frame).
    #[inline]
    pub fn set_origin(&mut self, origin: Vec3) {
        self.emitter.origin = origin;
    }

    fn push_particle(&mut self) {
        let particle = self.emitter.spawn(&mut self.rng);
        self.particles.push(particle);
        self.total_spawned += 1;
        if self.particles.len() > self.max_alive {
            self.max_alive = self.particles.len();
        }
    }

    fn spawn_from_rate(&mut self, dt: f32) {
        let rate = self.emitter.rate;
        if rate <= 0.0 {
            return;
        }
        self.rate_accumulator += rate * dt;
        let spawn_count = self.rate_accumulator as usize;
        self.rate_accumulator -= spawn_count as f32;
        for _ in 0..spawn_count {
            if self.particles.len() >= self.capacity {
                self.rate_accumulator = 0.0;
                break;
            }
            self.push_particle();
        }
    }

    fn integrate(&mut self, dt: f32) {
        let modifiers = &*self.emitter.modifiers;
        let mut write_index = 0_usize;
        for read_index in 0..self.particles.len() {
            let mut particle = self.particles[read_index];
            particle.age += dt;
            if particle.is_dead() {
                continue; // compacted away
            }
            let t = particle.life_fraction();
            for modifier in modifiers {
                modifier.update(&mut particle, dt, t);
            }
            self.particles[write_index] = particle;
            write_index += 1;
        }
        self.particles.truncate(write_index);
    }

    fn fold_batch(&mut self) {
        self.batch.items.clear();
        if self.particles.is_empty() {
            return;
        }
        self.batch.items.reserve(self.particles.len());
        for particle in &self.particles {
            let uv_rect = match &self.flipbook {
                Some(grid) => grid.uv_rect(grid.frame_at(particle.age)),
                None => [0.0, 0.0, 1.0, 1.0],
            };
            self.batch.items.push(ParticleRenderItem {
                position: particle.position.to_array(),
                size: particle.size,
                color: particle.color,
                uv_rect,
                rotation: particle.rotation,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::ColorCurve;
    use crate::modules::{ColorOverLifetime, Drag, SizeOverLifetime};
    use scenix_core::Color;

    fn fountain() -> ParticleEmitter {
        ParticleEmitter::new()
            .with_color(Color::from_rgb(0.2, 0.7, 1.0))
            .with_size(0.1)
            .with_rate(10.0)
            .with_lifetime(1.0, 1.0)
            .with_speed(1.0, 2.0)
    }

    fn geyser() -> ParticleEmitter {
        // High rate so short ticks still emit.
        ParticleEmitter::new()
            .with_color(Color::from_rgb(0.2, 0.7, 1.0))
            .with_size(0.1)
            .with_rate(240.0)
            .with_lifetime(1.0, 1.0)
            .with_speed(1.0, 2.0)
    }

    #[test]
    fn spawn_rate_accumulates_deterministically() {
        let mut system = ParticleSystem::new(geyser(), 256, 42);
        let mut reference = ParticleSystem::new(geyser(), 256, 42);
        let batches = system.update(0.05).clone(); // 240/s * 0.05 = 12 spawns
        let reference_batches = reference.update(0.05).clone();
        assert_eq!(batches.items, reference_batches.items);
        assert_eq!(batches.items.len(), 12);
    }

    #[test]
    fn particles_die_at_their_lifetime() {
        let mut system = ParticleSystem::new(fountain(), 256, 42);
        system.update(0.25);
        for _ in 0..40 {
            system.update(0.25);
        }
        // 10/s with 1s lifetime → the alive population must stabilize.
        assert!(system.alive() <= 12);
        assert!(system.total_spawned() > 50);
    }

    #[test]
    fn capacity_is_respected_and_bursts_fill_pool() {
        let mut system = ParticleSystem::new(fountain(), 64, 7);
        let spawned = system.spawn_burst(1_000);
        assert_eq!(spawned, 64);
        assert_eq!(system.alive(), 64);
        assert_eq!(system.max_alive(), 64);
        // further rate spawns are refused while full
        system.update(1.0);
        assert!(system.alive() <= 64);
    }

    #[test]
    fn update_returns_batch_with_valid_render_items() {
        let mut system = ParticleSystem::new(fountain(), 256, 42);
        let batch = system.update(0.1);
        for item in &batch.items {
            assert!(item.size > 0.0);
            assert!(item.color.iter().all(|c| c.is_finite()));
            assert_eq!(item.uv_rect, [0.0, 0.0, 1.0, 1.0], "no flipbook yet");
            assert!(item.position.iter().all(|p| p.is_finite()));
        }
    }

    #[test]
    fn flipbook_emits_uv_rects_per_age() {
        let mut system = ParticleSystem::new(fountain(), 256, 42);
        system.set_flipbook(Some(FlipbookConfig::full(2, 2, 4.0)));
        let batch = system.update(0.25);
        for item in &batch.items {
            assert!(item.uv_rect[0] >= 0.0 && item.uv_rect[2] <= 1.0);
        }
    }

    #[test]
    fn modifiers_act_on_alive_pool_only() {
        let emitter = fountain()
            .with_rate(0.0)
            .with_modifier(SizeOverLifetime::from_to(1.0, 0.0))
            .with_modifier(ColorOverLifetime::new(ColorCurve::default()))
            .with_modifier(Drag::new(4.0));
        let mut system = ParticleSystem::new(emitter, 256, 42);
        system.spawn_burst(8);
        let batch = system.update(0.99).clone();
        assert_eq!(batch.len(), 8, "lifetime 1.0 not reached yet");
        for item in &batch.items {
            assert!(item.size < 0.2, "shrunk toward zero");
        }
    }

    #[test]
    fn paused_frames_keep_the_batch() {
        let mut system = ParticleSystem::new(fountain(), 256, 42);
        let before = system.update(0.1).len();
        let after = system.update(0.0);
        assert_eq!(after.len(), before);
        assert_eq!(system.elapsed(), 0.1);
    }

    #[test]
    fn same_seed_same_frames() {
        let mut a = ParticleSystem::new(fountain(), 512, 123);
        let mut b = ParticleSystem::new(fountain(), 512, 123);
        for _ in 0..120 {
            assert_eq!(a.update(1.0 / 60.0).items, b.update(1.0 / 60.0).items);
        }
        assert_eq!(a.alive(), b.alive());
        assert_eq!(a.total_spawned(), b.total_spawned());
    }
}
