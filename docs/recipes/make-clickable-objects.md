# Make Clickable Objects

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Use the camera to create a ray, then query `Raycaster` for the nearest hit.

## Example

```rust
use scenix::{PerspectiveCamera, Raycaster, Vec2, Vec3};

let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
let ray = Raycaster::from_camera_ndc(&camera, Vec2::ZERO);
# let _ = ray;
```

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
