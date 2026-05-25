# scenix

> Italian: scenix - scene, the stage on which everything appears.

[![CI](https://github.com/AarambhDevHub/scenix/actions/workflows/ci.yml/badge.svg)](https://github.com/AarambhDevHub/scenix/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

scenix v0.6.0 is the Renderer release. It adds the first `wgpu` GPU layer while keeping the facade crate CPU-only by default.

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
- `scenix-renderer`: optional `wgpu` renderer with headless/surface targets, renderer-owned resource registries, G-buffer/shadow targets, culling/sorting helpers, and pipeline caching.
- `scenix`: facade crate re-exporting CPU APIs by default and renderer APIs behind `features = ["renderer"]`.

File loaders, image decoding, post-processing, raycasting, helpers, WASM integration, and `animato` integration remain later roadmap milestones.

## Installation

Most users should start with the facade crate:

```toml
[dependencies]
scenix = "0.6"
```

Enable GPU rendering explicitly:

```toml
[dependencies]
scenix = { version = "0.6", features = ["renderer"] }
```

Use focused crates directly when you only need one layer:

```toml
[dependencies]
scenix-math = "0.6"
scenix-core = "0.6"
scenix-input = "0.6"
scenix-scene = "0.6"
scenix-camera = "0.6"
scenix-mesh = "0.6"
scenix-material = "0.6"
scenix-light = "0.6"
scenix-texture = "0.6"
scenix-renderer = "0.6"
```

For `no_std`-capable CPU crates:

```toml
[dependencies]
scenix-math = { version = "0.6", default-features = false, features = ["libm"] }
scenix-core = { version = "0.6", default-features = false }
scenix-input = { version = "0.6", default-features = false }
scenix-scene = { version = "0.6", default-features = false }
scenix-camera = { version = "0.6", default-features = false }
scenix-mesh = { version = "0.6", default-features = false }
scenix-material = { version = "0.6", default-features = false }
scenix-light = { version = "0.6", default-features = false }
scenix-texture = { version = "0.6", default-features = false }
```

`scenix-renderer` is `std` + `wgpu` only.

## Quick Start

### Headless Renderer

```rust
use scenix::{
    Color, MaterialId, MeshId, PerspectiveCamera, PbrMaterial, Renderer, RendererConfig,
    SceneGraph, SceneNode, Vec3, box_geometry,
};

# async fn run() -> Result<(), scenix::ScenixError> {
let mut renderer = Renderer::headless(RendererConfig::new(256, 256)).await?;
let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);

renderer.register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;
renderer.register_pbr_material(
    material_id,
    &PbrMaterial::new()
        .albedo(Color::from_rgb(0.8, 0.25, 0.15))
        .metallic_roughness(0.0, 0.65),
)?;

let mut scene = SceneGraph::new();
scene.add(SceneNode::mesh("cube", mesh_id, material_id));
scene.update_world_transforms();

let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
let stats = renderer.render(&scene, &camera)?;

assert_eq!(stats.visible_meshes, 1);
# Ok(())
# }
```

### Textures And Camera

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

### Geometry And Scene Graph

```rust
use scenix::{MaterialId, Mesh, MeshId, SceneGraph, SceneNode, box_geometry};

let geometry = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);
let material_id = MaterialId::new(1);
let mesh = Mesh::new(geometry, material_id);

let mut scene = SceneGraph::new();
let node = scene.add(SceneNode::mesh("cube", MeshId::new(1), material_id));

assert!(!mesh.geometry.positions.is_empty());
assert!(scene.get(node).is_some());
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
| `renderer` | no | Enables optional `scenix-renderer`/`wgpu` APIs from the facade crate. |
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
│   ├── scenix-renderer/
│   └── scenix/
├── examples/
│   ├── hello_cube.rs
│   ├── pbr_sphere.rs
│   ├── shadow_demo.rs
│   ├── textures_and_camera.rs
│   └── orbit_camera.rs
├── benches/
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
cargo test -p scenix-math -p scenix-core -p scenix-input -p scenix-scene -p scenix-camera -p scenix-mesh -p scenix-material -p scenix-light -p scenix-texture --no-default-features
SCENIX_RUN_GPU_TESTS=1 WGPU_BACKEND=vulkan cargo test -p scenix-renderer --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo bench --workspace --no-run
```

## Roadmap

The long-term design remains the full scenix workspace described in [ARCHITECTURE.md](./ARCHITECTURE.md). Version `0.6.0` adds optional GPU rendering on top of the Foundation, Scene Graph, Geometry, Materials, Lights, Textures, and Camera APIs. The next milestone is `v0.7.0` Loaders & Post-Processing.

See [ROADMAP.md](./ROADMAP.md) for the full versioned plan.

## License

Licensed under either of:

- [MIT License](./LICENSE-MIT)
- [Apache License, Version 2.0](./LICENSE-APACHE)

at your option.
