# Render Target Capture

## Purpose

Renders a scene into a renderer-owned texture target and reads back one pixel.

## Source

`examples/render_target_capture.rs`

## Relevant Feature Flags

`renderer`

## Run Or Check

```sh
cargo run -p scenix --example render_target_capture --features renderer
```

## What To Look For

- The example should create a `TextureId` render target.
- It should render a cube into that target and print one RGBA pixel.

## Related Docs

- [Examples index](README.md)
- [Renderer](../concepts/renderer.md)
