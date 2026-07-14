# `scenix-raycaster`

## Role

BVH-accelerated picking, exact mesh intersections, selection volumes, drag
planes, and reversible transform interactions.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-raycaster = "1"
```

## Key Public API

`Raycaster`, `Intersection`, `Bvh`, `GeometryProvider`, `SelectionRect`,
`SelectionFrustum`, `SelectionContainment`, `DragPlane`, `DragController`, and
`TransformController`.

## Common Use

```rust
use scenix::{PerspectiveCamera, Raycaster, Vec2, Vec3};

let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
let ray = Raycaster::from_camera_ndc(&camera, Vec2::ZERO);
# let _ = ray;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Selection and drag example](../examples/selection-and-drag.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
