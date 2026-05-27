# Switch Material At Runtime

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Change a node material ID or update the renderer material registration for the same `MaterialId`.

## Example

```rust
use scenix::{MaterialId, PbrMaterial};
let material_id = MaterialId::new(1);
let material = PbrMaterial::new();
# let _ = (material_id, material);
```

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
