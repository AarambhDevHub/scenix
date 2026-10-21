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

## Added In

v1.7.0 adds the particle surface (behind `particles`): `register_particles`, `update_particles` (instanced 64-byte billboards), `register_particle_compute` + `simulate_particles_compute` (deterministic GPU simulation), and `read_particle_pool`/`read_particle_ring` readbacks; see [scenix-particles](scenix-particles.md).

v1.8.0 adds the environment surface (behind `environment`): `register_environment`/`unregister_environment` with `EnvironmentMaterial::{Terrain, Sky, Water}`, the `terrain_splat`/`sky_dome`/`water_surface` WGSL pipelines, sky-before-opaque and water-after-transparent draw ordering, and `scene_environments`/`visible_environments` in `CullingStats` and `FrameStats`; see [scenix-terrain](scenix-terrain.md), [scenix-sky](scenix-sky.md), and [scenix-water](scenix-water.md).

v1.9.0 changes nothing in the GPU path: the WGSL pipelines are untouched, and the culling pass gains a no-op arm for `NodeKind::{XrAnchor, AudioSource, PhysicsBody}` so bridge rider nodes traverse without submitting draws.


## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Renderer picking example](../examples/renderer-picking.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
