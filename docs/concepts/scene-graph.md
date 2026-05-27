# Scene Graph

## Purpose

Model hierarchy, visibility, layers, mesh IDs, material IDs, and transform ownership.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `scene`.

## Key Rules

- `SceneGraph` stores nodes and hierarchy.
- Mesh and material data live in caller-owned stores.
- Call `update_world_transforms()` after edits.


## Example

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

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
