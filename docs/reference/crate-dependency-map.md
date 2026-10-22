# Crate Dependency Map

Scenix keeps dependency direction layered and explicit.

```text
scenix facade
  -> CPU authoring crates: math, core, input, scene, camera, mesh, material, light, texture, raycaster, helpers
  -> optional environment crates: terrain (heightmaps, chunks, splats, collision, streaming),
     sky (procedural atmosphere, sun positions, bakes), water (waves, foam, fresnel, underwater)
  -> optional runtime bridges: xr (session plans, controllers, hands, anchors, planes, hit tests),
     audio (spatial mixing, Doppler, streams, analysis), physics (rigid bodies, colliders,
     character controller, transform sync)
  -> optional tooling crates: ui (renderer-agnostic panels, stats, widgets, overlays),
     editor (owning sessions, exact undo/redo, gizmo math, inspector surface)
  -> optional nodes (shader graphs, node materials, post node effects)
  -> optional particles (CPU particle systems, GPU billboard/compute path)
  -> optional loader
  -> optional renderer
  -> optional post
  -> optional animato
  -> optional wasm
```

## Practical Rules

- CPU crates should not depend on GPU crates.
- `scenix-nodes` is a CPU crate: it generates shader text and never touches
  `wgpu`. The renderer, post stack, and wasm preflight opt into node features
  individually.
- `scenix-nodes` depends on `scenix-material` for pipeline keys but the
  renderer depends on `scenix-nodes` only through its optional `nodes` feature.
- `scenix-particles` is a CPU crate: it simulates deterministically on the
  host and never touches `wgpu` directly. The renderer opts in through its
  optional `particles` feature, which owns the buffer/instance translation
  (64-byte draw instances and pool records).
- `scenix-particles` depends on `scenix-math` and `scenix-core` only; scene,
  material, texture, and mesh support come from the shared dynasties
  (`NodeKind::Particles`, `ParticleMaterial`, `FlipbookGrid`,
  `quad_geometry`) rather than direct dependencies.
- `scenix-terrain`, `scenix-sky`, and `scenix-water` are CPU crates: the
  environment crates are GPU-free (terrain on core/math/mesh, sky on
  core/light/math, water on camera/core/math/mesh) and never touch `wgpu`.
  The renderer opts in through its optional `environment` feature, which owns
  the slot registry, WGSL pipelines, and uniform belt; material descriptors
  and schema ids stay in the shared dynasties (`TerrainMaterial`, `SkyId`).
- `scenix-xr`, `scenix-audio`, and `scenix-physics` are CPU crates: the
  runtime bridges are GPU-free (xr on core/input/math/camera, audio on
  core/math, physics on core/math/mesh) and never touch `wgpu`. The facade
  exposes them through the `xr`/`audio`/`physics` features; schema ids and
  `NodeKind::{XrAnchor, AudioSource, PhysicsBody}` stay in the shared
  dynasties, `XrCameraRig`/`XrEyeProjection` stay in `scenix-camera`, and
  the renderer's culling pass treats bridge payloads as CPU-side data that
  submits no draws.
- `scenix-ui` and `scenix-editor` are tooling crates, not runtime crates:
  `scenix-ui` stays `no_std`-capable (math only + optional serde) with a
  fixed stats window and bounded overlay caps, and `scenix-editor` is the
  one std-only crate (it wraps the graph plus the UI panel stack and drives
  optional loader/material/animato via weak feature forwards). Neither
  renders; neither touches `wgpu`; both sit above the engine crates so hosts
  can build faces (egui, HTML tools, native windows) on shared exact
  semantics.
- `scenix-post` stays independent of `scenix-renderer`; renderer optionally integrates post support.
- `scenix-loader` outputs CPU data and does not upload to GPU.
- `scenix-wasm` is browser-oriented and optional.
