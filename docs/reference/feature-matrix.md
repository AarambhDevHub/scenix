# Feature Matrix

| Feature | Default | Use it for |
| --- | --- | --- |
| `std` | yes | Standard-library support for CPU crates. |
| `scene`, `camera`, `mesh`, `material`, `light`, `texture` | yes | CPU scene authoring. |
| `raycaster`, `helpers` | yes | Picking and debug helper geometry. |
| `loader` | no | glTF/GLB, OBJ/MTL, STL, image, KTX2, HDR/EXR loading. |
| `renderer` | no | `wgpu` surface and headless rendering. |
| `post` | no | GPU post-processing stack; normally used with `renderer`. |
| `animato` | no | Animato 1.4.0 scene, camera, material, and skeleton animation bridge. |
| `wasm` | no | Browser canvas wrapper and DOM input mapping. |
| `serde` | no | Serialization support where each crate supports it. |


Use this matrix when deciding what to enable in the facade crate. For libraries, prefer focused crates and only expose features that are part of your own public API.
