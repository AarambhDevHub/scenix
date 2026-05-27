# Create Debug Grid

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Generate `LineGeometry` grid data and draw it with your line-capable debug renderer.

## Example

```rust
use scenix::GridHelper;
let grid = GridHelper::new(20, 0.5).geometry();
# let _ = grid;
```

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
