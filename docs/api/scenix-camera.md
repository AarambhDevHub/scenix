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

PerspectiveCamera, OrthographicCamera, CubeCamera, Frustum, OrbitController, FlyController

## Common Use

```rust
use scenix_camera::PerspectiveCamera;
let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0);
# let _ = camera;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
