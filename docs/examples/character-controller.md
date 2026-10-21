# Character Controller

## Purpose

Grounds a kinematic capsule, walks it at a fixed wish speed, jumps, and lands back on the floor while reporting per-frame step data.

## Source

`examples/character_controller.rs`

## Relevant Feature Flags

facade `physics` features

## Run Or Check

```sh
cargo run -p scenix --example character_controller --features physics
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- The jump returns to the exact start height: the ground probe ignores the
  capsule itself and re-attachment is speed-gated.

## Related Docs

- [Examples index](README.md)
- [scenix-physics crate reference](../api/scenix-physics.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
