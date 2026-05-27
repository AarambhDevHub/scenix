# Materials

## Purpose

Use GPU-free material descriptions and renderer material registrations.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `material`; rendering needs `renderer`.

## Key Rules

- PBR is the standard material path.
- Physical, toon, normal, wireframe, unlit, and lambert materials cover v1 examples.
- Advanced physical shading is a preview contract in v1.


## Example

```rust
use scenix::{Color, PbrMaterial};

let material = PbrMaterial::new().albedo(Color::rgb(0.2, 0.8, 0.7));
# let _ = material;
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
