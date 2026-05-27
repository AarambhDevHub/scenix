# Render A Cube

## Goal

Render a generated cube through the optional `wgpu` renderer.

## Relevant Feature Flags

`renderer`

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```rust
use scenix::{PerspectiveCamera, Renderer, RendererConfig, Vec3};

# async fn run(scene: &scenix::SceneGraph) -> Result<(), scenix::ScenixError> {
let mut renderer = Renderer::headless(RendererConfig::new(512, 512)).await?;
let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
renderer.render(scene, &camera)?;
# Ok(())
# }
```

## Verify

Run `cargo run -p scenix --example hello_cube --features renderer`.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
