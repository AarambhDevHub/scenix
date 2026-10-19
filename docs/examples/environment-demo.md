# Environment Demo

## Purpose

Registers terrain, sky, and water environment slots with the headless renderer, draws them through scene nodes, and reports the per-frame environment stats.

## Source

`examples/environment_demo.rs`

## Relevant Feature Flags

facade `environment`, `renderer`, `scene`, `camera`, `material` features

## Run Or Check

```sh
cargo run -p scenix --example environment_demo --features environment,renderer,scene,camera,material
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Scenix v1.8.0 release](../release-v1.8.0.md)
