# Lights

## Purpose

Add ambient, directional, point, spot, area, hemisphere, and probe light data.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `light`; shadows need `renderer`.

## Key Rules

- Lights are CPU descriptions.
- Renderer registration uploads light data.
- Shadow settings live with light configuration.


## Example

```rust
use scenix::{DirectionalLight, Vec3};

let light = DirectionalLight::new(Vec3::new(-1.0, -2.0, -1.0));
# let _ = light;
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
