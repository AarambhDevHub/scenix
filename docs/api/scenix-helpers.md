# `scenix-helpers`

## Role

Debug line geometry generators for editors and diagnostics.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-helpers = "1"
```

## Key Public API

LineGeometry, GridHelper, AxesHelper, BoundingBoxHelper, ArrowHelper, CameraHelper, light helpers, SkeletonHelper

## Common Use

```rust
use scenix_helpers::AxesHelper;
let axes = AxesHelper::new(1.0).geometry();
# let _ = axes;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
