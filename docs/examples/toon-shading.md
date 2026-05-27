# Toon Shading

## Purpose

Shows toon material registration and preview rendering.

## Source

`examples/toon_shading.rs`

## Relevant Feature Flags

renderer

## Run Or Check

```sh
cargo run -p scenix --example toon_shading --features renderer
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
