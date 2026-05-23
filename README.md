# scenix

> Italian: scenix - scene, the stage on which everything appears.

[![CI](https://github.com/AarambhDevHub/scenix/actions/workflows/ci.yml/badge.svg)](https://github.com/AarambhDevHub/scenix/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

scenix v0.5.0 is the Textures & Camera release of a renderer-agnostic 3D scene library for Rust.

This release ships the GPU-free foundation, scene graph, CPU-side geometry, materials, lights, textures, and cameras:

- `scenix-math`: vectors, matrices, quaternions, transforms, rays, bounds, planes, and coordinate helpers.
- `scenix-core`: typed IDs, color, errors, and shared traits.
- `scenix-input`: pointer and keyboard state.
- `scenix-scene`: scene node hierarchy, transform propagation, traversal, fog, sprites, and LOD helpers.
- `scenix-camera`: perspective, orthographic, and cube cameras, frustums, screen rays, orbit controls, and fly controls.
- `scenix-mesh`: geometry buffers, primitive generation, morph targets, instancing, and batching helpers.
- `scenix-material`: GPU-free material traits, pipeline keys, PBR, physical, toon, line, point, and custom shader materials.
- `scenix-light`: GPU-free light types, shadow configuration, and raw-sample spherical-harmonics light probes.
- `scenix-texture`: raw CPU texture data, samplers, atlases, video-frame updates, and RGBA8 mipmap generation.
- `scenix`: facade crate re-exporting the default APIs.

Renderer, file loaders, post-processing, raycasting, helpers, WASM integration, and `animato` integration are planned in later roadmap milestones.

## Installation

Most users should start with the facade crate:

```toml
[dependencies]
scenix = "0.5"
```

Use focused crates directly when you only need one layer:

```toml
[dependencies]
scenix-math = "0.5"
scenix-core = "0.5"
scenix-input = "0.5"
scenix-scene = "0.5"
scenix-camera = "0.5"
scenix-mesh = "0.5"
scenix-material = "0.5"
scenix-light = "0.5"
scenix-texture = "0.5"
```

For `no_std`-capable crates with portable math trigonometry:

```toml
[dependencies]
scenix-math = { version = "0.5", default-features = false, features = ["libm"] }
scenix-core = { version = "0.5", default-features = false }
scenix-input = { version = "0.5", default-features = false }
scenix-scene = { version = "0.5", default-features = false }
scenix-camera = { version = "0.5", default-features = false }
scenix-mesh = { version = "0.5", default-features = false }
scenix-material = { version = "0.5", default-features = false }
scenix-light = { version = "0.5", default-features = false }
scenix-texture = { version = "0.5", default-features = false }
```

## Quick Start

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

### Orbit Camera

```rust
use scenix::{OrbitController, PerspectiveCamera, Vec2, Vec3};

let mut camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 500.0);
let mut orbit = OrbitController::new(Vec3::ZERO, 8.0);

orbit.on_drag(Vec2::new(24.0, -12.0), 1.0 / 60.0);
orbit.on_scroll(-0.5, 1.0 / 60.0);
orbit.apply_to_perspective(&mut camera);

assert!(camera.frustum().contains_point(Vec3::ZERO));
```

### Atlas And Sampler

```rust
use scenix::{AddressMode, Sampler, TextureAtlas};

let mut atlas = TextureAtlas::with_padding(256, 256, 2);
atlas.insert("hero", 64, 64).unwrap();
let uv = atlas.uv("hero").unwrap();

let sampler = Sampler::new()
    .address_modes(AddressMode::Repeat, AddressMode::Repeat, AddressMode::ClampToEdge)
    .anisotropy(8);

assert!(uv.u1 > uv.u0);
assert_eq!(sampler.anisotropy, 8);
```

### Materials And Lights

```rust
use scenix::{Color, DirectionalLight, Material, PbrMaterial, ShadowConfig, Vec3};

let material = PbrMaterial::new()
    .albedo(Color::from_hex(0xCC_88_44).to_linear())
    .metallic_roughness(0.0, 0.55);

let sun = DirectionalLight::new(Vec3::new(-1.0, -2.0, -1.0), Color::WHITE, 3.0)
    .shadow(ShadowConfig::default());

assert!(!material.is_transparent());
assert!(sun.shadow.unwrap().validate().is_ok());
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
| `std` | yes | Enables standard-library conveniences. |
| `scene` | yes | Enables the `scenix-scene` graph API from the facade crate. |
| `camera` | yes | Enables cameras, frustums, and controllers from the facade crate. |
| `mesh` | yes | Enables geometry and primitive APIs from the facade crate. |
| `material` | yes | Enables GPU-free material types and pipeline keys from the facade crate. |
| `light` | yes | Enables GPU-free light types, shadow config, and light probes from the facade crate. |
| `texture` | yes | Enables raw texture, sampler, atlas, video, and mipmap APIs from the facade crate. |
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
│   └── scenix/
├── examples/
│   ├── textures_and_camera.rs
│   └── orbit_camera.rs
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
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo bench --workspace --no-run
```

## Roadmap

The long-term design remains the full scenix workspace described in [ARCHITECTURE.md](./ARCHITECTURE.md). Version `0.5.0` adds GPU-free texture and camera systems on top of the Foundation, Scene Graph, Geometry, Materials, and Lights APIs. Upcoming milestones add renderer, loaders, post-processing, raycasting, helpers, `animato`, and WASM integration.

See [ROADMAP.md](./ROADMAP.md) for the full versioned plan.

## License

Licensed under either of:

- [MIT License](./LICENSE-MIT)
- [Apache License, Version 2.0](./LICENSE-APACHE)

at your option.
