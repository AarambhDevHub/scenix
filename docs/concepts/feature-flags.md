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
| `editor` | no | Owning editor sessions with exact inverse-command undo/redo, coalesced gizmo drags, scene-tree rows, the node inspector surface, and material field editing (the `scenix-editor` crate; implies `ui`). Asset-browser rows need `loader`, timeline previews need `animato`, and the JSON project exchange needs `serde`. |
| `egui` | no | Read-only egui adapter for inspector snapshots. |
| `ui` | no | Renderer-agnostic UI tooling: docked panel stacks with the space solver, rolling renderer stats with budget policies, tool widgets with hit testing, and bounded overlay composition (the `scenix-ui` crate). |
| `loader` | no | glTF/GLB, OBJ/MTL, STL, image, KTX2, HDR/EXR loading. |
| `renderer` | no | `wgpu` surface and headless rendering. |
| `post` | no | GPU post-processing stack; normally used with `renderer`. |
| `nodes` | no | Typed shader graphs, node materials, and post node effects (the `scenix-nodes` crate) with WGSL/GLSL backends; forwards to `renderer`, `post`, and `wasm` variants when enabled. |
| `particles` | no | Deterministic CPU particle systems (the `scenix-particles` crate) with emitter/curve modules, flipbooks, and billboards; forwards to the `renderer` particle draw/compute path and `wasm` fallback plans when enabled. |
| `terrain` | no | CPU heightfields, chunked LOD, splat maps, collision, and streaming (the `scenix-terrain` crate). |
| `sky` | no | Procedural sky profiles, sun positions, baked environment maps, and grounded skyboxes (the `scenix-sky` crate). |
| `water` | no | Gerstner waves, foam, fresnel, underwater tinting, and reflection helpers (the `scenix-water` crate). |
| `environment` | no | Enables all three environment crates and forwards to the renderer's `environment` GPU pipeline gate. |
| `xr` | no | Session plans, controller bays, hand frames, anchors, planes, hit tests, and light estimation (the `scenix-xr` crate). |
| `audio` | no | Spatial mixing with Doppler, streamed buffer health, and spectrum analysis (the `scenix-audio` crate). |
| `physics` | no | Deterministic rigid-body solver, sensors, spatial queries, sleeping, and the character controller (the `scenix-physics` crate). |
| `bridges` | no | Enables `xr`, `audio`, and `physics` together. |

| `animato` | no | Animato 1.7 bridge and clip-based animation runtime. |
| `wasm` | no | Browser canvas wrapper, DPR-aware input mapping, WebGPU path, WebGL2 full fallback, and WebGL1 reduced fallback. |
| `serde` | no | Serialization support where each crate supports it. |


## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
