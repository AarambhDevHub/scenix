# `scenix-scene`

## Role

Scene graph nodes, hierarchy, transforms, traversal, fog, sprites, LOD helpers,
selection state, layer policies, snapping, and sparse editor metadata.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-scene = "1"
```

## Key Public API

`SceneGraph`, `SceneNode`, `NodeKind`, `SelectionState`, `SelectionMode`,
`NodeEditorMetadata`, `LayerMask`, `LayerPolicy`, `TransformMode`,
`TransformSpace`, `TransformConstraint`, `SnapSettings`, `Fog`, `LodGroup`, and
`Sprite`.

## Common Use

```rust
use scenix::{MaterialId, MeshId, SceneGraph, SceneNode, box_geometry};

let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);
let geometry = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);

let mut scene = SceneGraph::new();
let cube = scene.add(SceneNode::mesh("cube", mesh_id, material_id));
scene.select(cube, scenix::SelectionMode::Replace).unwrap();
scene.update_world_transforms();
# let _ = geometry;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Interaction and editor primitives](../concepts/interaction-and-editor.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
