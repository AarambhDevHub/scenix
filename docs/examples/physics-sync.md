# Physics Sync

## Purpose

Drops a ball over a static floor, steps it to rest, snapshots the transform sync queue the scene would consume, and produces the collider debug wireframe for the renderer's debug pass.

## Source

`examples/physics_sync.rs`

## Relevant Feature Flags

facade `physics` (and `helpers`) features

## Run Or Check

```sh
cargo run -p scenix --example physics_sync --features physics,helpers
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Simulations are deterministic against body ordering.

## Related Docs

- [Examples index](README.md)
- [scenix-physics crate reference](../api/scenix-physics.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
