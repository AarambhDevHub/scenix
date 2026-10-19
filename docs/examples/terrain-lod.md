# Terrain LOD

## Purpose

Builds a seeded heightfield, splits it into LOD chunks, streams chunks around a moving camera, and probes the terrain collider with a downward ray.

## Source

`examples/terrain_lod.rs`

## Relevant Feature Flags

facade `terrain` feature

## Run Or Check

```sh
cargo run -p scenix --example terrain_lod --features terrain
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Scenix v1.8.0 release](../release-v1.8.0.md)
