# scenix

> Italian: scenix - scene, the stage on which everything appears.

[![CI](https://github.com/AarambhDevHub/scenix/actions/workflows/ci.yml/badge.svg)](https://github.com/AarambhDevHub/scenix/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

scenix v0.9.0 is the Integration release. It adds optional Animato 1.4.0 animation bridges and a browser/WASM renderer wrapper while keeping the default facade focused on CPU authoring, raycasting, and helper geometry.

This release ships:

- `scenix-math`: vectors, matrices, quaternions, transforms, rays, bounds, planes, and coordinate helpers.
- `scenix-core`: typed IDs, color, errors, and shared traits.
- `scenix-input`: pointer and keyboard state.
- `scenix-scene`: scene node hierarchy, transform propagation, traversal, fog, sprites, and LOD helpers.
- `scenix-camera`: perspective, orthographic, and cube cameras, frustums, screen rays, orbit controls, and fly controls.
- `scenix-mesh`: geometry buffers, primitive generation, morph targets, instancing, and batching helpers.
- `scenix-material`: GPU-free material traits, pipeline keys, PBR, physical, toon, line, point, and custom shader materials.
- `scenix-light`: GPU-free light types, shadow configuration, and raw-sample spherical-harmonics light probes.
- `scenix-texture`: raw CPU texture data, samplers, atlases, video-frame updates, and RGBA8 mipmap generation.
- `scenix-loader`: optional glTF/GLB, OBJ/MTL, STL, PNG/JPEG/WebP, KTX2, HDR/EXR, and path-cache loaders that output CPU-side scenix data.
- `scenix-renderer`: optional `wgpu` renderer with headless/surface targets, renderer-owned registries, G-buffer/shadow targets, culling/sorting helpers, and pipeline caching.
- `scenix-post`: optional `wgpu` full-screen post stack with bloom, SSAO, tonemap, FXAA, TAA, SMAA, depth of field, fog, outline, and motion blur passes.
- `scenix-raycaster`: BVH-accelerated CPU picking with exact mesh intersections and camera-ray helpers.
- `scenix-helpers`: CPU debug `LineGeometry` for grids, axes, bounds, arrows, lights, cameras, and skeletons.
- `scenix-animato`: optional Animato 1.4.0 bridge for node, camera, PBR material, and skeleton pose animation.
- `scenix-wasm`: optional browser canvas renderer wrapper, DOM input mapping helpers, and generated-scene setup.
- `scenix`: facade crate re-exporting CPU APIs by default, loader APIs behind `features = ["loader"]`, GPU APIs behind `features = ["renderer", "post"]`, and integration APIs behind `features = ["animato", "wasm"]`.

## Installation

Most users should start with the facade crate:

```toml
[dependencies]
scenix = "0.9"
```

Enable CPU asset loading:

```toml
[dependencies]
scenix = { version = "0.9", features = ["loader"] }
```

Enable GPU rendering and post-processing:

```toml
[dependencies]
scenix = { version = "0.9", features = ["renderer", "post"] }
```

Enable Animato or browser integration:

```toml
[dependencies]
scenix = { version = "0.9", features = ["animato"] }
scenix-wasm = "0.9"
```

Use focused crates directly when you only need one layer:

```toml
[dependencies]
scenix-math = "0.9"
scenix-core = "0.9"
scenix-input = "0.9"
scenix-scene = "0.9"
scenix-camera = "0.9"
scenix-mesh = "0.9"
scenix-material = "0.9"
scenix-light = "0.9"
scenix-texture = "0.9"
scenix-loader = "0.9"
scenix-renderer = "0.9"
scenix-post = "0.9"
scenix-raycaster = "0.9"
scenix-helpers = "0.9"
scenix-animato = "0.9"
scenix-wasm = "0.9"
```

For `no_std`-capable CPU crates:

```toml
[dependencies]
scenix-math = { version = "0.9", default-features = false, features = ["libm"] }
scenix-core = { version = "0.9", default-features = false }
scenix-input = { version = "0.9", default-features = false }
scenix-scene = { version = "0.9", default-features = false }
scenix-camera = { version = "0.9", default-features = false }
scenix-mesh = { version = "0.9", default-features = false }
scenix-material = { version = "0.9", default-features = false }
scenix-light = { version = "0.9", default-features = false }
scenix-texture = { version = "0.9", default-features = false }
scenix-raycaster = { version = "0.9", default-features = false }
scenix-helpers = { version = "0.9", default-features = false }
```

`scenix-loader`, `scenix-renderer`, `scenix-post`, and `scenix-wasm` are `std` crates. `scenix-animato` is optional and bridges the `animato = "1.4.0"` facade crate.

## Quick Start

### Load glTF And Render Headless

```rust
use scenix::{GltfLoader, PerspectiveCamera, Renderer, RendererConfig, Vec3};

# async fn run() -> Result<(), scenix::ScenixError> {
let asset = GltfLoader::new().load_file("path/to/scene.gltf")?;
let mut renderer = Renderer::headless(RendererConfig::new(256, 256)).await?;

for (mesh_id, geometry) in &asset.meshes {
    renderer.register_mesh(*mesh_id, geometry)?;
}
for (material_id, material) in &asset.materials {
    renderer.register_pbr_material(*material_id, material)?;
}

let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
let stats = renderer.render(&asset.scene, &camera)?;

assert!(stats.visible_meshes > 0);
# Ok(())
# }
```

### Post-Processing Stack

```rust
use scenix::{BloomConfig, FxaaConfig, PostStack, Renderer, RendererConfig, ToneMapper};

# async fn run() -> Result<(), scenix::ScenixError> {
let mut renderer = Renderer::headless(RendererConfig::new(256, 256)).await?;
renderer.set_post_stack(Some(
    PostStack::new()
        .with_bloom(BloomConfig::default())
        .with_tonemap(ToneMapper::Aces)
        .with_fxaa(FxaaConfig::default()),
));
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
let mut meshes = BTreeMap::<MeshId, Geometry>::new();
meshes.insert(mesh_id, box_geometry(1.0, 1.0, 1.0, 1, 1, 1));

let mut scene = SceneGraph::new();
scene.add(SceneNode::mesh("cube", mesh_id, material_id));
scene.update_world_transforms();

let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
let ray = Raycaster::from_camera_ndc(&camera, Vec2::ZERO);

let mut raycaster = Raycaster::new();
raycaster.build_bvh(&scene, &meshes).unwrap();
let hit = raycaster.cast_ray(ray, &scene, &meshes);

assert!(hit.is_some());
```

### Animato Integration

```rust
use std::collections::BTreeMap;
use scenix::{
    CameraId, CameraStores, MaterialId, NodeAnimationTarget, NodeAnimator,
    PerspectiveCamera, PbrMaterial, SceneGraph, SceneNode, ScenixAnimationDriver,
    Vec3, Vec3Track,
};

let mut scene = SceneGraph::new();
let node = scene.add(SceneNode::new("animated"));
let mut driver = ScenixAnimationDriver::new();
driver.add_node(NodeAnimator::new(
    node,
    NodeAnimationTarget::Translation(Vec3Track::tween(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 0.25)),
));

let camera_id = CameraId::new(1);
let mut perspective = BTreeMap::from([(camera_id, PerspectiveCamera::default())]);
let mut orthographic = BTreeMap::new();
let mut cameras = CameraStores { perspective: &mut perspective, orthographic: &mut orthographic };
let mut materials = BTreeMap::from([(MaterialId::new(1), PbrMaterial::new())]);
let mut skeletons = Vec::new();

driver.tick(0.25, &mut scene, &mut cameras, &mut materials, &mut skeletons).unwrap();
assert_eq!(scene.get(node).unwrap().transform.translation, Vec3::X);
```

### WASM Viewer

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn start(canvas: web_sys::HtmlCanvasElement) -> Result<scenix::WebRenderer, JsValue> {
    scenix::set_panic_hook();
    scenix::WebRenderer::new(canvas).await
}
```

The standalone example lives in `examples/wasm_viewer` and compiles with:

```sh
cargo check --manifest-path examples/wasm_viewer/Cargo.toml --target wasm32-unknown-unknown
```

### Debug Helpers

```rust
use scenix::{AxesHelper, BoundingBoxHelper, Color, GridHelper, LineGeometry, Aabb, Vec3};

let mut lines = LineGeometry::new();
lines.merge(&GridHelper::new(10.0, 10).to_geometry());
lines.merge(&AxesHelper::new(2.0).to_geometry());
lines.merge(
    &BoundingBoxHelper::new(Aabb::new(-Vec3::ONE, Vec3::ONE), Color::WHITE).to_geometry(),
);

lines.validate().unwrap();
assert_eq!(lines.segment_count(), 37);
```

### Texture And Camera

```rust
use scenix::{PerspectiveCamera, Texture2D, TextureFormat, Vec2, Vec3, mipmap};

let pixels = vec![
    255, 0, 0, 255, 0, 255, 0, 255,
    0, 0, 255, 255, 255, 255, 255, 255,
];
let mip_chain = mipmap::generate(&pixels, 2, 2).unwrap();
let texture = Texture2D::from_mips(2, 2, TextureFormat::Rgba8UnormSrgb, mip_chain).unwrap();

let camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 5.0))
    .target(Vec3::ZERO);
