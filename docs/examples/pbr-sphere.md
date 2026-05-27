# PBR Sphere

## Purpose

Shows metallic-roughness material setup.

## Source

`examples/pbr_sphere.rs`

## Relevant Feature Flags

renderer

## Run Or Check

```sh
cargo run -p scenix --example pbr_sphere --features renderer
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
