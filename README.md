# scenix

> Italian: scenix - scene, the stage on which everything appears.

[![CI](https://github.com/AarambhDevHub/scenix/actions/workflows/ci.yml/badge.svg)](https://github.com/AarambhDevHub/scenix/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

scenix v0.3.0 is the Geometry release of a renderer-agnostic 3D scene library for Rust.

This release ships the GPU-free foundation, scene graph, and CPU-side geometry layer:

- `scenix-math`: vectors, matrices, quaternions, transforms, rays, bounds, planes, and coordinate helpers.
- `scenix-core`: typed IDs, color, errors, and shared traits.
- `scenix-input`: pointer and keyboard state.
- `scenix-scene`: scene node hierarchy, transform propagation, traversal, fog, sprites, and LOD helpers.
- `scenix-mesh`: geometry buffers, primitive generation, morph targets, instancing, and batching helpers.
- `scenix`: facade crate re-exporting the default APIs.

Materials, lights, cameras, textures, renderer, loaders, WASM integration, and `animato` integration are planned in later roadmap milestones.

## Installation

Most users should start with the facade crate:

```toml
[dependencies]
scenix = "0.3"
```

Use the focused crates directly when you only need one layer:

```toml
[dependencies]
scenix-math = "0.3"
scenix-core = "0.3"
scenix-input = "0.3"
scenix-scene = "0.3"
scenix-mesh = "0.3"
```

For `no_std` math with portable trigonometry:

```toml
[dependencies]
scenix-math = { version = "0.3", default-features = false, features = ["libm"] }
scenix-core = { version = "0.3", default-features = false }
scenix-input = { version = "0.3", default-features = false }
scenix-scene = { version = "0.3", default-features = false }
scenix-mesh = { version = "0.3", default-features = false }
```

## Quick Start

### Geometry And Meshes

```rust
use scenix::{MaterialId, Mesh, SceneNode, sphere_geometry};

let geometry = sphere_geometry(1.0, 32, 16);
let material = MaterialId::new(1);
let mesh = Mesh::new(geometry, material);
let node = SceneNode::mesh("planet", scenix::MeshId::new(1), material);

assert!(!mesh.geometry.positions.is_empty());
assert!(matches!(node.kind, scenix::NodeKind::Mesh { .. }));
```

### Scene Graph

```rust
use scenix::{SceneGraph, SceneNode, Transform, Vec3};

let mut scene = SceneGraph::new();
let root = scene.add(SceneNode::group("root"));
let child = scene
    .add_child(
        root,
        SceneNode::new("child")
            .transform(Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))),
    )
    .unwrap();

scene.update_world_transforms();

assert_eq!(scene.parent(child), Some(root));
assert_eq!(
    scene.world_matrix(child).unwrap().mul_vec3(Vec3::ZERO),
    Vec3::new(1.0, 2.0, 3.0)
);
```

### Vector And Quaternion Math

```rust
use scenix::{Quat, Vec3};

let right = Vec3::X;
let up = Vec3::Y;
let forward = right.cross(up);

let rotation = Quat::from_axis_angle(Vec3::Y, core::f32::consts::FRAC_PI_2);
let rotated = rotation.mul_vec3(Vec3::X);

assert_eq!(forward, Vec3::Z);
assert!((rotated.z + 1.0).abs() < 1.0e-5);
```

### Matrices And Transforms

```rust
use scenix::{Mat4, Quat, Transform, Vec3};

let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))
    .rotate_by(Quat::from_axis_angle(Vec3::Y, 0.5))
    .scale_by(Vec3::new(2.0, 2.0, 2.0));

let matrix = transform.to_mat4();
let projection = Mat4::perspective(core::f32::consts::FRAC_PI_3, 16.0 / 9.0, 0.1, 1000.0);

assert_eq!(matrix.mul_vec3(Vec3::ZERO), Vec3::new(1.0, 2.0, 3.0));
assert_eq!(projection.to_cols_array()[15], 0.0);
```

### Ray Intersections

```rust
use scenix::{Aabb, Ray3, Vec3};

let ray = Ray3::new(Vec3::new(0.0, 0.0, 5.0), Vec3::NEG_Z);
let bounds = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));

let hit_t = ray.intersect_aabb(bounds).unwrap();
assert_eq!(ray.at(hit_t), Vec3::new(0.0, 0.0, 1.0));
```

### Color And IDs

```rust
use scenix::{Color, MaterialId, NodeId};

let node = NodeId::new(42);
let material = MaterialId::new(7);
let color = Color::from_hex(0xFF_80_00).to_linear();

assert_eq!(node.get(), 42);
assert_eq!(material.get(), 7);
assert_eq!(color.a, 1.0);
```

### Input State

```rust
use scenix::{KeyCode, KeyboardState, PointerButton, PointerState, Vec2};

let mut keyboard = KeyboardState::new();
keyboard.on_key_down(KeyCode::KeyW);
assert!(keyboard.is_pressed(KeyCode::KeyW));

let mut pointer = PointerState::new();
pointer.set_position(Vec2::new(100.0, 50.0));
pointer.on_button_down(PointerButton::Left);
assert!(pointer.is_pressed(PointerButton::Left));
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | Enables standard-library conveniences such as `std::error::Error` and `Named`. |
| `scene` | yes | Enables the `scenix-scene` graph API from the facade crate. |
| `mesh` | yes | Enables the `scenix-mesh` geometry and primitive API from the facade crate. |
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
│   ├── scenix-mesh/
│   └── scenix/
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
cargo test -p scenix-math -p scenix-core -p scenix-input -p scenix-scene -p scenix-mesh --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo bench --workspace --no-run
```

## Roadmap

The long-term design remains the full scenix workspace described in [ARCHITECTURE.md](./ARCHITECTURE.md). Version `0.3.0` adds mesh geometry and primitive generation on top of the Foundation and Scene Graph APIs. Upcoming milestones add materials, lights, cameras, textures, renderer, loaders, raycasting, helpers, `animato`, and WASM integration.

See [ROADMAP.md](./ROADMAP.md) for the full versioned plan.

## License

Licensed under either of:

- [MIT License](./LICENSE-MIT)
- [Apache License, Version 2.0](./LICENSE-APACHE)

at your option.
