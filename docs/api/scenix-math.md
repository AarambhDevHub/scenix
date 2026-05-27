# `scenix-math`

## Role

Vectors, matrices, quaternions, transforms, rays, planes, bounds, and coordinate helpers.

## Dependency Weight

Lightweight `no_std`; use `libm` without `std`.

## Install

```toml
[dependencies]
scenix-math = "1"
```

## Key Public API

Vec2, Vec3, Vec4, Mat4, Quat, Transform, Ray3, Aabb, Sphere, Plane

## Common Use

```rust
use scenix_math::{Transform, Vec3};
let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
# let _ = t;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
