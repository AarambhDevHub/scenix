# Show Bounding Box On Hover

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Use raycaster selection to choose a node, then generate `BoundingBoxHelper` line geometry for that node bounds.

## Example

```rust
use scenix::{Aabb, BoundingBoxHelper, Vec3};
let bounds = Aabb::new(Vec3::splat(-1.0), Vec3::splat(1.0));
let lines = BoundingBoxHelper::new(bounds).geometry();
# let _ = lines;
```

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
