# Crate Dependency Map

Scenix keeps dependency direction layered and explicit.

```text
scenix facade
  -> CPU authoring crates: math, core, input, scene, camera, mesh, material, light, texture, raycaster, helpers
  -> optional nodes (shader graphs, node materials, post node effects)
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
- `scenix-post` stays independent of `scenix-renderer`; renderer optionally integrates post support.
- `scenix-loader` outputs CPU data and does not upload to GPU.
- `scenix-wasm` is browser-oriented and optional.