let ray = camera.screen_to_ray(Vec2::ZERO);

assert_eq!(texture.mip_levels, 2);
assert!(ray.direction.z < 0.0);
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | Enables standard-library conveniences for CPU crates. |
| `scene` | yes | Enables `scenix-scene` graph APIs from the facade crate. |
| `camera` | yes | Enables cameras, frustums, and controllers from the facade crate. |
| `mesh` | yes | Enables geometry and primitive APIs from the facade crate. |
| `material` | yes | Enables GPU-free material types and pipeline keys from the facade crate. |
| `light` | yes | Enables GPU-free light types, shadow config, and light probes from the facade crate. |
| `texture` | yes | Enables raw texture, sampler, atlas, video, and mipmap APIs from the facade crate. |
| `raycaster` | yes | Enables BVH scene picking and exact mesh ray intersection APIs. |
| `helpers` | yes | Enables debug `LineGeometry` helpers for grids, axes, bounds, cameras, lights, and skeletons. |
| `animato` | no | Enables optional Animato 1.4.0 animation bridge APIs. |
| `wasm` | no | Enables optional browser renderer wrapper and DOM input helpers. |
| `loader` | no | Enables optional `scenix-loader` asset loading APIs from the facade crate. |
| `renderer` | no | Enables optional `scenix-renderer`/`wgpu` APIs from the facade crate. |
| `post` | no | Enables optional `scenix-post` APIs and renderer post-stack integration. |
| `libm` | no | Uses `libm` for portable `no_std` trigonometry in `scenix-math`. |
| `serde` | no | Derives `Serialize` and `Deserialize` for public data types. |
| `approx` | no | Implements `approx` traits for math types. |
| `gpu` | no | Enables the `GpuUpload` trait in `scenix-core`. |

