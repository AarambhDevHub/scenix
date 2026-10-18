# `scenix-particles`

## Role

Optional deterministic particle systems: emitter configuration, seeded CPU
simulation, lifetime modifier modules, flipbook batches, and the GPU
billboard/compute integration points. Added in scenix v1.7.0.

## Dependency Weight

Lightweight `no_std` crate (add the `std` and/or `serde` features as needed);
depends only on `scenix-math`, `scenix-core`, and `scenix-material`. It is not
a default facade crate; enable it with the `particles` feature.

## Install

```toml
[dependencies]
scenix = { version = "1", features = ["particles"] }
# or the focused crate:
scenix-particles = "1"
```

## Key Public API

- Systems: `ParticleSystem::new(emitter, capacity, seed)`,
  `update(dt) -> &ParticleRenderBatch`, `spawn_burst(count)`,
  `set_flipbook(Some(FlipbookConfig))`, `particles()` (pool snapshot for GPU
  seeding), and `alive`/`total_spawned` counters.
- Emitters: `ParticleEmitter` builder (`with_rate`, `with_lifetime`,
  `with_size`, `with_speed`, `at`, `toward`, `with_shape`, `with_modifier`)
  and `EmissionShape` (point, `Sphere`, `Hemisphere`, `Disc`, `Cone`, `Box`).
- Modifiers: `Gravity`, `Drag`, `ColorOverLifetime`, `SizeOverLifetime`,
  `SpinOverLife`, `ConfineBox` behind the `ParticleModifier` trait.
- Curves: `ColorCurve`/`ColorKey` and `ScalarCurve`/`ScalarKey` —
  deterministic keyed falloffs reused by several modifiers.
- Batches: `Particle`, `ParticleRenderBatch`, and `ParticleRenderItem`
  (`position`, `size`, `color`, flipbook `uv_rect`, `rotation`).
- Flipbook: `FlipbookConfig` (fps/looping) mirroring
  `scenix-texture::FlipbookGrid` frame math; `PARTICLE_POOL_LIMIT` (262,144)
  is the hard pool ceiling.
- Renderer surface (feature `particles` on the renderer):
  `register_particles`, `update_particles`, `register_particle_compute`,
  `simulate_particles_compute`, `read_particle_pool`, `read_particle_ring`,
  and `unregister_particles`.

## Common Use

```rust
use scenix_particles::{
    ColorCurve, ColorOverLifetime, Drag, EmissionShape, Gravity, ParticleEmitter,
    ParticleSystem, SizeOverLifetime,
};
use scenix_math::Vec3;

let emitter = ParticleEmitter::new()
    .with_rate(240.0)
    .with_lifetime(0.9, 1.6)
    .at(Vec3::new(0.0, 0.0, 0.0))
    .toward(Vec3::new(0.0, 1.0, 0.0))
    .with_shape(EmissionShape::Cone { radius: 0.15, height: 0.35 })
    .with_modifier(Gravity::new(Vec3::new(0.0, -12.0, 0.0)))
    .with_modifier(Drag::new(0.15))
    .with_modifier(ColorOverLifetime::new(ColorCurve::from_to(
        [1.0, 0.85, 0.35, 1.0],
        [0.9, 0.15, 0.0, 0.0],
    )))
    .with_modifier(SizeOverLifetime::from_to(1.15, 0.05));

let mut system = ParticleSystem::new(emitter, 1_024, 2021018);
for _ in 0..60 {
    let batch = system.update(1.0 / 60.0);
}
assert!(system.total_spawned() >= 240);
```

With the facade, the same surface is available via `scenix::ParticleEmitter`
and friends (see the curated facade re-exports in
[ARCHITECTURE.md](../../ARCHITECTURE.md)). Renderer drawing needs the
`particles` feature on the stack: `Renderer::register_particles` +
`Renderer::update_particles` draw camera-facing billboards, and
`Renderer::register_particle_compute` moves simulation onto the GPU with
`ParticleComputeParams`.

## Determinism

Simulation is seeded. The same emitter configuration, capacity, seed, and
step count produce identical pools on every host and reset, which is why
`tests/particles_batches.rs` re-runs scenarios from scratch to verify
bookkeeping.

## GPU Compute Notes

`register_particle_compute` pushes a 64-byte-per-slot storage pool to the
GPU; `simulate_particles_compute` runs an integrate pass (gravity, drag,
size/color fades from `ParticleComputeParams`) plus a deterministic
single-invocation ring-window respawn pass — no GPU atomics. CPU updates via
`update_particles` always revert the slot to the draw path. Readbacks
(`read_particle_pool`, `read_particle_ring`) block on the device, so they are
test/editor hooks rather than a frame-loop pattern.

## Notes

Use this crate directly when you need the boundary in your own public API;
use the `scenix` facade when building an application. Supporting types land
through their own globs: `ParticleMaterial`/`ShaderKind::Particles` from
`scenix-material`, `FlipbookGrid` from `scenix-texture`, `quad_geometry`
from `scenix-mesh`, and `NodeKind::Particles` from `scenix-scene`.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
- [v1.7.0 release guide](../release-v1.7.0.md)
