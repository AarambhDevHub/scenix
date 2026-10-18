//! Particle simulation hot-path benchmarks (scenix v1.7.0).
//!
//! Runs standalone (`harness = false`):
//!
//! ```text
//! cargo bench -p scenix-particles --bench particles_bench
//! ```

use std::time::Instant;

use scenix_math::Vec3;
use scenix_particles::{
    ColorCurve, ColorOverLifetime, Drag, EmissionShape, FlipbookConfig, Gravity, ParticleEmitter,
    ParticleSystem, SizeOverLifetime, SpinOverLife,
};

/// A realistic fire/spark emitter: cone spawn, gravity, drag, and three
/// lifetime curves. Used at several pool sizes for the tick benchmark.
fn fire_emitter() -> ParticleEmitter {
    ParticleEmitter::new()
        .at(Vec3::new(0.0, 2.0, 0.0))
        .with_shape(EmissionShape::Cone {
            radius: 0.3,
            height: 0.6,
        })
        .toward(Vec3::new(0.0, 1.0, 0.0))
        .with_rate(2_000.0)
        .with_lifetime(0.4, 1.1)
        .with_speed(2.0, 5.0)
        .with_modifier(Gravity::new(Vec3::new(0.0, -9.8, 0.0)))
        .with_modifier(Drag::new(0.6))
        .with_modifier(ColorOverLifetime::new(ColorCurve::from_to(
            [1.0, 0.8, 0.5, 1.0],
            [0.7, 0.1, 0.0, 0.0],
        )))
        .with_modifier(SizeOverLifetime::from_to(1.0, 0.15))
        .with_modifier(SpinOverLife::constant_rate(0.7))
}

/// Fills a pool with eternal particles so the tick loop integrates a fixed
/// population every iteration (spawning disabled, nothing dies).
fn full_pool(count: u32) -> ParticleSystem {
    let count = count as usize;
    let emitter = fire_emitter().with_rate(0.0).with_lifetime(1.0e9, 1.0e9);
    let mut system = ParticleSystem::new(emitter, count, 7);
    assert_eq!(system.spawn_burst(count as u32), count as u32);
    system
}

fn main() {
    let flip = FlipbookConfig::full(8, 8, 48.0);
    let dt = 1.0 / 60.0;

    // Integration + batch-build cost over fully-populated pools.
    for &(count, ticks) in &[(1_000, 64_u32), (8_000, 16_u32), (32_000, 8_u32)] {
        let mut system = full_pool(count as u32);
        let start = Instant::now();
        for _ in 0..ticks {
            let batch = system.update(dt);
            assert_eq!(batch.len(), count, "eternal particles stay alive");
        }
        let elapsed = start.elapsed();
        println!(
            "tick pool {count}: {ticks} ticks in {elapsed:?} ({:?}/tick)",
            elapsed / ticks
        );
    }

    // Per-tick spawn path at a high emission rate (2k/s over one frame).
    let mut system = ParticleSystem::new(fire_emitter(), 8_000, 7);
    for _ in 0..60 {
        // Warm the pool toward its steady-state population.
        system.update(dt);
    }
    let ticks = 64_u32;
    let start = Instant::now();
    for _ in 0..ticks {
        let batch = system.update(dt);
        assert!(!batch.is_empty(), "steady-state emission never stalls");
    }
    let elapsed = start.elapsed();
    println!(
        "tick steady 2k/s: {ticks} ticks in {elapsed:?} ({:?}/tick)",
        elapsed / ticks
    );

    // Batch ownership hand-off cost at 16k particles.
    let mut system = full_pool(16_384);
    let start = Instant::now();
    for _ in 0..32 {
        let batch = system.update(dt);
        assert_eq!(batch.len(), 16_384);
    }
    let elapsed = start.elapsed();
    println!("tick + batch fold 16k: 32 passes in {elapsed:?}");

    // Flipbook UV rect evaluation (pure math, 1M evals).
    let evals = 1_048_576;
    let start = Instant::now();
    let mut sink = 0.0_f32;
    for frame in 0..evals {
        let age = (frame & 63) as f32 * 0.015_625;
        let rect = flip.uv_rect(flip.frame_at(age));
        sink += rect[0] + rect[2];
    }
    let elapsed = start.elapsed();
    // Suppress the optimizer dropping the loop through the sink.
    assert!(sink.is_finite());
    println!("flipbook uv_rect x{evals}: {elapsed:?}");
}
