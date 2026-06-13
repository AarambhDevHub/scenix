# `scenix-renderer`

## Role

Optional `wgpu` renderer, GPU resource stores, material texture upload, light uniforms, render targets, pipeline cache, frame stats, shadows, and headless rendering.

## Dependency Weight

Heavy `std` path; enable `renderer` on facade.

## Install

```toml
[dependencies]
scenix-renderer = "1"
```

## Key Public API

Renderer, RendererConfig, FrameStats, RendererDiagnostics, ResourceStats, EnvironmentMap, RenderTargetDescriptor, GpuScene, GpuMaterial, PipelineCache, GBuffer, ShadowMapAtlas

## Common Use

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

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
