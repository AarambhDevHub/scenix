# `scenix-sky`

## Role

Procedural sky profiles, sun positions, deterministic environment bakes, and
grounded skyboxes (v1.8.0).

## Dependency Weight

Lightweight `no_std` on `scenix-core`, `scenix-light`, and `scenix-math`.

## Install

```toml
[dependencies]
scenix-sky = "1"
```

## Key Public API

`SkyModel`, `SunPosition`, `GroundedSkybox`, `SkyAmbient`,
`bake_equirectangular`, `bake_light_probe`, `bake_ambient`

## Common Use

```rust
use scenix_sky::{SkyModel, SunPosition, bake_equirectangular};
use scenix_math::Vec3;
let model = SkyModel::clear_day();
let sun = SunPosition::degrees(35.0, 40.0);
let zenith = model.rgb_at(Vec3::Y, sun.direction());
# let _ = zenith;
```

## Notes

v1.8.0 adds this crate. Sky colors are pure functions of direction, so the
same tap always replays identically; `bake_equirectangular` keeps the
row-0 = zenith convention that the renderer and `scenix-texture`'s
`EquirectSampler` share. `SunPosition::sun_light` builds a warm
`SunLight` for low elevations; the material state that drives the
WGSL dome lives in `scenix-material`'s `SkyMaterial`.

## Related Docs

- [scenix-light crate reference](scenix-light.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.8.0 release](../release-v1.8.0.md)
