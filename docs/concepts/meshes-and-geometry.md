# Meshes And Geometry

## Purpose

Create CPU vertex/index data and primitive geometry.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `mesh`.

## Key Rules

- Geometry is CPU data until registered with the renderer.
- Use primitive generators for examples and tests.
- Use instancing or batching for repeated geometry.


## Example

```rust
use scenix::box_geometry;

let cube = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);
assert!(!cube.positions.is_empty());
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
