# Getting Started

Use this page when you are new to Scenix and want to create a scene without choosing every subsystem up front. The default facade gives you CPU scene authoring, cameras, geometry, materials, lights, textures, raycasting, and helpers.

## Install

```toml
[dependencies]
scenix = "1"
```

Add heavier systems only when needed:

```toml
scenix = { version = "1", features = ["renderer"] }
scenix = { version = "1", features = ["loader", "renderer"] }
scenix = { version = "1", features = ["renderer", "post"] }
scenix = { version = "1", features = ["animato"] }
scenix = { version = "1", features = ["wasm"] }
```

## First Scene

```rust
use scenix::{MaterialId, MeshId, SceneGraph, SceneNode, box_geometry};

let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);
let geometry = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);

let mut scene = SceneGraph::new();
scene.add(SceneNode::mesh("cube", mesh_id, material_id));
scene.update_world_transforms();
# let _ = geometry;
```

## What To Read Next

- [Installation](installation.md) for feature and crate choices.
- [Quick start](quick-start.md) for copyable snippets.
- [Scene graph](concepts/scene-graph.md) for hierarchy and transform rules.
- [Render a cube](guides/render-a-cube.md) when you are ready to use `wgpu`.
