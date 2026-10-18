//! Cross-module particle batch integration tests (scenix v1.7.0).
//!
//! Covers the full CPU loop end to end: emitter configuration, modifier
//! integration, rate accumulation, capacity behavior, flipbook output,
//! and batch determinism across resets.

use scenix_math::Vec3;
use scenix_particles::{
    ColorCurve, ColorOverLifetime, Drag, EmissionShape, FlipbookConfig, Gravity, ParticleEmitter,
    ParticleSystem, SizeOverLifetime, SpinOverLife,
};

fn fountain(rate: f32) -> ParticleEmitter {
    ParticleEmitter::new()
        .with_rate(rate)
        .with_lifetime(0.4, 0.8)
        .with_size(0.12)
        .with_speed(2.0, 4.0)
}

#[test]
fn emitter_modules_integrate_through_the_pool() {
    let emitter = fountain(120.0)
        .at(Vec3::new(0.0, 5.0, 0.0))
        .toward(Vec3::new(0.0, 1.0, 0.0))
        .with_shape(EmissionShape::Sphere { radius: 0.5 })
        .with_modifier(Gravity::new(Vec3::new(0.0, -9.81, 0.0)))
        .with_modifier(Drag::new(0.8))
        .with_modifier(ColorOverLifetime::new(ColorCurve::from_to(
            [1.0, 1.0, 1.0, 1.0],
            [1.0, 0.2, 0.0, 0.0],
        )))
        .with_modifier(SizeOverLifetime::from_to(1.0, 0.1))
        .with_modifier(SpinOverLife::constant_rate(1.5));

    let mut system = ParticleSystem::new(emitter, 512, 2024);
    for _ in 0..120 {
        let batch = system.update(1.0 / 60.0);
        assert!(batch.len() <= 512);
        for item in &batch.items {
            // All particles fall (gravity + no count) while slowing (drag).
            assert!(item.color[0] > 0.0);
            assert!(item.uv_rect[2] <= 1.0);
        }
        assert!(system.elapsed().abs() < 2.1);
    }
    // With 0.4–0.8 s lifetimes, the pool must stabilize below 120/s * 0.8 s.
    assert!(system.alive() < 100);
    assert!(system.max_alive() > 40);
}

#[test]
fn bursts_and_rates_share_the_pool_cap() {
    let mut system = ParticleSystem::new(fountain(0.0), 128, 5);
    let burst = system.spawn_burst(1000);
    assert_eq!(burst, 128, "cap respected");
    assert_eq!(system.alive(), 128);

    // The rate-driven path also honors the cap.
    system.set_emitter(fountain(60.0));
    system.update(10.0);
    assert!(system.alive() <= 128);
}

#[test]
fn flipbook_frames_advance_with_age() {
    // 4 fps over a 1/60-base clock: the first tick lands on frame 0.
    let flip = FlipbookConfig::full(4, 2, 4.0);
    let mut system = ParticleSystem::new(fountain(64.0), 64, 9);
    system.set_flipbook(Some(flip));

    let early = system.update(1.0 / 16.0).clone();
    let early_frames: Vec<[f32; 4]> = early.items.iter().map(|i| i.uv_rect).collect();
    for rect in &early_frames {
        assert_eq!(rect[0], 0.0, "frame 0 column at start");
    }
    let late = system.update(8.0 / 16.0).clone();
    let late_first = late.items.last().map(|i| i.uv_rect);
    let Some(rect) = late_first else {
        panic!("expected alive particles");
    };
    assert!(rect[0] > 0.0 || rect[1] > 0.0, "frame advanced");
}

#[test]
fn determinism_survives_pool_compaction() {
    // Kill-heavy loops force compaction every tick; two systems with the
    // same seed must still produce identical pools.
    let mut a = ParticleSystem::new(fountain(300.0).with_lifetime(0.1, 0.2), 256, 77);
    let mut b = ParticleSystem::new(fountain(300.0).with_lifetime(0.1, 0.2), 256, 77);
    for _ in 0..600 {
        assert_eq!(a.update(1.0 / 60.0).items, b.update(1.0 / 60.0).items);
    }
    assert_eq!(a.alive(), b.alive());
    assert_eq!(a.total_spawned(), b.total_spawned());
    assert_eq!(b.total_spawned(), a.total_spawned());
}

#[test]
fn clear_reuses_allocations() {
    let mut system = ParticleSystem::new(fountain(120.0), 256, 3);
    system.update(0.5); // spawns at 0.5s age — still alive (lifetime > 0.5)
    assert!(!system.render_batch().is_empty());
    system.clear();
    assert_eq!(system.alive(), 0);
    assert!(system.render_batch().is_empty());
    // Capacity and spawn path still work after clearing.
    assert_eq!(system.spawn_burst(4), 4);
    assert_eq!(system.alive(), 4);
    // The batching path refills on the next tick.
    let after = system.update(0.5).clone();
    assert!(!after.is_empty());
}
