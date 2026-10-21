# Scenix Developer Documentation

Scenix v1.9.0 is a modular Rust-native 3D scene workspace. These docs are
written for application developers who need to choose crates, create scenes,
add interaction/editor primitives, render or load assets, integrate Animato,
build terrain, sky, and water environments, bridge XR, audio, and physics
runtime systems, target WASM, and debug performance.

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
- [Interaction and editor primitives](concepts/interaction-and-editor.md)
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
- [v1.7.0 release notes](../.github/release-notes/1.7.0.md)
- [v1.8.0 release notes — Terrain, Sky, And Water](release-v1.8.0.md)
- [v1.9.0 release notes — XR, Audio, And Physics Bridges](release-v1.9.0.md)

## Feature Flags At A Glance

| Feature | Default | Use it for |
| --- | --- | --- |
| `std` | yes | Standard-library support for CPU crates. |
| `scene`, `camera`, `mesh`, `material`, `light`, `texture` | yes | CPU scene authoring. |
| `raycaster`, `helpers` | yes | Picking and debug helper geometry. |
| `interaction` | no | Controls, selection, dragging, transforms, and gizmos. |
| `editor` | no | Typed inspector snapshots for editor-facing systems. |
| `egui` | no | Read-only egui adapter for inspector snapshots. |
| `loader` | no | Asset packages, asset manager, glTF extension metadata, importers, and exporters. |
| `renderer` | no | `wgpu` surface and headless rendering. |
| `post` | no | GPU post-processing stack; normally used with `renderer`. |
| `nodes` | no | Typed shader graphs, node materials, and post node effects (the `scenix-nodes` crate) with WGSL/GLSL backends. |
| `particles` | no | Deterministic CPU particle systems (the `scenix-particles` crate) with the billboard GPU path and optional compute simulation. |
| `terrain` | no | CPU heightfields, chunked LOD, splat maps, collision, and streaming (the `scenix-terrain` crate). |
| `sky` | no | Procedural sky profiles, sun positions, baked environment maps, and grounded skyboxes (the `scenix-sky` crate). |
| `water` | no | Gerstner waves, foam, fresnel, underwater tinting, and reflection helpers (the `scenix-water` crate). |
| `environment` | no | Enables all three environment crates and forwards to the renderer's `environment` GPU pipeline gate. |
| `xr` | no | Session plans, controller bays, hand frames, anchors, planes, hit tests, and light estimation (the `scenix-xr` crate). |
| `audio` | no | Spatial mixing with Doppler, streamed buffer health, and spectrum analysis (the `scenix-audio` crate). |
| `physics` | no | Deterministic rigid-body solver, sensors, spatial queries, sleeping, and the character controller (the `scenix-physics` crate). |
| `bridges` | no | Enables `xr`, `audio`, and `physics` together. |

| `animato` | no | Animato 1.7 bridge and clip-based animation runtime. |
| `wasm` | no | Browser canvas wrapper, DOM input mapping, WebGPU path, WebGL2 full fallback, and WebGL1 reduced fallback. |
| `serde` | no | Serialization support where each crate supports it. |


## Verification Commands

```sh
cargo fmt --check
cargo test --workspace --all-features
cargo test -p scenix --test scenix_v15 --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```
