# Scenix v1.7.0 — Particles

Version 1.7 gives effects and lightweight simulation a first-class path: a
deterministic CPU particle crate (`scenix-particles`), an instanced billboard
GPU draw path, and an optional GPU compute simulation. The CPU simulator stays
the source of truth, so authoring, headless tests, and uploads always agree;
the GPU path is a pure accelerator with an explicit parameter profile.

## Data Flow

```text
 ParticleEmitter (rate/lifetime/size/speed, shapes, modifiers)
        |
        v
 ParticleSystem (seeded pool, deterministic update)
        |
        +--> update(dt) ----------> ParticleRenderBatch (CPU path; serdeable state)
        |          |
        |          +--> Renderer::update_particles --> instanced billboards
        |          |      (64-byte instance, additive or alpha blend)
        |
        +--> spawn pool --seeds--> GPU storage pool (64 bytes per slot)
                   |                       ^
                   +--> simulate_particles_compute
                           dispatch 1: integrate (gravity, drag, fades)
                           dispatch 2: ring-window respawn (deterministic)
```

`register_particle_compute` seeds the pool once; every step afterwards is
profile-driven (`ParticleComputeParams`), and `update_particles` can always
revert a slot to the CPU-driven draw path. Readback hooks
(`read_particle_pool`, `read_particle_ring`) exist for editors and tests.

## New Public Surfaces

- `scenix-particles` (new crate): `ParticleEmitter`, `ParticleSystem`,
  `Particle`, `ParticleRenderBatch`/`ParticleRenderItem`, `EmissionShape`,
  the `ParticleModifier` modules (`Gravity`, `Drag`, `ColorOverLifetime`,
  `SizeOverLifetime`, `SpinOverLife`, `ConfineBox`),
  `ColorCurve`/`ScalarCurve`, `FlipbookConfig`, `DeterministicRng`, and
  `PARTICLE_POOL_LIMIT`.
- `scenix-material`: `ParticleMaterial`, `ParticleBlend`, the
  `FEATURE_SOFT_PARTICLES` (`1 << 20`) and `FEATURE_FLIPBOOK` (`1 << 21`)
  bits, and `ShaderKind::Particles` for pipeline keys.
- `scenix-texture`: `FlipbookGrid` — shared frame-index → UV math for
  `columns × rows` sheets (top-left origin, clamped last frame).
- `scenix-mesh`: `quad_geometry(width, height)` — the CCW +Z billboard quad
  with the scenix plane UV convention.
- `scenix-scene`: `NodeKind::Particles { system_id }`, the
  `SceneNode::particles` constructor, and the `ParticleSystemId` core
  newtype.
- `scenix-renderer` (behind `particles`): `ParticleRegistry` draw/compute
  slots, `PARTICLE_INSTANCE_STRIDE` (64B) instancing, additive and alpha
  pipelines with depth-test-on/depth-write-off blending, the compute pool
  (`PARTICLE_POOL_STRIDE`, 64B) with a 128-byte `ParticleComputeParams`
  uniform, and readback helpers.
- `scenix-wasm`: `particle_fallback_plan(WebGlCapabilityLevel)` →
  `ParticleFallbackPlan` (`Full` on WebGPU, `InstancedBillboards` on WebGL 2,
  `ExpandedQuads` on WebGL 1).
- `scenix-math`: the public `float` shim module (`sqrt`, `sin`, `cos`, `tan`,
  `asin`, `acos`, `atan2`, `cbrt`, `PI`, `FRAC_PI_2`) keeping std and `libm`
  builds API-identical.
- `scenix` facade: the `particles` feature with curated re-exports (the
  `particle` and `flipbook` module names collide with `scenix-material` and
  `scenix-texture` globs, so bulk re-exports stay explicit).

## First Render

```rust
use scenix::{Color, ParticleEmitter, ParticleMaterial, ParticleSystem, Vec3};

// 1. Author the emitter (add modifiers for acceleration, color, size, spin).
let emitter = ParticleEmitter::new()
    .with_rate(120.0)
    .with_lifetime(0.5, 1.0)
    .with_size(0.25)
    .with_speed(2.0, 4.0)
    .at(Vec3::new(0.0, 1.0, 0.0))
    .toward(Vec3::new(0.0, 1.0, 0.0));

// 2. Simulate deterministically on the CPU.
let mut system = ParticleSystem::new(emitter, 256, 42);
let batch = system.update(1.0 / 60.0);

// 3. Draw with the renderer (features: particles, renderer).
let material = ParticleMaterial::new().with_color(Color::from_rgb(1.0, 0.6, 0.2));
// renderer.register_particles(ParticleSystemId::new(1), &material)?;
// renderer.update_particles(ParticleSystemId::new(1), batch, &material)?;
// stats.particle_instances == batch.len()
```

Optional compute: `register_particle_compute(id, &material, &system, &params)`
seeds the GPU pool from the CPU snapshot, then
`simulate_particles_compute(id, &params, spawn_budget)` per frame replaces CPU
simulation.

## Migration

No v1.6 API changed. `NodeKind` gained the additive `Particles` variant, which
scene traversal and editor code already match exhaustively in their own
fallback arms. Default features are unchanged, so existing apps compile
identically. Particle textures, materials, and meshes live in their existing
crates; the `particles` feature only switches on the new systems.

## Validation

The release is covered by the `scenix-particles` suites (CPU determinism,
capacity/burst behavior, modifier composition, flipbook batches, serde) and
`benches/particles_bench.rs` hot paths, the facade `scenix_v17` suite, the
`particles_basic` example with an optional headless compute tail, and the
GPU-gated `particles_gpu` suite (pipeline draw counts plus the full compute
lifecycle to expiry/respawn/cursor-boundary on lavapipe-friendly software
platforms). CI runs the facade feature lanes
(`--no-default-features --features particles` with and without `renderer`),
no-default CPU builds, `wasm32-unknown-unknown`, rustdoc
warnings-as-errors, packaging, and benchmark compilation.

Related reading:

- [scenix-particles crate reference](api/scenix-particles.md)
- [Examples](examples/README.md)
- [GitHub release notes](../.github/release-notes/1.7.0.md)
