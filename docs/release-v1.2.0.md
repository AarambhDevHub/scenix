# Scenix v1.2.0 Renderer And Material Parity

Scenix `1.2.0` moves the renderer from preview-oriented material color paths to real renderer-owned GPU resources for textures, lights, environment descriptors, render targets, diagnostics, and lifecycle workflows.

## Highlights

- All workspace crates are bumped to `1.2.0`.
- `scenix-animato` now targets `animato = "1.5.0"`.
- `scenix-renderer` uploads `Texture2D`, `TextureCube`, and `Texture3D` resources with mip-aware layout validation and sampler conversion.
- PBR, physical, unlit, Lambert, toon, normal, and wireframe materials now feed the active renderer path through material uniforms and texture bind groups.
- Ambient, hemisphere, directional, point, spot, area, and light-probe data can be registered with the renderer.
- `EnvironmentMap`, renderer-owned render targets, render-to-texture, readback, resource diagnostics, and lifecycle APIs are available through additive APIs.
- WebGL fallback now prefers a real WebGL2 renderer path when WebGPU is unavailable, with WebGL1 kept as a reduced last-resort path.

## Install

```toml
[dependencies]
scenix = "1.2"
```

Renderer stack:

```toml
[dependencies]
scenix = { version = "1.2", features = ["renderer", "post"] }
```

Full optional stack:

```toml
[dependencies]
scenix = { version = "1.2", features = ["loader", "renderer", "post", "animato", "wasm"] }
```

## Code Example

```rust
use scenix::{
    Color, MaterialId, MeshId, PbrMaterial, PerspectiveCamera, Renderer, RendererConfig,
    Sampler, SceneGraph, SceneNode, Texture2D, TextureFormat, TextureId, Vec3, sphere_geometry,
};

# async fn run() -> Result<(), scenix::ScenixError> {
let mut renderer = Renderer::headless(RendererConfig::new(512, 512)).await?;

let albedo_id = TextureId::new(10);
let albedo = Texture2D::new(
    1,
    1,
    TextureFormat::Rgba8UnormSrgb,
    vec![255, 180, 80, 255],
)?;
renderer.register_texture2d(albedo_id, &albedo, Sampler::new())?;

let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);
renderer.register_mesh(mesh_id, &sphere_geometry(1.0, 48, 24))?;

let mut material = PbrMaterial::new()
    .albedo(Color::WHITE)
    .metallic_roughness(0.25, 0.4);
material.albedo_texture = Some(albedo_id);
renderer.register_pbr_material(material_id, &material)?;

let mut scene = SceneGraph::new();
scene.add(SceneNode::mesh("textured sphere", mesh_id, material_id));
scene.update_world_transforms();

let camera = PerspectiveCamera::new(50.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 3.5))
    .target(Vec3::ZERO);

let stats = renderer.render(&scene, &camera)?;
println!("draws={}, textures={}", stats.opaque_draws, renderer.diagnostics().textures);
# Ok(())
# }
```

## Migration Notes

- Existing v1 renderer code keeps compiling; new APIs are additive.
- `Renderer::register_texture2d` now uploads a real GPU texture instead of storing metadata only.
- Use `TextureId` for render targets in v1.2.0: create targets with `create_render_target`, render with `render_to_texture`, and read with `read_texture_pixel`.
- Enable the `animato` facade feature as before; the bridge now resolves to Animato 1.5.0.

## WebGPU And WebGL

- Browser rendering is WebGPU first. If WebGPU is unavailable or unsafe, `BrowserRenderer` falls back to WebGL2 and renders the generated scene through WebGL textures, material uniforms, directional/point lights, toon/physical approximations, post toggles, picking, and animation.
- WebGL2 reports `parity=full-fallback` through diagnostics and is the intended full browser fallback for v1.2.0 demos.
- WebGL1 remains a reduced last-resort fallback for old browsers and reports `parity=reduced-fallback`.

## Known Limitations

- v1.2.0 keeps GPU upload explicit; loaders still produce CPU-side assets.
- Physical material transmission and environment response are realtime approximations.
- Shadow support is designed for practical smoke scenes and editor previews; large production shadow systems remain future work.
- GPU tests still require a Vulkan-capable device or Mesa lavapipe.

## Links

- Website and demo: `https://aarambhdevhub.github.io/scenix/`
- Documentation: `https://docs.rs/scenix`
- Crates: `https://crates.io/crates/scenix`
