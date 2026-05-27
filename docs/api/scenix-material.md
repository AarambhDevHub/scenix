# `scenix-material`

## Role

GPU-free material descriptions and pipeline keys.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-material = "1"
```

## Key Public API

PbrMaterial, PhysicalMaterial, UnlitMaterial, LambertMaterial, ToonMaterial, WireframeMaterial, NormalMaterial, PipelineKey

## Common Use

```rust
use scenix_material::PbrMaterial;
let material = PbrMaterial::new();
# let _ = material;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
