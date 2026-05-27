# `scenix-mesh`

## Role

CPU geometry buffers, primitive generators, instancing, batching, and morph targets.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-mesh = "1"
```

## Key Public API

Geometry, Mesh, BufferLayout, InstancedMesh, BatchedMesh, MorphTarget, primitive generators

## Common Use

```rust
use scenix_mesh::box_geometry;
let cube = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);
# let _ = cube;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
