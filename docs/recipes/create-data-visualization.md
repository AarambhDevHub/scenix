# Create Data Visualization

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Generate geometry or helper lines from data and keep renderer dependencies optional until display time.

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

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
