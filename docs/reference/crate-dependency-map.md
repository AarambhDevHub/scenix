# Crate Dependency Map

Scenix keeps dependency direction layered and explicit.

```text
scenix facade
  -> CPU authoring crates: math, core, input, scene, camera, mesh, material, light, texture, raycaster, helpers
  -> optional loader
  -> optional renderer
  -> optional post
  -> optional animato
  -> optional wasm
```

## Practical Rules

- CPU crates should not depend on GPU crates.
- `scenix-post` stays independent of `scenix-renderer`; renderer optionally integrates post support.
- `scenix-loader` outputs CPU data and does not upload to GPU.
- `scenix-wasm` is browser-oriented and optional.
