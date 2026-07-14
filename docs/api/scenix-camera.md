# `scenix-camera`

## Role

Perspective, orthographic, cube cameras, frustums, and controllers.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-camera = "1"
```

## Key Public API

`PerspectiveCamera`, `OrthographicCamera`, `CubeCamera`, `Frustum`,
`OrbitController`, `FlyController`, `ArcballController`, `TrackballController`,
`MapController`, `FirstPersonController`, and `PointerLockController`.

## Common Use

```rust
use scenix_camera::{ArcballController, PerspectiveCamera};
use scenix_input::InputState;
use scenix_math::Vec3;

let input = InputState::default();
let mut controls = ArcballController::new(Vec3::ZERO, 5.0);
let mut camera = PerspectiveCamera::default();
controls.update_from_input(&input, 1.0 / 60.0);
controls.apply_to_perspective(&mut camera);
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Interaction and editor primitives](../concepts/interaction-and-editor.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
