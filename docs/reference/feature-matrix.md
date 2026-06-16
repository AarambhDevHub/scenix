# Feature Matrix

| Feature | Default | Use it for |
| --- | --- | --- |
| `std` | yes | Standard-library support for CPU crates. |
| `scene`, `camera`, `mesh`, `material`, `light`, `texture` | yes | CPU scene authoring. |
| `raycaster`, `helpers` | yes | Picking and debug helper geometry. |
| `loader` | no | Asset packages, asset manager, glTF extension metadata, OBJ/MTL, STL, image, KTX2, HDR/EXR loading, and exporters. |
| `renderer` | no | `wgpu` surface and headless rendering. |
| `post` | no | GPU post-processing stack; normally used with `renderer`. |
| `animato` | no | Animato bridge for scene, camera, material, and skeleton animation; Animato 1.6.0 is the release gate when published. |
| `wasm` | no | Browser canvas wrapper, DOM input mapping, WebGPU path, WebGL2 full fallback, and WebGL1 reduced fallback. |
| `serde` | no | Serialization support where each crate supports it. |


Use this matrix when deciding what to enable in the facade crate. For libraries, prefer focused crates and only expose features that are part of your own public API.
