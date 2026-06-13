# Environment Map

## Purpose

Registers a cube texture and uses it as a renderer environment map.

## Source

`examples/environment_map.rs`

## Relevant Feature Flags

`renderer`

## Run Or Check

```sh
cargo run -p scenix --example environment_map --features renderer
```

## What To Look For

- The example should compile with the listed features.
- The example should report one rendered draw and one registered environment texture.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
