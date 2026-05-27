# Renderer Performance

## Goal

Register resources outside the hot frame path, keep pipeline cache warm, and avoid per-frame material churn.

## Measure First

Use focused commands and compare one change at a time. Avoid enabling heavy features globally when only one binary or example needs them.

## Command Or Pattern

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

## Practical Checks

- Keep CPU-only crates lightweight.
- Avoid rebuilding data structures every frame unless inputs changed.
- Separate load, registration, update, and render costs when profiling.
