# Feature Flags

## Purpose

Choose the smallest dependency surface for each app or library.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

All facade features are documented here.

## Key Rules

- Default is CPU authoring plus raycaster/helpers.
- Heavy systems are opt-in.
- Forward `serde` only when serialization is required.


## Example

| Feature | Default | Use it for |
| --- | --- | --- |
| `std` | yes | Standard-library support for CPU crates. |
| `scene`, `camera`, `mesh`, `material`, `light`, `texture` | yes | CPU scene authoring. |
| `raycaster`, `helpers` | yes | Picking and debug helper geometry. |
| `interaction` | no | Controls, selection, dragging, transforms, and gizmos. |
| `editor` | no | Typed inspector snapshots for editor-facing systems. |
| `egui` | no | Read-only egui adapter for inspector snapshots. |
| `loader` | no | glTF/GLB, OBJ/MTL, STL, image, KTX2, HDR/EXR loading. |
| `renderer` | no | `wgpu` surface and headless rendering. |
| `post` | no | GPU post-processing stack; normally used with `renderer`. |
| `nodes` | no | Typed shader graphs, node materials, and post node effects (the `scenix-nodes` crate) with WGSL/GLSL backends; forwards to `renderer`, `post`, and `wasm` variants when enabled. |
| `particles` | no | Deterministic CPU particle systems (the `scenix-particles` crate) with emitter/curve modules, flipbooks, and billboards; forwards to the `renderer` particle draw/compute path and `wasm` fallback plans when enabled. |
| `terrain` | no | CPU heightfields, chunked LOD, splat maps, collision, and streaming (the `scenix-terrain` crate). |
| `sky` | no | Procedural sky profiles, sun positions, baked environment maps, and grounded skyboxes (the `scenix-sky` crate). |
| `water` | no | Gerstner waves, foam, fresnel, underwater tinting, and reflection helpers (the `scenix-water` crate). |
| `environment` | no | Enables all three environment crates and forwards to the renderer's `environment` GPU pipeline gate. |

| `animato` | no | Animato 1.7 bridge and clip-based animation runtime. |
| `wasm` | no | Browser canvas wrapper, DPR-aware input mapping, WebGPU path, WebGL2 full fallback, and WebGL1 reduced fallback. |
| `serde` | no | Serialization support where each crate supports it. |


## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
