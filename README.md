# scenix

> Modular Rust-native 3D scenes for native and WASM apps.

[![CI](https://github.com/AarambhDevHub/scenix/actions/workflows/ci.yml/badge.svg)](https://github.com/AarambhDevHub/scenix/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

scenix `1.8.0` is the current stable release. It adds the environment
crates — `scenix-terrain` (heightfields, chunked LOD, splat maps, collision,
chunk streaming), `scenix-sky` (procedural atmosphere, sun positions,
deterministic sky bakes, grounded skyboxes), and `scenix-water` (Gerstner
waves, foam, fresnel, underwater tinting, planar reflection helpers) — wired
through the scene schema, materials, textures, the renderer's `environment`
GPU draw path, browser capability notes, and facade feature flags.
v1.7.0's deterministic CPU particle systems keep their billboard GPU path,
v1.6.0's typed shader graphs, node materials, and post-node effect language
remain the shading backbone, and v1.5.0's cross-platform controls, input
stack, selection/transform primitives, reusable gizmos, inspector snapshots,
optional egui rendering, and on-demand GPU picking remain the interaction
backbone. CPU authoring stays lightweight by default while GPU, browser, and
UI paths remain opt-in.

## Install

Most applications start with the facade crate:

```toml
[dependencies]
scenix = "1"
```

Enable optional systems only when needed:

```toml
[dependencies]
scenix = { version = "1", features = ["loader"] }
scenix = { version = "1", features = ["renderer", "post"] }
scenix = { version = "1", features = ["animato"] }
scenix = { version = "1", features = ["nodes"] }
scenix = { version = "1", features = ["environment"] }
scenix = { version = "1", features = ["wasm"] }
scenix = { version = "1", features = ["interaction"] }
scenix = { version = "1", features = ["editor", "egui"] }
```

Focused crates can be used directly:

```toml
[dependencies]
scenix-math = "1"
scenix-core = "1"
scenix-input = "1"
scenix-scene = "1"
scenix-camera = "1"
scenix-mesh = "1"
scenix-material = "1"
scenix-nodes = "1"
scenix-particles = "1"
scenix-terrain = "1"
scenix-sky = "1"
scenix-water = "1"
scenix-light = "1"
scenix-texture = "1"
scenix-loader = "1"
scenix-renderer = "1"
scenix-post = "1"
scenix-raycaster = "1"
scenix-helpers = "1"
scenix-animato = "1"
scenix-wasm = "1"
```

For `no_std` CPU authoring:

```toml
[dependencies]
scenix-math = { version = "1", default-features = false, features = ["libm"] }
scenix-core = { version = "1", default-features = false }
scenix-input = { version = "1", default-features = false }
scenix-scene = { version = "1", default-features = false }
scenix-camera = { version = "1", default-features = false }
scenix-mesh = { version = "1", default-features = false }
scenix-material = { version = "1", default-features = false }
scenix-nodes = { version = "1", default-features = false }
scenix-particles = { version = "1", default-features = false }
scenix-light = { version = "1", default-features = false }
scenix-texture = { version = "1", default-features = false }
scenix-raycaster = { version = "1", default-features = false }
scenix-helpers = { version = "1", default-features = false }
```

`scenix-loader`, `scenix-renderer`, `scenix-post`, `scenix-animato`, and
`scenix-wasm` are optional `std` paths. `scenix-animato` targets Animato
`1.7.2`. `scenix-nodes` is a `no_std`-capable CPU crate; its renderer, post
stack, and browser preflight integrations are `std`/`wasm32` features of their
host crates.

## Feature Flags

| Feature | Default | Description |
| --- | --- | --- |
| `std` | yes | Standard-library support for CPU crates. |
| `scene`, `camera`, `mesh`, `material`, `light`, `texture` | yes | CPU authoring crates. |
| `raycaster`, `helpers` | yes | BVH picking and debug line helper data. |
| `interaction` | no | Convenience bundle for scene, camera, raycaster, and helpers. |
| `editor` | no | Inspector snapshots and editor-facing selection/transform data. |
| `egui` | no | Read-only egui inspector adapter; the host owns its window/render loop. |
| `loader` | no | Asset packages, asset manager, glTF/GLB extension metadata, OBJ/MTL, STL, image, KTX2, HDR/EXR loading, and exporters. |
| `renderer` | no | `wgpu` renderer with surface/headless targets. |
| `post` | no | Full-screen post-processing stack; use with `renderer`. |
| `nodes` | no | Typed shader graphs, node materials, and post node effects with WGSL/GLSL backends under the `scenix-nodes` crate; enables node-material rendering in `renderer` and browser preflight in `wasm` when those features are on. |
| `particles` | no | Deterministic CPU particle systems under the `scenix-particles` crate; enables the billboard GPU path (with optional compute simulation) in `renderer` and browser fallback plans in `wasm` when those features are on. |
| `terrain` | no | Heightfields, chunked LOD geometry, splat maps, ray colliders, and chunk streaming under the `scenix-terrain` crate. |
| `sky` | no | Procedural sky profiles, sun positions, baked environment maps, and grounded skyboxes under the `scenix-sky` crate. |
| `water` | no | Gerstner wave fields, displaced surface meshes, foam/fresnel/underwater math, and mirror helpers under the `scenix-water` crate. |
| `environment` | no | Bundles `terrain`, `sky`, and `water`; enables the environment WGSL pipelines (terrain splats, sky dome, water surface) in `renderer` when that feature is on. |
| `animato` | no | Animato bridge: procedural tween/spring tracks **and** the clip-based Animation Runtime (clips, actions, mixer, loop modes, crossfade, additive blending, markers/events, light/morph targets, retargeting, CPU skinning). Targets Animato `1.7.2`. |
| `wasm` | no | Browser canvas wrapper with WebGPU first, WebGL2 full fallback, WebGL1 reduced fallback, and generated demo scene. |
| `serde` | no | Serialization support where the focused crate supports it. |

## Quick Start

### Controls and interaction

```rust
use scenix::{
    ArcballController, InputState, PerspectiveCamera, PointerButton, Vec2, Vec3,
    ViewportMetrics,
};

let mut input = InputState::new(ViewportMetrics::new(Vec2::new(1280.0, 720.0), 2.0));
input.on_pointer_down(PointerButton::Left);
input.on_pointer_move(Vec2::new(24.0, 12.0));
input.on_scroll(-0.25);

let mut controls = ArcballController::new(Vec3::ZERO, 5.0);
let mut camera = PerspectiveCamera::default();
controls.update_from_input(&input, 1.0 / 60.0);
controls.apply_to_perspective(&mut camera);
input.end_frame();
```

`InputState` also accepts fixed-capacity touch contacts, two-finger gestures,
four standard gamepads, and pointer-lock relative motion without allocating in
the event/control hot path.

### Scene authoring and animation

```rust
use std::collections::BTreeMap;
use scenix::{
    CameraId, CameraStores, Geometry, MaterialId, MeshId, NodeAnimationTarget, NodeAnimator,
    PbrMaterial, PerspectiveCamera, SceneGraph, SceneNode, ScenixAnimationDriver, Vec3,
    Vec3Track, box_geometry,
};

# fn run() -> Result<(), scenix::ValidationError> {
let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);

let mut meshes = BTreeMap::<MeshId, Geometry>::new();
meshes.insert(mesh_id, box_geometry(1.0, 1.0, 1.0, 1, 1, 1));

let mut scene = SceneGraph::new();
let cube = scene.add(SceneNode::mesh("cube", mesh_id, material_id));

let mut driver = ScenixAnimationDriver::new();
driver.add_node(NodeAnimator::new(
    cube,
    NodeAnimationTarget::Translation(Vec3Track::tween(
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
        0.5,
    )),
));

let camera_id = CameraId::new(1);
let mut perspective = BTreeMap::from([(
    camera_id,
    PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 0.0, 4.0))
        .target(Vec3::ZERO),
)]);
let mut orthographic = BTreeMap::new();
let mut cameras = CameraStores {
    perspective: &mut perspective,
    orthographic: &mut orthographic,
};
let mut materials = BTreeMap::from([(material_id, PbrMaterial::new())]);
let mut skeletons = Vec::new();

driver.tick(0.5, &mut scene, &mut cameras, &mut materials, &mut skeletons)?;
scene.update_world_transforms();
# Ok(())
# }
```

### Headless Rendering

```rust
use scenix::{Renderer, RendererConfig, PerspectiveCamera, Vec3};

# async fn run(scene: &scenix::SceneGraph) -> Result<(), scenix::ScenixError> {
let mut renderer = Renderer::headless(RendererConfig::new(256, 256)).await?;
let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 5.0))
    .target(Vec3::ZERO);
let stats = renderer.render(scene, &camera)?;
assert!(stats.frame_index > 0);
# Ok(())
# }
```

### Loading Assets

```rust
use scenix::{AssetManager, RendererAssetExt};

# async fn run() -> Result<(), scenix::ScenixError> {
let mut manager = AssetManager::new();
let package = manager.load_file("scene.glb")?;

let mut renderer = scenix::Renderer::headless(scenix::RendererConfig::new(512, 512)).await?;
let uploaded = renderer.register_asset_package(&package)?;
println!("meshes={}, textures={}", uploaded.meshes, uploaded.textures);
# Ok(())
# }
```

### Animation Runtime

The v1.4.0 `AnimationMixer` plays clip-based animation on top of Animato:

```rust
use scenix::{
    AnimationClip, AnimationMixer, ClipChannel, ClipTrack, KeyframeInterpolation,
    KeyframeVec3, LoopMode, NodeProperty, PropertyBinding, SceneGraph, SceneNode, Vec3,
};

let mut scene = SceneGraph::new();
let node = scene.add(SceneNode::new("mover"));

let clip = AnimationClip::empty("move").with_channel(ClipChannel {
    binding: PropertyBinding::Node { node_id: node, property: NodeProperty::Translation },
    track: ClipTrack::Vec3(KeyframeVec3::new(
        vec![0.0, 1.0],
        vec![Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0)],
        KeyframeInterpolation::Linear,
    )),
});

let mut mixer = AnimationMixer::new();
let clip_index = mixer.add_clip(clip);
let action = mixer.add_action(clip_index);
mixer.action_mut(action).unwrap().set_loop_mode(LoopMode::REPEAT);
mixer.action_mut(action).unwrap().play(0.0);

// each frame:
// mixer.tick(dt, &mut scene, &mut cameras, &mut materials,
//            &mut lights, &mut skeletons, &mut morphs)
```

### Shader nodes and node materials

v1.6.0 adds the optional `scenix-nodes` crate. Describe shading as a typed
graph, validate it without a GPU, and generate WGSL (renderer contract) or
GLSL ES 3.00 (WebGL tooling) from the same graph:

```rust
use scenix::{
    generate_material_wgsl, LightingStyle, Material, NodeInput, NodeMaterial, ShaderGraph,
    ShaderNodeKind, UniformInput, WgslOptions,
};

// Base color pulses over time, then feeds a Lambert lighting term.
let mut graph = ShaderGraph::new();
let base = graph.push(ShaderNodeKind::Uniform(UniformInput::BaseColor));
let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
let pulse = graph.add(
    ShaderNodeKind::Multiply,
    vec![NodeInput::Node(base), NodeInput::Node(time)],
);
let lit = graph.chain(ShaderNodeKind::Lighting(LightingStyle::Lambert), pulse);
graph.add_fragment_output(lit, None);

let material = NodeMaterial::new("pulse", graph);
assert!(material.validate().is_empty());
assert!(material.feature_bits() & scenix::FEATURE_NODE_GRAPH != 0);

let wgsl = generate_material_wgsl(&material, &WgslOptions::default()).unwrap();
// wgsl.source() matches the renderer bind contract:
// group 0 frame, group 1 object, group 2 material, group 3 lights.
```

With the `renderer` feature, `Renderer::register_node_material(id, &material)`
shades draws through the generated module; graphs that cannot resolve carry a
specific `NodeDiagnostic` instead of crashing. With the `wasm` feature,
`scenix_wasm::compile_node_material_glsl(json)` runs the same preflight in the
browser before anything is uploaded.

### Particles

```rust
use scenix::{
    ColorOverLifetime, ColorCurve, Drag, EmissionShape, Gravity, ParticleEmitter,
    ParticleSystem, Vec3, SizeOverLifetime,
};

let emitter = ParticleEmitter::new()
    .with_rate(240.0)
    .with_lifetime(0.9, 1.6)
    .with_size(0.2)
    .with_speed(4.0, 7.0)
    .at(Vec3::new(0.0, 0.0, 0.0))
    .toward(Vec3::new(0.0, 1.0, 0.0))
    .with_shape(EmissionShape::Cone { radius: 0.15, height: 0.35 })
    .with_modifier(Gravity::new(Vec3::new(0.0, -12.0, 0.0)))
    .with_modifier(Drag::new(0.15))
    .with_modifier(ColorOverLifetime::new(ColorCurve::from_to(
        [1.0, 0.85, 0.35, 1.0],
        [0.9, 0.15, 0.0, 0.0],
    )))
    .with_modifier(SizeOverLifetime::from_to(1.15, 0.05));

// One deterministic CPU system drives the entire loop.
let mut system = ParticleSystem::new(emitter, 1_024, 2021018);
let batch = system.update(1.0 / 60.0);
assert!(batch.len() <= 1_024);

// With `renderer` + `particles`, `Renderer::register_particles` +
// `Renderer::update_particles` draw the batch as camera-facing billboards;
// `register_particle_compute` moves the whole simulation onto the GPU.
```

### Raycasting


```rust
use std::collections::BTreeMap;
use scenix::{
    Geometry, MaterialId, MeshId, PerspectiveCamera, Raycaster, SceneGraph, SceneNode,
    Vec2, Vec3, box_geometry,
};

let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);
let meshes = BTreeMap::<MeshId, Geometry>::from([(
    mesh_id,
    box_geometry(1.0, 1.0, 1.0, 1, 1, 1),
)]);

let mut scene = SceneGraph::new();
scene.add(SceneNode::mesh("cube", mesh_id, material_id));
scene.update_world_transforms();

let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
    .position(Vec3::new(0.0, 0.0, 4.0))
    .target(Vec3::ZERO);
let ray = Raycaster::from_camera_ndc(&camera, Vec2::ZERO);

let mut raycaster = Raycaster::new();
raycaster.build_bvh(&scene, &meshes).unwrap();
assert!(raycaster.cast_ray(ray, &scene, &meshes).is_some());
```

## Workspace

| Crate | Role |
| --- | --- |
| `scenix` | Facade crate with stable v1 feature flags. |
| `scenix-math` | `no_std` vectors, matrices, quaternions, transforms, rays, and bounds. |
| `scenix-core` | IDs, colors, errors, and shared traits. |
| `scenix-input` | Pointer, keyboard, touch, gesture, gamepad, pointer-lock, and viewport state. |
| `scenix-scene` | Scene graph plus editor metadata, layers, snapping, and selection state. |
| `scenix-camera` | Cameras plus orbit, fly, arcball, trackball, map, first-person, and pointer-lock controls. |
| `scenix-mesh` | Geometry buffers, primitives, instancing, batching, and morph targets. |
| `scenix-material` | GPU-free material descriptions and pipeline keys. |
| `scenix-nodes` | Optional typed shader graphs, node materials, and post node effects with WGSL/GLSL codegen. |
| `scenix-particles` | Optional deterministic CPU particle systems with emitter/curve modules and GPU billboard/compute integration. |
| `scenix-terrain` | Optional CPU heightfields, chunked LOD geometry, splat maps, ray colliders, and chunk streaming. |
| `scenix-sky` | Optional procedural sky profiles, sun positions, baked environment maps, and grounded skyboxes. |
| `scenix-water` | Optional Gerstner wave fields, displaced surface meshes, foam/fresnel/underwater math, and mirror helpers. |
| `scenix-light` | Lights, shadow settings, and light probes. |
| `scenix-texture` | CPU textures, samplers, atlases, video updates, and mipmaps. |
| `scenix-loader` | Optional CPU asset packages, asset manager, importers, exporters, diagnostics, and cache. |
| `scenix-renderer` | Optional `wgpu` renderer with on-demand ID/normal/depth editor picking. |
| `scenix-post` | Optional `wgpu` post-processing effects. |
| `scenix-raycaster` | BVH ray picking, marquee selection, drag planes, and transform interactions. |
| `scenix-helpers` | Debug lines, reusable gizmos, selection visuals, snap grids, and optional egui. |
| `scenix-animato` | Optional Animato bridge. |
| `scenix-wasm` | Optional browser canvas wrapper with WebGPU and WebGL paths. |

## Examples

The facade crate registers the example set from [ARCHITECTURE.md](./ARCHITECTURE.md):

```sh
cargo run -p scenix --example hello_cube --features renderer
cargo run -p scenix --example pbr_sphere --features renderer
cargo run -p scenix --example physical_material --features renderer
cargo run -p scenix --example toon_shading --features renderer
cargo run -p scenix --example gltf_scene --features "loader renderer"
cargo run -p scenix --example asset_pipeline --features "loader renderer"
cargo run -p scenix --example asset_manager --features loader
cargo run -p scenix --example export_scene --features loader
cargo run -p scenix --example animation_import --features loader
cargo run -p scenix --example compressed_assets --features loader
cargo run -p scenix --example shadow_demo --features renderer
cargo run -p scenix --example raycasting
cargo run -p scenix --example post_processing --features "renderer post"
cargo run -p scenix --example instanced_mesh
cargo run -p scenix --example animato_integration --features animato
cargo run -p scenix --example animation_runtime --features "animato scene"
cargo run -p scenix --example animation_mixer --features "animato scene material light"
cargo run -p scenix --example skeleton_skinning --features "mesh animato"
cargo run -p scenix --example animation_events --features "animato scene light"
cargo run -p scenix --example orbit_camera
cargo run -p scenix --example lod_demo
cargo run -p scenix --example morph_targets
cargo run -p scenix --example fog_demo
cargo run -p scenix --example helpers_demo
cargo run -p scenix --example controls_showcase
cargo run -p scenix --example selection_and_drag
cargo run -p scenix --example transform_gizmo
cargo run -p scenix --example editor_inspector --features egui
cargo run -p scenix --example renderer_picking --features renderer
cargo run -p scenix --example nodes_material --features nodes
cargo run -p scenix --example nodes_wgsl --features nodes
cargo run -p scenix --example node_post_effect --features nodes
cargo run -p scenix --example particles_basic --features particles
cargo run -p scenix --example sprite_particles
cargo run -p scenix --example environment_map --features renderer
cargo run -p scenix --example render_target_capture --features renderer
cargo run -p scenix --example terrain_lod --features terrain
cargo run -p scenix --example sky_sun --features sky
cargo run -p scenix --example water_fresnel --features water,camera
cargo run -p scenix --example environment_demo --features "environment renderer scene camera material"
```

The browser example lives in `examples/wasm_viewer`:

```sh
rustup target add wasm32-unknown-unknown
cargo check --manifest-path examples/wasm_viewer/Cargo.toml --target wasm32-unknown-unknown
```

## Website

The static website is a standalone Leptos CSR app in `website/`. It is intentionally outside the main workspace so website dependencies do not affect normal library users.

```sh
cd website
trunk serve
trunk build --release --public-url /scenix/
```

GitHub Pages deployment is handled by `.github/workflows/pages.yml`, which builds `website/dist` with Trunk and deploys it at `/scenix/`. The demo uses `scenix-wasm`: it tries WebGPU where safe, falls back to a WebGL2 renderer with the same generated scene/material/light controls when WebGPU is unavailable, uses reduced WebGL1 only as a last GPU path, and only uses the Canvas2D preview when both GPU paths are unavailable.

## Development Checks

```sh
cargo fmt --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --all-features
cargo test -p scenix-math -p scenix-core -p scenix-input -p scenix-scene -p scenix-camera -p scenix-mesh -p scenix-material -p scenix-light -p scenix-texture -p scenix-raycaster -p scenix-helpers -p scenix-animato --no-default-features
cargo test -p scenix-loader --all-features
cargo test -p scenix-raycaster -p scenix-helpers --all-features
cargo test -p scenix-animato --all-features
cargo test -p scenix-terrain -p scenix-sky -p scenix-water --all-features
cargo test -p scenix --test scenix_v15 --all-features
cargo test -p scenix --test scenix_v18 --all-features
cargo check -p scenix --no-default-features --features interaction
cargo check -p scenix --no-default-features --features editor
cargo check -p scenix --no-default-features --features egui
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
cargo check --manifest-path examples/wasm_viewer/Cargo.toml --target wasm32-unknown-unknown
SCENIX_RUN_GPU_TESTS=1 WGPU_BACKEND=vulkan cargo test -p scenix-renderer -p scenix-post --all-features
SCENIX_RUN_GPU_TESTS=1 WGPU_BACKEND=vulkan cargo test -p scenix-renderer --features environment
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo bench --workspace --no-run
cargo llvm-cov --workspace --all-features
```

## Documentation

- [Developer docs](./docs/README.md)
- [Architecture](./ARCHITECTURE.md)
- [Roadmap](./ROADMAP.md)
- [Changelog](./CHANGELOG.md)
- [Getting started](./docs/getting-started.md)
- [Installation](./docs/installation.md)
- [Quick start](./docs/quick-start.md)
- [Concepts](./docs/concepts/README.md)
- [Guides](./docs/guides/create-your-first-scene.md)
- [API reference](./docs/api/facade-crate.md)
- [Examples](./docs/examples/README.md)
- [Recipes](./docs/recipes/README.md)
- [Performance](./docs/performance/README.md)
- [Deployment](./docs/deployment/README.md)
- [Migration](./docs/migration/from-0.9-to-1.0.md)
- [Reference](./docs/reference/feature-matrix.md)
- [v1.8.0 release notes](./.github/release-notes/1.8.0.md)
- [v1.7.0 release notes](./.github/release-notes/1.7.0.md)
- [v1.6.0 release notes](./docs/release-v1.6.0.md)
- [v1.5.0 release notes](./.github/release-notes/1.5.0.md)
- [v1.4.0 release notes](./.github/release-notes/1.4.0.md)
- [v1.3.0 release notes](./.github/release-notes/v1.3.0.md)

## Known Limitations

- The renderer now uploads material textures, light data, environment descriptors, and render targets through real GPU resources. Advanced physical shading is a pragmatic realtime path, not an offline film renderer.
- Loader APIs produce CPU-side scenix data; `RendererAssetExt` is a convenience bridge that still uploads into renderer-owned GPU resources explicitly.
- Draco and meshopt compressed glTF assets currently produce explicit diagnostics unless preprocessed by an external converter.
- WebGL2 is the full browser fallback for the generated renderer scene when WebGPU is unavailable. WebGL1 remains a reduced last-resort fallback for older browsers.
- The website demo does not vendor large model assets.
- GPU tests require a Vulkan-capable device or Mesa lavapipe.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license