## Workspace Layout

```text
scenix/
├── crates/
│   ├── scenix-math/
│   ├── scenix-core/
│   ├── scenix-input/
│   ├── scenix-scene/
│   ├── scenix-camera/
│   ├── scenix-mesh/
│   ├── scenix-material/
│   ├── scenix-light/
│   ├── scenix-texture/
│   ├── scenix-loader/
│   ├── scenix-renderer/
│   ├── scenix-post/
│   ├── scenix-raycaster/
│   ├── scenix-helpers/
│   ├── scenix-animato/
│   ├── scenix-wasm/
│   └── scenix/
├── examples/
│   ├── animato_integration.rs
│   ├── wasm_viewer/
│   ├── raycasting.rs
│   ├── helpers_demo.rs
│   ├── gltf_scene.rs
│   ├── post_processing.rs
│   ├── hello_cube.rs
│   ├── pbr_sphere.rs
│   ├── shadow_demo.rs
│   ├── textures_and_camera.rs
│   └── orbit_camera.rs
├── benches/
│   ├── loader_bench.rs
│   ├── post_bench.rs
│   ├── bvh_bench.rs
│   ├── helpers_bench.rs
│   ├── animato_bridge_bench.rs
│   └── render_bench.rs
├── ARCHITECTURE.md
├── ROADMAP.md
├── CHANGELOG.md
└── README.md
```

## Running Checks

```sh
cargo fmt --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --all-features
cargo test -p scenix-math -p scenix-core -p scenix-input -p scenix-scene -p scenix-camera -p scenix-mesh -p scenix-material -p scenix-light -p scenix-texture -p scenix-raycaster -p scenix-helpers --no-default-features
cargo test -p scenix-loader --all-features
cargo test -p scenix-raycaster -p scenix-helpers --all-features
cargo test -p scenix-animato --all-features
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
cargo check --manifest-path examples/wasm_viewer/Cargo.toml --target wasm32-unknown-unknown
SCENIX_RUN_GPU_TESTS=1 WGPU_BACKEND=vulkan cargo test -p scenix-renderer -p scenix-post --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo bench --workspace --no-run
```

## Roadmap

The long-term design remains the full scenix workspace described in [ARCHITECTURE.md](./ARCHITECTURE.md). Version `0.9.0` adds Animato and browser integration. The next milestone is `v1.0.0` Stable.

See [ROADMAP.md](./ROADMAP.md) for the full versioned plan.

## License

Licensed under either of:

- [MIT License](./LICENSE-MIT)
- [Apache License, Version 2.0](./LICENSE-APACHE)

at your option.
