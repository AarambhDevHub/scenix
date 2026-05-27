# Getting Started

Scenix is a modular Rust 3D workspace. Use the `scenix` facade when you want the common CPU authoring API, and opt into heavier systems with feature flags.

## Install

```toml
[dependencies]
scenix = "1"
```

Optional systems:

```toml
scenix = { version = "1", features = ["loader"] }
scenix = { version = "1", features = ["renderer", "post"] }
scenix = { version = "1", features = ["animato"] }
scenix = { version = "1", features = ["wasm"] }
```

## Create A Scene

```rust
use scenix::{MaterialId, MeshId, SceneGraph, SceneNode, box_geometry};

let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);
let _geometry = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);

let mut scene = SceneGraph::new();
scene.add(SceneNode::mesh("cube", mesh_id, material_id));
scene.update_world_transforms();
```

## Render Headless

The renderer owns GPU resources. Register CPU scene resources once, then call `render`.

```rust
use scenix::{PerspectiveCamera, Renderer, RendererConfig, Vec3};

# async fn run(scene: &scenix::SceneGraph) -> Result<(), scenix::ScenixError> {
let mut renderer = Renderer::headless(RendererConfig::new(512, 512)).await?;
let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
renderer.render(scene, &camera)?;
# Ok(())
# }
```

## Build The Website

```sh
cd website
trunk serve
trunk build --release --public-url /scenix/
```

The site is client-side Leptos and deploys as static files to GitHub Pages.
