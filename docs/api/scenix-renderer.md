# `scenix-renderer`

## Role

Optional `wgpu` renderer, GPU resource stores, material texture upload, light
uniforms, render targets, pipeline cache, frame stats, shadows, headless
rendering, and on-demand object-ID/normal/depth editor picking.

## Dependency Weight

Heavy `std` path; enable `renderer` on facade.

## Install

```toml
[dependencies]
scenix-renderer = "1"
```

## Key Public API

`Renderer`, `RendererConfig`, `FrameStats`, `RendererDiagnostics`,
`ResourceStats`, `EditorPickRequest`, `EditorPickResult`, `EditorBufferStats`,
`EnvironmentMap`, `RenderTargetDescriptor`, `GpuScene`, `GpuMaterial`,
`PipelineCache`, `GBuffer`, and `ShadowMapAtlas`.

## Common Use

```rust
use scenix::{PerspectiveCamera, Renderer, RendererConfig, Vec3};

# async fn run(scene: &scenix::SceneGraph) -> Result<(), scenix::ScenixError> {
let mut renderer = Renderer::headless(RendererConfig::new(512, 512)).await?;
let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
renderer.render(scene, &camera)?;
let picked = renderer.pick(scene, &camera, scenix::EditorPickRequest::new(256, 256))?;
# let _ = picked;
# Ok(())
# }
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Renderer picking example](../examples/renderer-picking.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
