# XR Controllers

## Purpose

Updates both controllers with tracked poses, reads pointer rays and velocity-predicted poses through the bay, and confirms the handedness checks the facade re-exports.

## Source

`examples/xr_controllers.rs`

## Relevant Feature Flags

facade `xr` features

## Run Or Check

```sh
cargo run -p scenix --example xr_controllers --features xr
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Controller snapshots share the `scenix-input` XR vocabulary.

## Related Docs

- [Examples index](README.md)
- [scenix-xr crate reference](../api/scenix-xr.md)
- [scenix-input crate reference](../api/scenix-input.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
