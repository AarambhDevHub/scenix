# Hello Cube

## Purpose

Creates a generated cube scene and renders it through the renderer.

## Source

`examples/hello_cube.rs`

## Relevant Feature Flags

renderer

## Run Or Check

```sh
cargo run -p scenix --example hello_cube --features renderer
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
