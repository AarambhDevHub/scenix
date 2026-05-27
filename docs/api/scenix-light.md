# `scenix-light`

## Role

CPU light descriptions, shadow settings, and light probes.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-light = "1"
```

## Key Public API

AmbientLight, DirectionalLight, PointLight, SpotLight, HemisphereLight, LightProbe, ShadowSettings

## Common Use

```rust
use scenix_light::AmbientLight;
let light = AmbientLight::default();
# let _ = light;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
