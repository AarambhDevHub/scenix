# Renderer

## Purpose

Use the optional `wgpu` renderer for surface and headless rendering.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Enable `renderer`.

## Key Rules

- The renderer owns device, queue, surfaces, buffers, textures, and pipeline caches.
- SceneGraph stores IDs; renderer resource registration maps IDs to GPU resources.
- GPU tests are gated with `SCENIX_RUN_GPU_TESTS=1`.


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

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
