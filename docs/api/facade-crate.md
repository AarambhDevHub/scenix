# `scenix`

## Role

Facade crate for applications that want stable v1 imports from one package.

## Dependency Weight

Default CPU authoring plus optional heavy systems.

## Install

```toml
[dependencies]
scenix = "1"
```

## Key Public API

SceneGraph, SceneNode, PerspectiveCamera, Geometry, PbrMaterial, Raycaster, Renderer, GltfLoader, PostStack, ScenixAnimationDriver, BrowserRenderer, WebRenderer

## Common Use

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

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
