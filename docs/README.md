# Scenix Developer Documentation

Scenix v1.0.0 is a modular Rust-native 3D scene workspace. These docs are written for application developers who need to choose crates, create scenes, render or load assets, integrate Animato, target WASM, and debug performance.

## Start Here

- [Getting started](getting-started.md) - the shortest path from install to a scene.
- [Installation](installation.md) - facade, selected crates, optional features, and `no_std` setup.
- [Quick start](quick-start.md) - copyable scene, camera, raycast, and renderer snippets.
- [Project setup](project-setup.md) - recommended app layout and check commands.

## Concepts

- [Concepts overview](concepts/README.md)
- [Architecture overview](concepts/architecture-overview.md)
- [Scene graph](concepts/scene-graph.md)
- [Transforms](concepts/transforms.md)
- [Cameras](concepts/cameras.md)
- [Meshes and geometry](concepts/meshes-and-geometry.md)
- [Materials](concepts/materials.md)
- [Lights](concepts/lights.md)
- [Textures](concepts/textures.md)
- [Renderer](concepts/renderer.md)
- [Post-processing](concepts/post-processing.md)
- [Raycasting](concepts/raycasting.md)
- [Helpers](concepts/helpers.md)
- [Animation with Animato](concepts/animation-with-animato.md)
- [WASM and browser](concepts/wasm-and-browser.md)
- [Feature flags](concepts/feature-flags.md)
- [no_std](concepts/no-std.md)
- [Error handling](concepts/error-handling.md)

## Guides

- [Create your first scene](guides/create-your-first-scene.md)
- [Render a cube](guides/render-a-cube.md)
- [Use orbit camera](guides/use-orbit-camera.md)
- [Load a glTF model](guides/load-gltf-model.md)
- [Create a PBR material](guides/create-pbr-material.md)
- [Add lights and shadows](guides/add-lights-and-shadows.md)
- [Use post-processing](guides/use-post-processing.md)
- [Pick objects with raycaster](guides/pick-objects-with-raycaster.md)
- [Animate scene with Animato](guides/animate-scene-with-animato.md)
- [Build for WASM](guides/build-for-wasm.md)
- [Deploy to GitHub Pages](guides/deploy-to-github-pages.md)
- [Optimize large scenes](guides/optimize-large-scenes.md)
- [Use only selected crates](guides/use-only-selected-crates.md)

## Reference Sections

- [API by crate](api/facade-crate.md)
- [Examples](examples/README.md)
- [Recipes](recipes/README.md)
- [Performance](performance/README.md)
- [Deployment](deployment/README.md)
- [Migration](migration/from-0.9-to-1.0.md)
- [Reference](reference/feature-matrix.md)
- [v1.0.0 release notes](release-v1.0.0.md)

## Feature Flags At A Glance

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


## Verification Commands

```sh
cargo fmt --check
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```
