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
| `serde` | no | Serialization support where each crate supports it. |


Use this matrix when deciding what to enable in the facade crate. For libraries, prefer focused crates and only expose features that are part of your own public API.
