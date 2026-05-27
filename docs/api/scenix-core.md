# `scenix-core`

## Role

Shared IDs, colors, errors, and traits used across crates.

## Dependency Weight

Lightweight `no_std`.

## Install

```toml
[dependencies]
scenix-core = "1"
```

## Key Public API

NodeId, MeshId, MaterialId, TextureId, LightId, CameraId, Color, ScenixError, ValidationError

## Common Use

```rust
use scenix_core::{Color, MeshId};
let mesh = MeshId::new(1);
let color = Color::rgb(1.0, 0.8, 0.2);
# let _ = (mesh, color);
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
