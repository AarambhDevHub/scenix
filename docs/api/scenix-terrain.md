# `scenix-terrain`

## Role

CPU heightfields, chunked LOD grids, splat maps, collision probes, and chunk
streaming (v1.8.0).

## Dependency Weight

Lightweight `no_std` on `scenix-core`, `scenix-math`, and `scenix-mesh`.

## Install

```toml
[dependencies]
scenix-terrain = "1"
```

## Key Public API

`Heightmap`, `TerrainCollider`, `TerrainGrid`, `TerrainConfig`,
`TerrainChunk`, `LodPolicy`, `LodBand`, `NoiseConfig`, `noise_heights`,
`SplatLayer`, `SplatMap`, `ChunkStreamer`, `ChunkCoord`, `StreamDiff`

## Common Use

```rust
use scenix_terrain::{Heightmap, LodBand, LodPolicy, TerrainConfig, TerrainGrid};
let map = Heightmap::from_fn(17, 17, 1.0, |ix, iz| ((ix + iz) as f32) * 0.5);
let policy = LodPolicy::new(&[LodBand::new(40.0, 0)])?;
let grid = TerrainGrid::generate(&map, TerrainConfig::new(2, 8, policy));
let geometry = grid.chunk_geometry(0, 0)?;
# fn main() -> Result<(), Box<dyn std::error::Error>> { let map = Heightmap::from_fn(17, 17, 1.0, |ix, iz| ((ix + iz) as f32) * 0.5); let policy = LodPolicy::new(&[LodBand::new(40.0, 0)])?; let grid = TerrainGrid::generate(&map, TerrainConfig::new(2, 8, policy)); let _ = grid.chunk_geometry(0, 0)?; Ok(()) }
```

## Notes

v1.8.0 adds this crate. GPU chunk registration lives behind
`scenix-renderer`'s `environment` feature, and the material state lives in
`scenix-material`'s `TerrainMaterial`. Deterministic seeded noise replays
identically; raw noise stays in `[-1, 1]` until `remap_range` refits it.

## Related Docs

- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.8.0 release](../release-v1.8.0.md)
