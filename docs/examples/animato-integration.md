# Animato Integration

## Purpose

Ticks node, camera, and material animation through the Animato bridge.

## Source

`examples/animato_integration.rs`

## Relevant Feature Flags

animato

## Run Or Check

```sh
cargo run -p scenix --example animato_integration --features animato
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
