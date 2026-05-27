# Environment Map

## Purpose

Uses texture cube and environment-facing data structures.

## Source

`examples/environment_map.rs`

## Relevant Feature Flags

default facade

## Run Or Check

```sh
cargo run -p scenix --example environment_map
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
