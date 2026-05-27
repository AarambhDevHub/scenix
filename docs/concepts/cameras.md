# Cameras

## Purpose

Choose perspective, orthographic, cube, orbit, and fly camera tools.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `camera`.

## Key Rules

- Perspective cameras are common for interactive scenes.
- Orthographic cameras fit editors and technical views.
- Frustums support visibility tests and helpers.


## Example

```rust
use scenix::{OrbitController, PerspectiveCamera, Vec3};

let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 2.0, 5.0))
    .target(Vec3::ZERO);
let controller = OrbitController::new(Vec3::ZERO, 5.0);
# let _ = (camera, controller);
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
