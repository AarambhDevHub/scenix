# Feature Matrix

| Feature | Default | Use it for |
| --- | --- | --- |
| `std` | yes | Standard-library support for CPU crates. |
| `scene`, `camera`, `mesh`, `material`, `light`, `texture` | yes | CPU scene authoring. |
| `raycaster`, `helpers` | yes | Picking and debug helper geometry. |
| `interaction` | no | Controls, selection, dragging, transforms, and gizmos. |
| `editor` | no | Typed inspector snapshots for editor-facing systems. |
| `egui` | no | Read-only egui adapter for inspector snapshots. |
| `loader` | no | Asset packages, asset manager, glTF extension metadata, OBJ/MTL, STL, image, KTX2, HDR/EXR loading, and exporters. |
| `renderer` | no | `wgpu` surface and headless rendering. |
| `post` | no | GPU post-processing stack; normally used with `renderer`. |
| `animato` | no | Animato 1.7 bridge and clip-based animation runtime. |
| `wasm` | no | Browser canvas wrapper, DPR-aware input mapping, WebGPU path, WebGL2 full fallback, and WebGL1 reduced fallback. |
| `terrain` | no | CPU heightfields, chunked LOD, splat maps, collision, and streaming (the `scenix-terrain` crate). |
| `sky` | no | Procedural sky profiles, sun positions, baked environment maps, and grounded skyboxes (the `scenix-sky` crate). |
| `water` | no | Gerstner waves, foam, fresnel, underwater tinting, and reflection helpers (the `scenix-water` crate). |
| `environment` | no | Enables all three environment crates and forwards to the renderer's `environment` GPU pipeline gate. |
| `xr` | no | Session plans, controller bays, hand frames, anchors, planes, hit tests, and light estimation (the `scenix-xr` crate). |
| `audio` | no | Spatial mixing with Doppler, streamed buffer health, and spectrum analysis (the `scenix-audio` crate). |
| `physics` | no | Deterministic rigid-body solver, sensors, spatial queries, sleeping, and the character controller (the `scenix-physics` crate). |
| `bridges` | no | Enables `xr`, `audio`, and `physics` together. |

| `serde` | no | Serialization support where each crate supports it. |


Use this matrix when deciding what to enable in the facade crate. For libraries, prefer focused crates and only expose features that are part of your own public API.
