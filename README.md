# scenix

> Modular Rust-native 3D scenes for native and WASM apps.

[![CI](https://github.com/AarambhDevHub/scenix/actions/workflows/ci.yml/badge.svg)](https://github.com/AarambhDevHub/scenix/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

scenix `1.0.0` is the first stable release. The public API is frozen around small focused crates: CPU authoring stays lightweight by default, while loading, GPU rendering, post-processing, Animato integration, and browser support remain opt-in.

## Install

Most applications start with the facade crate:

```toml
[dependencies]
scenix = "1"
```

Enable optional systems only when needed:

```toml
[dependencies]
scenix = { version = "1", features = ["loader"] }
scenix = { version = "1", features = ["renderer", "post"] }
scenix = { version = "1", features = ["animato"] }
scenix = { version = "1", features = ["wasm"] }
```

Focused crates can be used directly:

```toml
[dependencies]
scenix-math = "1"
scenix-core = "1"
scenix-input = "1"
scenix-scene = "1"
scenix-camera = "1"
scenix-mesh = "1"
scenix-material = "1"
scenix-light = "1"
scenix-texture = "1"
scenix-loader = "1"
scenix-renderer = "1"
scenix-post = "1"
scenix-raycaster = "1"
scenix-helpers = "1"
scenix-animato = "1"
scenix-wasm = "1"
```

For `no_std` CPU authoring:

```toml
[dependencies]
scenix-math = { version = "1", default-features = false, features = ["libm"] }
scenix-core = { version = "1", default-features = false }
scenix-input = { version = "1", default-features = false }
scenix-scene = { version = "1", default-features = false }
scenix-camera = { version = "1", default-features = false }
scenix-mesh = { version = "1", default-features = false }
scenix-material = { version = "1", default-features = false }
scenix-light = { version = "1", default-features = false }
scenix-texture = { version = "1", default-features = false }
scenix-raycaster = { version = "1", default-features = false }
scenix-helpers = { version = "1", default-features = false }
```

`scenix-loader`, `scenix-renderer`, `scenix-post`, `scenix-animato`, and `scenix-wasm` are optional `std` paths. The Animato bridge uses `animato = "1.4.0"`.

## Feature Flags

| Feature | Default | Description |
| --- | --- | --- |
| `std` | yes | Standard-library support for CPU crates. |
| `scene`, `camera`, `mesh`, `material`, `light`, `texture` | yes | CPU authoring crates. |
| `raycaster`, `helpers` | yes | BVH picking and debug line helper data. |
| `loader` | no | glTF/GLB, OBJ/MTL, STL, image, KTX2, HDR/EXR loading. |
| `renderer` | no | `wgpu` renderer with surface/headless targets. |
| `post` | no | Full-screen post-processing stack; use with `renderer`. |
| `animato` | no | Animato 1.4.0 tracks and scene/camera/material drivers. |
| `wasm` | no | Browser canvas wrapper and generated WebGPU demo scene. |
| `serde` | no | Serialization support where the focused crate supports it. |

## Quick Start

```rust
use std::collections::BTreeMap;
use scenix::{
    CameraId, CameraStores, Geometry, MaterialId, MeshId, NodeAnimationTarget, NodeAnimator,
    PbrMaterial, PerspectiveCamera, SceneGraph, SceneNode, ScenixAnimationDriver, Vec3,
    Vec3Track, box_geometry,
};

# fn run() -> Result<(), scenix::ValidationError> {
let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);

let mut meshes = BTreeMap::<MeshId, Geometry>::new();
meshes.insert(mesh_id, box_geometry(1.0, 1.0, 1.0, 1, 1, 1));

let mut scene = SceneGraph::new();
let cube = scene.add(SceneNode::mesh("cube", mesh_id, material_id));

let mut driver = ScenixAnimationDriver::new();
driver.add_node(NodeAnimator::new(
    cube,
    NodeAnimationTarget::Translation(Vec3Track::tween(
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
        0.5,
    )),
));

let camera_id = CameraId::new(1);
let mut perspective = BTreeMap::from([(
    camera_id,
    PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 0.0, 4.0))
        .target(Vec3::ZERO),
)]);
let mut orthographic = BTreeMap::new();
let mut cameras = CameraStores {
    perspective: &mut perspective,
    orthographic: &mut orthographic,
};
let mut materials = BTreeMap::from([(material_id, PbrMaterial::new())]);
let mut skeletons = Vec::new();

driver.tick(0.5, &mut scene, &mut cameras, &mut materials, &mut skeletons)?;
scene.update_world_transforms();
# Ok(())
# }
```

### Headless Rendering

```rust
use scenix::{Renderer, RendererConfig, PerspectiveCamera, Vec3};

# async fn run(scene: &scenix::SceneGraph) -> Result<(), scenix::ScenixError> {
let mut renderer = Renderer::headless(RendererConfig::new(256, 256)).await?;
let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 5.0))
    .target(Vec3::ZERO);
let stats = renderer.render(scene, &camera)?;
assert!(stats.frame_index > 0);
# Ok(())
# }
```

### Loading Assets

```rust
use scenix::GltfLoader;

# fn run() -> Result<(), scenix::ScenixError> {
let asset = GltfLoader::new().load_file("scene.gltf")?;
assert!(!asset.meshes.is_empty());
# Ok(())
# }
```

### Raycasting

```rust
use std::collections::BTreeMap;
use scenix::{
    Geometry, MaterialId, MeshId, PerspectiveCamera, Raycaster, SceneGraph, SceneNode,
    Vec2, Vec3, box_geometry,
};

let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);
let meshes = BTreeMap::<MeshId, Geometry>::from([(
    mesh_id,
    box_geometry(1.0, 1.0, 1.0, 1, 1, 1),
)]);

let mut scene = SceneGraph::new();
scene.add(SceneNode::mesh("cube", mesh_id, material_id));
scene.update_world_transforms();

let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
let ray = Raycaster::from_camera_ndc(&camera, Vec2::ZERO);

let mut raycaster = Raycaster::new();
raycaster.build_bvh(&scene, &meshes).unwrap();
assert!(raycaster.cast_ray(ray, &scene, &meshes).is_some());
```

## Workspace

| Crate | Role |
| --- | --- |
| `scenix` | Facade crate with stable v1 feature flags. |
| `scenix-math` | `no_std` vectors, matrices, quaternions, transforms, rays, and bounds. |
| `scenix-core` | IDs, colors, errors, and shared traits. |
| `scenix-input` | Platform-neutral pointer and keyboard state. |
| `scenix-scene` | Scene graph, transforms, traversal, fog, sprites, and LOD helpers. |
| `scenix-camera` | Perspective, orthographic, cube cameras, frustums, and controllers. |
| `scenix-mesh` | Geometry buffers, primitives, instancing, batching, and morph targets. |
| `scenix-material` | GPU-free material descriptions and pipeline keys. |
| `scenix-light` | Lights, shadow settings, and light probes. |
| `scenix-texture` | CPU textures, samplers, atlases, video updates, and mipmaps. |
| `scenix-loader` | Optional CPU asset loaders and asset cache. |
| `scenix-renderer` | Optional `wgpu` renderer and resource registries. |
| `scenix-post` | Optional `wgpu` post-processing effects. |
| `scenix-raycaster` | BVH scene picking and exact mesh intersections. |
| `scenix-helpers` | Debug `LineGeometry` generators. |
| `scenix-animato` | Optional Animato 1.4.0 bridge. |
| `scenix-wasm` | Optional browser canvas wrapper. |

## Examples

The facade crate registers the example set from [ARCHITECTURE.md](./ARCHITECTURE.md):

```sh
cargo run -p scenix --example hello_cube --features renderer
cargo run -p scenix --example pbr_sphere --features renderer
cargo run -p scenix --example physical_material --features renderer
cargo run -p scenix --example toon_shading --features renderer
cargo run -p scenix --example gltf_scene --features "loader renderer"
cargo run -p scenix --example shadow_demo --features renderer
cargo run -p scenix --example raycasting
cargo run -p scenix --example post_processing --features "renderer post"
cargo run -p scenix --example instanced_mesh
cargo run -p scenix --example animato_integration --features animato
cargo run -p scenix --example orbit_camera
cargo run -p scenix --example lod_demo
cargo run -p scenix --example morph_targets
cargo run -p scenix --example fog_demo
cargo run -p scenix --example helpers_demo
cargo run -p scenix --example sprite_particles
cargo run -p scenix --example environment_map
```

The browser example lives in `examples/wasm_viewer`:

```sh
rustup target add wasm32-unknown-unknown
cargo check --manifest-path examples/wasm_viewer/Cargo.toml --target wasm32-unknown-unknown
```

## Website

The static website is a standalone Leptos CSR app in `website/`. It is intentionally outside the main workspace so website dependencies do not affect normal library users.

```sh
cd website
trunk serve
trunk build --release --public-url /scenix/
```

GitHub Pages deployment is handled by `.github/workflows/pages.yml`, which builds `website/dist` with Trunk and deploys it at `/scenix/`. The demo uses `scenix-wasm` and falls back cleanly when WebGPU or WebAssembly is unavailable.

## Development Checks

```sh
cargo fmt --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --all-features
cargo test -p scenix-math -p scenix-core -p scenix-input -p scenix-scene -p scenix-camera -p scenix-mesh -p scenix-material -p scenix-light -p scenix-texture -p scenix-raycaster -p scenix-helpers -p scenix-animato --no-default-features
cargo test -p scenix-loader --all-features
cargo test -p scenix-raycaster -p scenix-helpers --all-features
cargo test -p scenix-animato --all-features
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
cargo check --manifest-path examples/wasm_viewer/Cargo.toml --target wasm32-unknown-unknown
SCENIX_RUN_GPU_TESTS=1 WGPU_BACKEND=vulkan cargo test -p scenix-renderer -p scenix-post --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo bench --workspace --no-run
cargo llvm-cov --workspace --all-features
```

## Documentation

- [Developer docs](./docs/README.md)
- [Architecture](./ARCHITECTURE.md)
- [Roadmap](./ROADMAP.md)
- [Changelog](./CHANGELOG.md)
- [Getting started](./docs/getting-started.md)
- [Installation](./docs/installation.md)
- [Quick start](./docs/quick-start.md)
- [Concepts](./docs/concepts/README.md)
- [Guides](./docs/guides/create-your-first-scene.md)
- [API reference](./docs/api/facade-crate.md)
- [Examples](./docs/examples/README.md)
- [Recipes](./docs/recipes/README.md)
- [Performance](./docs/performance/README.md)
- [Deployment](./docs/deployment/README.md)
- [Migration](./docs/migration/from-0.9-to-1.0.md)
- [Reference](./docs/reference/feature-matrix.md)
- [v1.0.0 release notes](./docs/release-v1.0.0.md)

## Known Limitations

- The renderer is stable for generated scenes, headless/surface rendering, material preview paths, and examples. Advanced physical shading is intentionally documented as a preview contract rather than a film-grade renderer.
- Loader APIs produce CPU-side scenix data; GPU upload stays explicit through `Renderer` registration.
- The website demo does not vendor large model assets.
- GPU tests require a Vulkan-capable device or Mesa lavapipe.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license
