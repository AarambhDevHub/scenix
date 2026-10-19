# `scenix-water`

## Role

Gerstner wave fields, displaced surface meshes, foam and fresnel shapes,
underwater tinting, and planar reflection helpers (v1.8.0).

## Dependency Weight

Lightweight `no_std` on `scenix-camera`, `scenix-core`, `scenix-math`, and
`scenix-mesh`.

## Install

```toml
[dependencies]
scenix-water = "1"
```

## Key Public API

`WaveField`, `GerstnerWave`, `WaveSample`, `MAX_WAVES`, `WaterSurface`,
`WaterConfig`, `FoamParams`, `FresnelParams`, `UnderwaterView`,
`ReflectionSetup`, `mirror_transform`, `reflected_camera`

## Common Use

```rust
use scenix_water::{WaterConfig, WaterSurface, WaveField};
let surface = WaterSurface::new(WaveField::ocean_shore())
    .with_config(WaterConfig::new(24.0, 24.0, 6, 1.5));
let geometry = surface.geometry(0.6);
# let _ = geometry;
```

## Notes

v1.8.0 adds this crate. `WaterSurface::geometry(t)` is the CPU source of
truth: every vertex height equals `height_at` at the same timestamp, and the
GPU paver in the renderer replays the same wave math. `ocean_shore` stacks
three waves; `FoamParams` gates crest energy hard at its threshold;
`FresnelParams::water` mirrors fully at grazing angles; `UnderwaterView`
absorbs red first and drifts toward murk with depth.

## Related Docs

- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.8.0 release](../release-v1.8.0.md)
