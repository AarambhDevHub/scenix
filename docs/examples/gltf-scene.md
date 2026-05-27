# glTF Scene

## Purpose

Loads a generated glTF fixture and registers loaded data for rendering.

## Source

`examples/gltf_scene.rs`

## Relevant Feature Flags

loader, renderer

## Run Or Check

```sh
cargo run -p scenix --example gltf_scene --features "loader renderer"
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
