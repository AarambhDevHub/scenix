# Pick Objects With Raycaster

## Goal

Convert camera coordinates to a ray and test scene meshes.

## Relevant Feature Flags

Default `raycaster`.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

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

Run `cargo run -p scenix --example raycasting`.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
