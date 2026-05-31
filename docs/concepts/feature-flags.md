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
| `loader` | no | glTF/GLB, OBJ/MTL, STL, image, KTX2, HDR/EXR loading. |
| `renderer` | no | `wgpu` surface and headless rendering. |
| `post` | no | GPU post-processing stack; normally used with `renderer`. |
| `animato` | no | Animato 1.4.0 scene, camera, material, and skeleton animation bridge. |
| `wasm` | no | Browser canvas wrapper, DOM input mapping, WebGPU path, and WebGL fallback. |
| `serde` | no | Serialization support where each crate supports it. |


## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
