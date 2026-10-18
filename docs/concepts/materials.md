# Materials

## Purpose

Use GPU-free material descriptions and renderer material registrations.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `material`; rendering needs `renderer`.
Node-authored materials live in the optional `nodes` feature (`scenix-nodes`),
which builds graphs and generates WGSL/GLSL for the renderer built-ins above.

## Key Rules

- PBR is the standard material path.
- Physical, toon, normal, wireframe, unlit, and lambert materials cover v1 examples.
- v1.2 renderer registrations upload material uniforms and bound textures for the active GPU path.
- v1.6 `NodeMaterial` pairs a typed, serializable graph with the same CPU state
  surface (base color, alpha mode, textures); it reports a `Custom` shader-kind
  pipeline key and validates through typed `NodeDiagnostic`s.
- Advanced physical shading is a realtime approximation, not an offline film renderer.


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
