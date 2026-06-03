# scenix — Project Roadmap

> *Italian: scenix — scene, the stage on which everything appears.*
> A professional-grade, renderer-agnostic 3D scene library for Rust.

This roadmap tracks the completed path from `v0.1.0` through `v1.1.0` and the post-1.0 ideas that may become future `v1.x` releases.
Each milestone is a working, published crate — not a draft. Nothing ships without tests, docs, and benchmarks.

---

## Status Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Complete |
| 🔄 | In progress |
| 📋 | Planned |
| 🔮 | Future / post-1.0 |

---

## Release Overview

| Version | Name | Focus | Status |
|---------|------|-------|--------|
| `v0.1.0` | Foundation | Math, core traits, IDs, color, error types | ✅ |
| `v0.2.0` | Scene Graph | Scene node tree, transforms, traversal, fog, LOD | ✅ |
| `v0.3.0` | Geometry | Mesh, all primitives, morph targets, instanced/batched mesh | ✅ |
| `v0.4.0` | Materials & Lights | Material trait, PBR, physical, toon, all light types | ✅ |
| `v0.5.0` | Textures & Camera | Texture loading, sampler, atlas, camera types, controllers | ✅ |
| `v0.6.0` | Renderer | wgpu pipeline, deferred+forward rendering, shadow maps | ✅ |
| `v0.7.0` | Loaders & Post | GLTF/OBJ/STL loaders, post-processing stack | ✅ |
| `v0.8.0` | Raycasting & Helpers | BVH raycaster, debug helpers, input abstraction | ✅ |
| `v0.9.0` | Integration | animato bridge, WASM browser support, framework compat | ✅ |
| `v1.0.0` | Stable | API freeze, full docs, examples, all CI green | ✅ |
| `v1.1.0` | Browser fallback | WebGPU-to-WebGL browser fallback and updated release automation | ✅ |
| `v1.2.0` | Renderer parity | Real material GPU paths, texture binding, lights, shadows, IBL, render targets | 📋 |
| `v1.3.0` | Asset pipeline | glTF extensions, animation import, compression, extra loaders, exporters, asset manager | 📋 |
| `v1.4.0` | Animation runtime | Clip/mixer/action layer, skeletal animation, morph playback, retargeting helpers | 📋 |
| `v1.5.0` | Interaction tools | Transform/drag/pointer-lock controls, selection helpers, editor primitives | 📋 |
| `v1.6.0` | Shader nodes | New optional shader graph and node material crate | 🔮 |
| `v1.7.0` | Particles | New optional CPU/GPU particle crate | 🔮 |
| `v1.8.0` | Environment systems | New optional terrain, sky, and water crates | 🔮 |
| `v1.9.0` | Runtime bridges | New optional XR, audio, and physics crates | 🔮 |
| `v1.10.0` | Editor tooling | New optional editor and UI overlay crates | 🔮 |
| `v1.x+` | Advanced rendering | Post effects, geometry extras, modifiers, GPU-driven rendering, realtime GI | 🔮 |

---

## v0.1.0 — Foundation

**Goal:** The smallest useful version of scenix. A developer can create 3D math types, transforms, and use the core trait system. No GPU required.

### Crates shipped

- `scenix-math` `v0.1.0`
- `scenix-core` `v0.1.0`
- `scenix-input` `v0.1.0`
- `scenix` `v0.1.0` (facade — math + core + input only)

### Deliverables

**`scenix-math`**
- [x] `Vec2` — `new`, `dot`, `length`, `normalize`, `lerp`, `angle_between`
- [x] `Vec3` — `new`, `dot`, `cross`, `length`, `normalize`, `lerp`, `reflect`, `angle_between`
- [x] `Vec4` — `new`, `dot`, `length`, `normalize`, `lerp`
- [x] `Mat3` — `identity`, `from_mat4`, `determinant`, `inverse`, `transpose`
- [x] `Mat4` — `identity`, `look_at`, `perspective`, `orthographic`, `inverse`, `transpose`, `mul`
- [x] `Quat` — `identity`, `from_axis_angle`, `from_euler_xyz`, `slerp`, `normalize`, `inverse`, `mul`
- [x] `Euler` — `new`, `from_quat`, `from_mat4`, `to_quat`, 6 rotation orders
- [x] `Transform` — `IDENTITY`, `to_mat4`, `mul_transform`, `inverse`, `forward`, `right`, `up`
- [x] `Ray3` — `at`, `intersect_aabb`, `intersect_sphere`, `intersect_triangle`
- [x] `Aabb` — `from_points`, `center`, `half_extents`, `contains_point`, `intersects_aabb`, `merge`, `surface_area`
- [x] `Plane` — `from_normal_and_point`, `from_three_points`, `signed_distance`, `intersect_ray`
- [x] `Spherical` — `from_vec3`, `to_vec3`, `clamp_phi`
- [x] `Cylindrical` — `from_vec3`, `to_vec3`
- [x] `no_std` compile gate with `libm` feature for trig
- [x] `serde` feature for all types
- [x] `approx` feature for `AbsDiffEq` impls
- [x] Full doc comments on every public item
- [x] Test: `Mat4::perspective` produces correct frustum
- [x] Test: `Quat::slerp` interpolates correctly at t=0, t=0.5, t=1
- [x] Test: `Transform::to_mat4` round-trips with decompose
- [x] Test: `Ray3::intersect_triangle` Möller–Trumbore correctness

**`scenix-core`**
- [x] `NodeId`, `MeshId`, `MaterialId`, `TextureId`, `LightId` — `Copy + Hash + Eq` newtypes over `u64`
- [x] `Renderable` trait — `fn render_order() -> u32`
- [x] `Bounded` trait — `fn aabb() -> Aabb`, `fn bounding_sphere() -> (Vec3, f32)`
- [x] `GpuUpload` trait (behind `gpu` feature) — `type GpuData: bytemuck::Pod`, `fn to_gpu()`
- [x] `Named` trait — `fn name()`, `fn set_name()`
- [x] `Color` struct — RGBA f32, `rgb()`, `rgba()`, `hex()`, `to_linear()`, `to_srgb()`
- [x] `ColorSpace` enum — `Linear`, `Srgb`
- [x] `ScenixError` enum — `LoadError`, `GpuError`, `ValidationError`
- [x] `no_std` compatible
- [x] Tests: Color hex parsing, sRGB ↔ linear conversion correctness

**`scenix-input`**
- [x] `PointerState` — position, delta, buttons bitmask
- [x] `KeyboardState` — `is_pressed()`, `on_key_down()`, `on_key_up()`
- [x] `KeyCode` enum — WASD, arrows, space, shift, ctrl
- [x] `Modifiers` struct — shift, ctrl, alt, meta
- [x] Tests: key press/release state tracking

**Documentation & Infrastructure**
- [x] `README.md` with installation, quick-start, feature table
- [x] `ARCHITECTURE.md`
- [x] `ROADMAP.md` (this file)
- [x] `CONTRIBUTING.md`
- [x] `CHANGELOG.md` with `## [0.1.0]` entry
- [x] `LICENSE-MIT` and `LICENSE-APACHE`
- [x] `.github/workflows/ci.yml` — test, clippy, fmt, docs, no_std
- [x] `.github/workflows/publish.yml` — dep-ordered crates.io publish
- [x] `benches/math_bench.rs` — Mat4 multiply, Quat slerp, AABB intersection

---

## v0.2.0 — Scene Graph

**Goal:** A working scene graph with parent-child hierarchy, transform propagation, and traversal. A developer can build a node tree and compute world transforms.

### Crates shipped

- `scenix-scene` `v0.2.0` (new)
- All previous crates bumped to `v0.2.0`

### Deliverables

**`scenix-scene`**
- [x] `SceneGraph` — `SlotMap`-backed node storage, root node management
- [x] `SceneNode` — `name`, `transform`, `visible`, `layer`, `NodeKind`
- [x] `NodeKind` enum — `Empty`, `Group`, `Mesh`, `Light`, `Camera`, `Sprite`
- [x] `graph.add(node) -> NodeId`
- [x] `graph.add_child(parent, node) -> Result<NodeId, ValidationError>`
- [x] `graph.remove(id)` — removes node and all children
- [x] `graph.get(id) -> Option<&SceneNode>`, `graph.get_mut(id)`
- [x] `graph.parent(id)`, `graph.children(id)`
- [x] `graph.find_by_name(name) -> Option<NodeId>`
- [x] Dirty-flag transform propagation — `graph.update_world_transforms()`
- [x] `graph.world_matrix(id) -> Option<Mat4>`
- [x] `graph.iter_depth_first()`, `graph.iter_breadth_first()`
- [x] `Fog` — `Fog::Linear { near, far, color }`, `Fog::Exponential { density, color }`
- [x] `LodGroup` — sorted `(max_distance, MeshId)` levels, `fn select(distance: f32) -> Option<MeshId>`
- [x] `Sprite` — `width`, `height`, `texture_id`, billboard facing mode
- [x] Tests: parent-child hierarchy, world transform correctness, remove cascades
- [x] Tests: depth-first traversal order, dirty-flag correctness
- [x] `benches/scene_graph_bench.rs` — 10K node traversal + transform propagation

---

## v0.3.0 — Geometry

**Goal:** All geometry types, vertex buffer management, and mesh primitives. A developer can generate any standard 3D shape.

### Crates shipped

- `scenix-mesh` `v0.3.0` (new)
- All previous crates bumped to `v0.3.0`

### Deliverables

**`scenix-mesh`**
- [x] `Geometry` struct — positions, normals, uvs, uvs2, colors, indices, tangents
- [x] `geometry.compute_normals()` — face-weighted vertex normals
- [x] `geometry.compute_tangents()` — tangent generation with handedness
- [x] `geometry.aabb()`, `geometry.bounding_sphere()`
- [x] `geometry.merge(other)` — combine geometries
- [x] `Mesh` struct — `Geometry` + `MaterialId`
- [x] `BufferLayout`, `VertexAttribute`, `IndexFormat`
- [x] `MorphTarget` — `name`, `positions_delta`, `normals_delta`, `weight`
- [x] `InstancedMesh` — `mesh_id`, `material_id`, `transforms: Vec<Mat4>`, `set_transform_at()`
- [x] `BatchedMesh` — multiple geometries in single draw call

**Primitives (all return `Geometry`)**
- [x] `box_geometry(w, h, d, w_seg, h_seg, d_seg)`
- [x] `sphere_geometry(radius, width_seg, height_seg)`
- [x] `plane_geometry(w, h, w_seg, h_seg)`
- [x] `cylinder_geometry(top_r, bottom_r, height, radial_seg, height_seg, open_ended)`
- [x] `cone_geometry(radius, height, radial_seg, height_seg)`
- [x] `capsule_geometry(radius, height, cap_seg, radial_seg)`
- [x] `torus_geometry(radius, tube, radial_seg, tubular_seg)`
- [x] `torus_knot_geometry(radius, tube, tubular_seg, radial_seg, p, q)`
- [x] `icosphere_geometry(radius, subdivisions)`
- [x] `circle_geometry(radius, segments, theta_start, theta_length)`
- [x] `ring_geometry(inner_r, outer_r, theta_seg, phi_seg)`
- [x] `lathe_geometry(points, segments, phi_start, phi_length)`
- [x] `extrude_geometry(shape, depth, bevel_thickness, bevel_size, bevel_segments)`
- [x] `tube_geometry(path, tubular_seg, radius, radial_seg, closed)`
- [x] `shape_geometry(shape)` — 2D shape triangulation

- [x] Tests: every primitive produces valid normals (dot(n, face_normal) > 0)
- [x] Tests: every primitive has correct vertex count for given segment params
- [x] Tests: UV coordinates in [0, 1] range
- [x] `benches/mesh_gen_bench.rs` — primitive generation throughput

---

## v0.4.0 — Materials & Lights

**Goal:** The material system and all light types. A developer can create PBR, toon, and custom shader materials, and illuminate scenes with all standard light types.

### Crates shipped

- `scenix-material` `v0.4.0` (new)
- `scenix-light` `v0.4.0` (new)

### Deliverables

**`scenix-material`**
- [x] `Material` trait — `pipeline_key()`, `is_transparent()`, `double_sided()`, `alpha_cutoff()`
- [x] `PipelineKey` struct — determines which shader pipeline to use
- [x] `AlphaMode` enum — `Opaque`, `Mask(f32)`, `Blend`
- [x] `PbrMaterial` — albedo, metallic, roughness, normal/ao/emissive textures, alpha mode
- [x] `PhysicalMaterial` — clearcoat, sheen, transmission, thickness, IOR, iridescence
- [x] `UnlitMaterial` — color + optional texture, no lighting
- [x] `LambertMaterial` — diffuse-only, faster than PBR
- [x] `ToonMaterial` — gradient map, discrete steps, outline
- [x] `NormalMaterial` — debug normals → RGB
- [x] `WireframeMaterial` — wireframe overlay
- [x] `DepthMaterial` — shadow pass depth output
- [x] `LineMaterial` — width, dash pattern, color
- [x] `PointsMaterial` — point size, attenuation
- [x] `ShaderMaterial` — custom WGSL vertex/fragment, raw uniforms
- [x] Tests: `PipelineKey` uniqueness for different material configs

**`scenix-light`**
- [x] `AmbientLight` — color, intensity
- [x] `DirectionalLight` — direction, color, intensity, optional `ShadowConfig`
- [x] `PointLight` — color, intensity, range, decay, optional `ShadowConfig`
- [x] `SpotLight` — color, intensity, range, angle, penumbra, optional `ShadowConfig`
- [x] `HemisphereLight` — sky_color, ground_color, intensity
- [x] `AreaLight` — width, height, color, intensity (LTC approximation)
- [x] `LightProbe` — 9-coefficient SH, `from_coefficients()`, `from_cube_faces()`, `from_equirectangular_samples()`
- [x] `ShadowConfig` — map_size, near, far, bias, pcf_radius, cascades
- [x] Tests: SH projection from raw cube samples produces non-zero coefficients

---

## v0.5.0 — Textures & Camera

**Goal:** CPU-side texture management and camera system. A developer can load textures, configure samplers, and set up perspective/orthographic/cube cameras with orbit controls.

### Crates shipped

- `scenix-texture` `v0.5.0` (new)
- `scenix-camera` `v0.5.0` (new)

### Deliverables

**`scenix-texture`**
- [x] `Texture2D` — width, height, format, data, mip_levels
- [x] `TextureCube` — 6 faces, format, mip_levels
- [x] `Texture3D` — width, height, depth, format, data
- [x] `VideoTexture` — frame-by-frame update interface
- [x] `Sampler` — mag/min/mip filter, address_u/v/w, anisotropy, compare
- [x] `TextureAtlas` — rect packing, UV lookup by name
- [x] `TextureFormat` enum — Rgba8, Rgba16Float, Depth32Float, Bc7, Astc, Etc2
- [x] `mipmap::generate(data, width, height) -> Vec<Vec<u8>>` — CPU mipmap generation
- [x] Tests: atlas packing fits expected number of rects, UV coords are valid

**`scenix-camera`**
- [x] `PerspectiveCamera` — fov_y, aspect, near, far, `projection_matrix()`, `view_matrix()`
- [x] `OrthographicCamera` — left, right, top, bottom, near, far
- [x] `CubeCamera` — 6-face render for environment maps
- [x] `Frustum` — 6 planes extracted from VP matrix, `contains_point()`, `intersects_aabb()`
- [x] `OrbitController` — target, distance, min/max polar angle, zoom, damping
- [x] `FlyController` — speed, sensitivity, WASD movement
- [x] Controllers consume `PointerState` + `KeyboardState` from `scenix-input`
- [x] Tests: frustum correctly culls points outside the view volume
- [x] Tests: orbit controller clamps polar angle to [min, max]

---

## v0.6.0 — Renderer

**Goal:** GPU rendering via wgpu. A developer can render a scene with PBR materials, shadows, and deferred+forward pipeline.

### Crates shipped

- `scenix-renderer` `v0.6.0` (new)

### Deliverables

**`scenix-renderer`**
- [x] `Renderer` — owns `wgpu::Device`, `Queue`, `Surface`, `PipelineCache`, `GpuScene`
- [x] `RendererConfig` — width, height, sample_count, vsync, hdr, present_mode, backends
- [x] `Renderer::new(window, config)` — async initialization
- [x] `Renderer::headless(config)` — offscreen renderer for tests, tools, and captures
- [x] `Renderer::render(&scene, &camera)` — full frame render
- [x] `Renderer::resize(w, h)` — surface/offscreen target reconfiguration
- [x] `GpuMaterial` trait — `bind_group_layout()`, `to_uniform_bytes()`, `create_bind_group()`
- [x] `GpuMaterial` impls for `PbrMaterial`, `UnlitMaterial`, `LambertMaterial`
- [x] `PipelineCache` — keyed by material/pass/target state, lazy compilation
- [x] `GpuScene` — renderer-owned mesh/material/texture/light registries
- [x] `FrameContext` — per-frame camera VP, resolution, and camera position state

**Render passes**
- [x] `shadow_pass.rs` — depth-only pass marker and `ShadowMapAtlas`
- [x] `geometry_pass.rs` — G-buffer pass marker and `GBuffer`
- [x] `lighting_pass.rs` — deferred lighting pass marker
- [x] `forward_pass.rs` — transparent forward pass marker
- [x] `culling.rs` — frustum culling using scene graph bounds
- [x] `sort.rs` — front-to-back opaque and back-to-front transparent sorting

**Shaders (WGSL)**
- [x] `pbr.vert.wgsl`, `pbr.frag.wgsl` — PBR vertex/fragment shader entry points
- [x] `unlit.frag.wgsl` — unlit fragment shader
- [x] `shadow_depth.vert.wgsl` — shadow pass vertex shader
- [x] `deferred_resolve.wgsl` — deferred lighting full-screen quad

- [x] Tests: pipeline cache returns same pipeline for same `PipelineKey`
- [x] Tests: headless render produces non-black framebuffer
- [x] `benches/render_bench.rs` — frame time with 1K / 10K / 100K triangles
- [x] `examples/hello_cube.rs` — headless cube render
- [x] `examples/pbr_sphere.rs` — PBR sphere with ambient and directional light setup
- [x] `examples/shadow_demo.rs` — directional light with shadow-map configuration

---

## v0.7.0 — Loaders & Post-Processing

**Goal:** Asset loading and post-processing effects. A developer can load GLTF files and apply bloom, SSAO, tone mapping, and other effects.

### Crates shipped

- `scenix-loader` `v0.7.0` (new)
- `scenix-post` `v0.7.0` (new)

### Deliverables

**`scenix-loader`**
- [x] `GltfLoader::load(path) -> Result<GltfAsset>` — meshes, materials, textures, cameras, hierarchy
- [x] `GltfLoader::load_url(url) -> Result<GltfAsset>` — async HTTP behind the `http` feature
- [x] `obj::load(path) -> Result<Vec<Geometry>>` — OBJ + MTL parsing
- [x] `stl::load(path) -> Result<Geometry>` — binary + ASCII STL
- [x] `image::load(path) -> Result<Texture2D>` — PNG, JPEG, WebP
- [x] `ktx2::load(path) -> Result<Texture2D>` — KTX2 container metadata and supported raw texture formats
- [x] `hdr::load(path) -> Result<TextureCube>` — HDR/EXR-compatible image decode to cube texture data
- [x] `AssetCache` — canonical path deduplication with `Arc<T>`, invalidation, and clear
- [x] Tests: generated glTF/GLB, OBJ/MTL, STL, image, KTX2, HDR cube, cache, and serde metadata coverage

**`scenix-post`**
- [x] `PostStack` — ordered chain of effects, builder pattern
- [x] Full-screen GPU pass stack with grow-only scratch targets and cached pipelines
- [x] Bloom — threshold, intensity, radius
- [x] SSAO — radius, intensity, bias
- [x] Tone mapping — `ToneMapper::None`, `Reinhard`, `Aces`, `Exposure`
- [x] FXAA — fast approximate anti-aliasing pass
- [x] TAA — feedback and jitter pass
- [x] SMAA — quality preset pass
- [x] Depth of field — focus distance, aperture, blur radius
- [x] Fog — screen-space fog color/density blend
- [x] Outline — luminance-edge outline
- [x] Motion blur — compact screen-space blur pass
- [x] Tests: PostStack ordering, removal, clear behavior, config clamps, serde, and GPU-gated smoke path
- [x] `examples/post_processing.rs` — stack: SSAO + Bloom + ToneMap + FXAA + TAA
- [x] `examples/gltf_scene.rs` — generate, load, register, and render a tiny glTF scene

---

## v0.8.0 — Raycasting & Helpers

**Goal:** BVH-accelerated raycasting and debug visualization. A developer can pick objects with mouse and visualize scene structure.

### Crates shipped

- `scenix-raycaster` `v0.8.0` (new)
- `scenix-helpers` `v0.8.0` (new)

### Deliverables

**`scenix-raycaster`**
- [x] `Raycaster` — `cast_ray(scene, ray) -> Option<Intersection>`
- [x] `Raycaster::cast_ray_all(scene, ray) -> Vec<Intersection>` — all hits, sorted by distance
- [x] `Raycaster::from_camera_ndc(camera, ndc_x, ndc_y) -> Ray3`
- [x] `Intersection` — `node_id`, `distance`, `point`, `normal`, `uv`
- [x] `Bvh` — SAH-based build from scene AABB list
- [x] `Bvh::traverse(ray) -> Vec<NodeId>` — candidate list
- [x] Tests: ray-AABB, ray-triangle, ray-sphere intersection correctness
- [x] Tests: BVH produces same results as brute-force (correctness proof)
- [x] `benches/bvh_bench.rs` — BVH build + 1K ray queries

**`scenix-helpers`**
- [x] `LineGeometry` — validated line-list storage for helper output
- [x] `GridHelper` — `to_geometry()` → line-list grid plane
- [x] `AxesHelper` — `to_geometry()` → RGB XYZ axis lines
- [x] `BoundingBoxHelper` — wireframe AABB
- [x] `ArrowHelper` — directional arrow with configurable head
- [x] `SpotLightHelper`, `PointLightHelper`, `DirectionalLightHelper`
- [x] `CameraHelper` — frustum wireframe visualization
- [x] `SkeletonHelper` — bone visualization
- [x] `examples/raycasting.rs` — mouse picking with BVH
- [x] `examples/helpers_demo.rs` — all helpers in one scene

---

## v0.9.0 — Integration

**Goal:** Connect scenix to animato and the browser. A developer can animate scene properties with springs/tweens and run scenix in a web page.

### Crates shipped

- `scenix-animato` `v0.9.0` (new)
- `scenix-wasm` `v0.9.0` (new)

### Deliverables

**`scenix-animato`**
- [x] `AnimVec3`, `AnimQuat`, `AnimColor` wrappers for Animato interpolation, with quaternion slerp for rotations
- [x] `ScalarTrack`, `Vec3Track`, `QuatTrack`, `ColorTrack`, `BoolTrack` backed by Animato 1.4.0 tweens and springs where applicable
- [x] `NodeAnimator` — binds tracks to `NodeId` transform and visibility fields
- [x] `NodeAnimationTarget` enum — `Translation`, `Rotation`, `Scale`, `Visibility`
- [x] `CameraAnimator` — animates fov, position, target, up vector, and orthographic bounds through `CameraStoreMut`
- [x] `MaterialAnimator` — animates PBR albedo, opacity, emissive, roughness, and metallic fields
- [x] `SkeletonPose`, `BoneAnimation`, `SkinnedMeshAnimator` — drives explicit bone transform arrays
- [x] `ScenixAnimationDriver` — ticks all bound animators per frame with pause/resume, add/remove/clear, completion pruning, and deterministic order
- [x] Tests: node transform/visibility animation, camera stores, PBR material fields, skeleton poses, driver behavior, serde round trips

**`scenix-wasm`**
- [x] `WebRenderer` — wraps `Renderer`, `SceneGraph`, `PerspectiveCamera`, `PointerState`, and `KeyboardState` for `<canvas>` + `requestAnimationFrame`
- [x] `WebRenderer::new(canvas) -> Result<WebRenderer, JsValue>` — async init
- [x] `WebRenderer::tick(timestamp_ms)` — called from rAF
- [x] `WebRenderer::resize(w, h)`
- [x] `on_pointer_move/down/up`, `on_wheel`, `on_key_down/up` — DOM input forwarding
- [x] `key_code_from_dom`, `pointer_button_from_dom`, `canvas_size`, `clamp_canvas_size`, and panic hook helpers
- [x] `examples/wasm_viewer/` — generated-scene browser viewer
- [x] `examples/animato_integration.rs` — spring camera target + tween node/material animation
- [x] Tests/checks: DOM mapping unit tests, zero-size resize clamping, wasm target compile, wasm viewer compile

---

## v1.0.0 — Stable

**Goal:** API freeze. Every public item is documented, every example compiles, every feature has tests, CI is fully green on stable + beta + nightly.

### Deliverables

**API Stability**
- [x] Review public facade and subsystem APIs for the stable v1 contract
- [x] Keep optional heavy systems behind explicit feature flags
- [x] Prefer additive v1 changes and document deprecation policy

**Documentation**
- [x] `docs/getting-started.md`
- [x] `docs/concepts.md`
- [x] `docs/materials-guide.md`
- [x] `docs/platform-guide.md`
- [x] `docs/benchmarks.md`
- [x] `docs/release-v1.1.0.md`
- [x] README, architecture notes, changelog, and release automation updated

**Testing**
- [x] Stable/beta/nightly test workflow
- [x] WASM compile checks for `scenix-wasm` and the standalone viewer example
- [x] Headless GPU test workflow with lavapipe
- [x] Coverage workflow step using `cargo-llvm-cov`
- [x] Renderer material registration and facade v1 integration coverage

**CI**
- [x] `stable`, `beta`, `nightly` tests
- [x] WASM target checks
- [x] no-default checks for CPU/no_std crates
- [x] Clippy `--all-features -- -D warnings`
- [x] `cargo fmt --check`
- [x] Bench compile gate

**Release**
- [x] `CHANGELOG.md` includes the stable release
- [x] GitHub Release uses `docs/release-v1.1.0.md`
- [x] GitHub Pages website and WASM demo workflow added

---

## Current Focus

The project is now in post-1.1 planning and maintenance. New work should preserve the stable modular API, keep heavy dependencies optional, and add deprecations before removing public APIs.

Scenix is not only a website or WASM demo library. Future work must treat **desktop, mobile, and web** as first-class runtime targets:

- Desktop: Linux, Windows, and macOS through `wgpu` surfaces.
- Mobile: Android and iOS through `wgpu`, native lifecycle handling, touch/gamepad input, and mobile texture formats.
- Web: WASM with WebGPU first, WebGL2 fallback, and clean unavailable-backend handling.

## Future Milestones (`v1.x`)

The version numbers below are planning buckets, not release promises. Each milestone should ship only when tests, docs, examples, and benches are ready.

## v1.2.0 — Renderer And Material Parity

**Goal:** Move from stable preview rendering to production-quality material and lighting behavior.

### Crates shipped / updated

- `scenix-renderer` — primary work: real GPU material paths, texture binding, lights, shadows, IBL, render targets, diagnostics, resource lifecycle, and scene-to-renderer sync.
- `scenix-material` — material parameters, texture slots, physical material extensions, alpha behavior, pipeline keys, and future node-material integration points.
- `scenix-texture` — sampler metadata, mipmaps, compressed texture formats, video texture update contract, and texture validation needed by GPU upload.
- `scenix-light` — light/shadow data needed by renderer upload, cascades, probes, and area-light renderer-facing metadata.
- `scenix-post` — post target reuse, depth/normal/motion input contracts, and backend fallback hooks where renderer changes affect post-processing.
- `scenix-wasm` — WebGPU/WebGL feature parity notes, fallback behavior, and browser renderer smoke paths.
- `scenix` — facade re-exports and feature flags for any new public renderer/material/texture APIs.
- No new crate is planned for this release.

- [ ] Real GPU texture allocation and binding for `Texture2D`, `TextureCube`, `Texture3D`, video textures, mip levels, and samplers.
- [ ] PBR shader path with albedo, metallic-roughness, normal, occlusion, emissive, vertex colors, alpha mask, alpha blend, and double-sided rendering.
- [ ] Physical shader path with clearcoat, sheen, transmission, thickness, IOR, iridescence, attenuation, and environment response.
- [ ] Real lighting integration for ambient, hemisphere, directional, point, spot, area, and light probes.
- [ ] Shadow map rendering for directional, point, and spot lights, including atlas allocation, PCF, bias controls, and cascade support.
- [ ] Environment lighting with cube/equirectangular maps, irradiance, prefiltered radiance, BRDF lookup, and PMREM-style filtering.
- [ ] Render targets for 2D, cube, 3D, array, depth, multisampled, HDR, offscreen capture, and readback.
- [ ] Color management and tone mapping policy shared across desktop, mobile, WebGPU, and WebGL fallback.
- [ ] Renderer stats for draw calls, triangles, material count, texture memory, visible/culled objects, GPU timings where supported, and pipeline cache activity.
- [ ] Resource lifecycle APIs for register, update, unregister, clear, dispose, and reload workflows.
- [ ] Scene-to-renderer sync helpers that report created, updated, reused, skipped, and removed GPU resources.
- [ ] Capability matrix for renderer features across desktop, mobile, WebGPU, and WebGL fallback.

## v1.3.0 — Asset Pipeline

**Goal:** Make Scenix useful for real production assets, not only generated scenes.

### Crates shipped / updated

- `scenix-loader` — primary work: glTF extensions, skins, morphs, animation import, extra loaders, exporters, asset cache, async loading, hot reload, and asset metadata.
- `scenix-mesh` — geometry attributes required by imported assets, morph target import, skinning-related vertex attributes, and mesh compression/decompression integration points.
- `scenix-material` — imported material extension mapping for clearcoat, transmission, volume, sheen, specular, IOR, iridescence, emissive strength, texture transforms, and variants.
- `scenix-texture` — KTX2/BasisU metadata, compressed texture validation, texture transform metadata, and image/HDR/EXR integration points.
- `scenix-scene` — imported hierarchy, node metadata, variant metadata, and loaded scene organization.
- `scenix-camera` — imported perspective/orthographic camera conversion and metadata.
- `scenix-light` — imported punctual lights and future IES/light metadata.
- `scenix-animato` — imported animation data handoff into the animation runtime planned for `v1.4.0`.
- `scenix-renderer` — optional asset-to-GPU convenience helpers for registered loaded assets.
- `scenix` — facade re-exports and feature flags for new loader/exporter APIs.
- No new crate is planned for this release.

- [ ] glTF skins, skeletons, morph targets, animation clips, and node/light/camera extension support.
- [ ] glTF material extensions: clearcoat, transmission, volume, sheen, specular, IOR, iridescence, emissive strength, texture transform, variants, KTX2/BasisU, meshopt, and Draco.
- [ ] Additional loaders: FBX, Collada, PLY, SVG, USD/USDZ, 3MF, VOX, VTK, Rhino 3DM, LDraw, TTF/font, IES, DDS, TGA, TIFF, EXR, UltraHDR, and LUT formats.
- [ ] Exporters: glTF/GLB, OBJ, STL, PLY, USDZ, KTX2, EXR/HDR, and scene JSON.
- [ ] Asset manager with async loading, progress callbacks, cancellation, dependency graphs, memory budgets, cache invalidation, and desktop hot reload.
- [ ] Asset examples covering local files, URLs, embedded bytes, browser assets, desktop assets, and mobile packaged assets.
- [ ] Asset-to-GPU convenience helpers for loaded glTF/OBJ/STL/image assets while preserving manual renderer registration.

## v1.4.0 — Animation Runtime

**Goal:** Keep Animato as the value engine while adding a scene/asset animation layer comparable to Three.js clips and mixers.

### Crates shipped / updated

- `scenix-animato` — primary work: `AnimationClip`, `AnimationAction`, `AnimationMixer`, property binding, blending, events, loop modes, and deterministic sampling.
- `scenix-loader` — loaded animation clips, skeletons, morph animation channels, and imported clip metadata from glTF/FBX-style assets.
- `scenix-mesh` — morph weights, skinning data model, vertex attributes, and CPU/GPU skinning data handoff.
- `scenix-scene` — animation property paths for nodes, visibility, transforms, and scene hierarchy targets.
- `scenix-material` — animation targets for material fields and texture-driven material variants.
- `scenix-camera` — animation targets for camera position, target, projection, and orthographic bounds.
- `scenix-light` — animation targets for light intensity, color, range, angle, and shadow-related fields.
- `scenix-renderer` — GPU skinning, morph upload, animation-driven resource updates, and renderer sync hooks.
- `scenix-helpers` — skeleton, path, and animation debugging helpers.
- `scenix` — facade re-exports and feature flags for new animation APIs.
- No new crate is planned for this release.

- [ ] `AnimationClip`, `AnimationAction`, and `AnimationMixer` equivalents for imported clips.
- [ ] Property binding for node transforms, visibility, material fields, cameras, lights, morph weights, and skeleton bones.
- [ ] Playback controls: loop modes, pause/resume, time scale, markers, events, crossfade, additive blending, and deterministic sampling.
- [ ] Skeletal animation data model, GPU skinning path, CPU fallback tests, skeleton helpers, and pose debugging.
- [ ] Retargeting helpers and optional IK helpers.
- [ ] Animation path helper and docs for imported animation workflows.

## v1.5.0 — Controls, Interaction, And Editor Primitives

**Goal:** Support product viewers, editors, games, CAD-like tools, and data visualization without every app rebuilding interaction basics.

### Crates shipped / updated

- `scenix-camera` — primary camera-control work: arcball, trackball, map, first-person, pointer-lock, and improved orbit/fly behavior.
- `scenix-input` — touch, gesture, gamepad, pointer lock, high-DPI normalization, and cross-platform input mapping.
- `scenix-raycaster` — selection box/frustum picking, drag-plane support, hover/active/selected workflows, layer masks, and picking helpers.
- `scenix-helpers` — transform gizmo geometry, selection helpers, bounds helpers, camera/light/skeleton editor helpers, and snapping/grid visuals.
- `scenix-scene` — selection metadata, layer policies, editor-facing node metadata, and scene inspector support.
- `scenix-renderer` — object ID/depth/normal buffers or readback hooks required by editor picking and viewport overlays.
- `scenix-wasm` — browser input forwarding for pointer lock, touch gestures, drag controls, and WebView/browser viewer behavior.
- `scenix` — facade re-exports and feature flags for new controls/input/helper APIs.
- No new crate is planned for this release; `scenix-editor` should wait until these primitives are ready.

- [ ] Arcball, Trackball, Map, FirstPerson, PointerLock, Drag, and Transform controls.
- [ ] Translation, rotation, scale, bounds, camera, light, and skeleton gizmos.
- [ ] Selection box/frustum picking, hover/active/selected state model, drag planes, snapping, grid constraints, and layer masks.
- [ ] Inspector data model for scene graph, cameras, lights, materials, textures, animations, renderer stats, and GPU resources.
- [ ] Web overlay support and native overlay integration for egui or Iced.
- [ ] Mobile touch gesture mapping for orbit, pan, pinch zoom, drag, and transform operations.

## v1.6.0 — Shader Nodes And Node Materials

**Goal:** Add a typed shader graph layer above raw `ShaderMaterial` without weakening the low-level WGSL escape hatch.

### Crates shipped / updated

- `scenix-nodes` — new optional crate for shader nodes, material graphs, serialized node graphs, WGSL generation, and WebGL-compatible subset validation.
- `scenix-material` — node-material integration points, material graph references, pipeline-key integration, and built-in node material descriptors.
- `scenix-renderer` — node shader compilation, shader cache integration, bind group layout generation, diagnostics, and fallback behavior.
- `scenix-post` — post-processing node graph integration where practical.
- `scenix-wasm` — WebGPU/WebGL compatibility checks for generated shaders.
- `scenix` — optional `nodes` facade feature and re-exports.

- [ ] `scenix-nodes` crate scaffold with docs, tests, and facade feature.
- [ ] Typed nodes for constants, uniforms, attributes, varyings, textures, math, color space, lighting, fog, tone mapping, and post effects.
- [ ] WGSL backend for renderer integration.
- [ ] WebGL-compatible subset validator for browser fallback.
- [ ] First node material rendered through `scenix-renderer`.
- [ ] Serialization format for editor-generated material graphs.

## v1.7.0 — Particles

**Goal:** Add reusable particle systems for effects, visualizations, sprites, and lightweight simulations.

### Crates shipped / updated

- `scenix-particles` — new optional crate for emitters, particle data, CPU simulation, GPU simulation where supported, modules, and examples.
- `scenix-scene` — particle scene attachments or node metadata.
- `scenix-mesh` — billboard/sprite/point geometry helpers and particle buffer layouts.
- `scenix-material` — particle, sprite, points, soft-particle, and flipbook material support.
- `scenix-texture` — atlas/flipbook texture support and particle texture metadata.
- `scenix-renderer` — particle draw path, batching, instancing, GPU buffers, optional compute path, and fallback path.
- `scenix-wasm` — WebGPU/WebGL particle capability notes.
- `scenix` — optional `particles` facade feature and re-exports.

- [ ] CPU particle emitter with spawn, lifetime, velocity, acceleration, color, size, rotation, and curve modules.
- [ ] Sprite/point particle rendering example.
- [ ] Particle texture atlas and flipbook animation support.
- [ ] Batched particle upload path.
- [ ] Optional GPU compute simulation where supported.
- [ ] WebGL fallback strategy for non-compute particle scenes.

## v1.8.0 — Terrain, Sky, And Water

**Goal:** Add reusable environment systems for real scenes, games, product viewers, and simulations.

### Crates shipped / updated

- `scenix-terrain` — new optional crate for heightmaps, chunked LOD, splat maps, terrain collision data, streaming, and terrain examples.
- `scenix-sky` — new optional crate for procedural sky, atmosphere, sun/sky lighting helpers, environment map generation, and grounded skybox support.
- `scenix-water` — new optional crate for water surfaces, waves, foam, reflection, refraction, fresnel behavior, and underwater helpers.
- `scenix-renderer` — terrain LOD draw path, sky/background path, water reflection/refraction targets, environment capture hooks, and fallback behavior.
- `scenix-material` — terrain, sky, atmosphere, and water material descriptors.
- `scenix-texture` — heightmap, normal map, splat map, and environment texture handling.
- `scenix-light` — sun/sky/environment light integration.
- `scenix-scene` — environment object attachments and metadata.
- `scenix` — optional `terrain`, `sky`, and `water` facade features and re-exports.

- [ ] Heightmap terrain with chunked LOD example.
- [ ] Terrain splat-map material path.
- [ ] Procedural sky background and sun/sky lighting helper.
- [ ] Water plane with reflection/refraction render targets.
- [ ] Mobile/web capability notes for environment features.
- [ ] Shared environment demo scene.

## v1.9.0 — XR, Audio, And Physics Bridges

**Goal:** Add optional runtime bridges for immersive applications, spatial sound, and simulation while keeping the core renderer independent.

### Crates shipped / updated

- `scenix-xr` — new optional crate for WebXR/OpenXR sessions, controllers, hands, hit tests, anchors, planes, depth sensing, estimated lighting, and mobile XR lifecycle.
- `scenix-audio` — new optional crate for audio listeners, positional audio sources, streaming audio, analyser data, and scene-node attachment.
- `scenix-physics` — new optional crate for Rapier/Jolt bridge, rigid bodies, colliders, character controller helpers, debug visualization, and scene transform sync.
- `scenix-scene` — node attachments and sync metadata for XR, audio, and physics.
- `scenix-input` — XR controller, gamepad, touch, and device input mapping.
- `scenix-camera` — XR camera rigs, stereo camera helpers, and listener/camera sync.
- `scenix-helpers` — physics collider helpers, XR/controller helpers, and debug visualizations.
- `scenix-renderer` — stereo/XR render target hooks and debug drawing support where needed.
- `scenix-wasm` — WebXR browser integration.
- `scenix` — optional `xr`, `audio`, and `physics` facade features and re-exports.

- [ ] One WebXR or OpenXR viewer example.
- [ ] XR controller input and pose mapping.
- [ ] Audio listener plus positional source attached to scene nodes.
- [ ] Physics rigid body/collider bridge with transform synchronization.
- [ ] Character-controller or simple collision example.
- [ ] Debug helpers for controllers, colliders, and physics state.

## v1.10.0 — Editor And UI Tooling

**Goal:** Build editor-facing tools only after renderer, asset, animation, controls, and resource lifecycle APIs are strong enough.

### Crates shipped / updated

- `scenix-editor` — new optional crate for visual editor shell, asset browser, scene inspector, gizmos, material editor, animation timeline, import/export workflow, and project metadata.
- `scenix-ui` or `scenix-egui` — new optional crate for cross-platform debug UI overlays, renderer stats, scene inspector panels, and tool widgets.
- `scenix-scene` — editor metadata, inspector support, selection state, and serialization hooks.
- `scenix-renderer` — viewport overlays, object ID/depth/normal buffers, renderer stats, and resource inspector hooks.
- `scenix-material` — material inspector and node material editor support.
- `scenix-loader` — asset browser, import/export workflow, reload, and asset dependency graph integration.
- `scenix-animato` — animation timeline, clips, action controls, and preview playback.
- `scenix-helpers` — transform gizmos, selection helpers, debug overlays, and editor visuals.
- `scenix-wasm` — browser editor support where practical.
- `scenix` — optional `editor` and `ui` facade features and re-exports.

- [ ] Minimal scene inspector/editor shell.
- [ ] Renderer stats overlay.
- [ ] Scene graph panel with selection and transform editing.
- [ ] Material inspector with texture slots and physical material fields.
- [ ] Asset browser using loader/exporter APIs.
- [ ] Animation timeline preview for imported clips.
- [ ] Save/load project or scene metadata format.

## v1.x+ — Advanced Rendering And Geometry Extras

**Goal:** Track advanced work that is not assigned to a specific future version yet.

| Area | Future Work |
|------|-------------|
| Post-processing | SSR, SSGI, GTAO, SAO, LUT, film, vignette, chromatic aberration, glitch, halftone, pixelation, afterimage, transition, mask, denoise, god rays, lens flare |
| Geometry | Polyhedron, tetrahedron, octahedron, dodecahedron, rounded box, text, decal, convex, parametric, NURBS, edges, wireframe geometry |
| Modifiers | Simplify, tessellate, edge split, curve flow, mesh surface sampler, convex hull, OBB, octree |
| Scene objects | Reflector, refractor, lens flare, marching cubes, shadow catcher, impostors, decals, volume slices |
| GPU-driven rendering | Indirect draws, GPU culling, clustered/forward+ lighting, large-scene renderer paths |
| Realtime GI | SSGI first, future probe/grid/path options when practical |

### Optional Future Crates

| Crate | Notes |
|------|-------|
| `scenix-nodes` | Shader graph, typed shader nodes, node materials, serialized material graphs, WGSL backend, and WebGL-compatible subset |
| `scenix-particles` | CPU particles, GPU particles where supported, sprite batching, emitter modules, and particle examples |
| `scenix-terrain` | Heightmap terrain, chunked LOD, splat maps, terrain collision data, and streaming |
| `scenix-sky` | Procedural sky, atmosphere scattering, sun/sky lighting helpers, and grounded skybox support |
| `scenix-water` | Water surfaces, reflection/refraction helpers, foam, waves, and underwater/fresnel material support |
| `scenix-xr` | WebXR and OpenXR support for VR/AR, controllers, hand tracking, hit tests, anchors, planes, estimated lighting, and mobile XR lifecycle |
| `scenix-audio` | Audio listener, spatial audio source, streaming audio, analyser data, and scene-node attachment |
| `scenix-physics` | Rapier/Jolt bridge, rigid bodies, colliders, character controller helpers, debug visualization, and scene transform sync |
| `scenix-editor` | Visual scene editor, asset browser, scene inspector, gizmos, material editor, animation timeline, import/export workflow |
| `scenix-ui` or `scenix-egui` | Cross-platform debug UI overlays for desktop, mobile, and web |

### Crate Work Map

**Goal:** Make it clear where future work belongs before opening issues or creating new crates.

Existing crates to extend first:

| Work Area | Primary Crate(s) | Work To Do |
|-----------|------------------|------------|
| Production renderer | `scenix-renderer` | PBR/physical GPU shaders, real lights, shadows, IBL, render targets, renderer stats, GPU resource lifecycle |
| Material model | `scenix-material` | Material parameters, pipeline keys, texture slots, physical extensions, future node-material integration points |
| Texture system | `scenix-texture`, `scenix-renderer` | Compressed formats, mipmaps, sampler metadata, video texture updates, GPU upload/binding, texture memory accounting |
| Light and shadows | `scenix-light`, `scenix-renderer` | Shadow config, cascades, probes, light upload, shadow atlas integration, area-light renderer behavior |
| Asset import/export | `scenix-loader` | glTF extensions, animation import, skins, morphs, extra loaders, exporters, asset cache, async loading, hot reload |
| Post-processing | `scenix-post`, `scenix-renderer` | Extra effects, depth/normal/motion buffers, post graph, target reuse, backend feature fallback |
| Imported animation | `scenix-animato`, `scenix-loader`, `scenix-mesh` | Animation clips, mixer/action layer, property binding, skeleton playback, morph playback, blending |
| Controls and picking | `scenix-camera`, `scenix-input`, `scenix-raycaster`, `scenix-helpers` | Arcball/trackball/map/first-person/pointer-lock controls, drag helpers, selection box, gizmo geometry |
| Scene graph data | `scenix-scene`, `scenix-core` | Resource/version tracking, layer policies, scene-to-renderer sync metadata, stronger IDs and error handling |
| Browser runtime | `scenix-wasm`, `scenix-renderer` | WebGPU/WebGL parity notes, fallback behavior, browser smoke scenes, WebView support |
| Desktop/mobile runtime | `scenix-renderer`, `scenix-input`, examples | winit examples, Android/iOS examples, surface loss/recreate, high-DPI, touch, gesture, gamepad |
| Visual validation | `tests/`, `examples/`, `benches/` | Golden images, renderer smoke tests, compatibility scenes, benchmark gates |

Future crates to create only when implementation starts:

| Future Crate | Create When | First Useful Deliverable |
|--------------|-------------|--------------------------|
| `scenix-nodes` | Shader graph work needs more than `ShaderMaterial` | One node material rendered through `scenix-renderer` with docs and tests |
| `scenix-particles` | Sprite/billboard examples need reusable emitters | CPU particle emitter with one renderer example |
| `scenix-terrain` | Heightmap/chunked LOD work begins | Heightmap terrain mesh with LOD example |
| `scenix-sky` | Environment/sky features grow beyond materials | Procedural sky driving scene lighting or background |
| `scenix-water` | Water needs reflection/refraction render targets | Water plane example with reflection/refraction path |
| `scenix-xr` | WebXR/OpenXR runtime work begins | One WebXR or OpenXR viewer example with controller input |
| `scenix-audio` | Spatial audio integration begins | Listener plus positional source attached to scene nodes |
| `scenix-physics` | Physics sync design is ready | Rigid body/collider bridge with debug helper example |
| `scenix-editor` | Inspector/gizmo/resource APIs are ready enough | Minimal scene inspector/editor shell using existing crates |
| `scenix-ui` or `scenix-egui` | Debug UI needs reusable overlays | Renderer stats overlay and scene inspector panel |

Do not add a future crate to the workspace manifest until it has a crate directory, feature flag plan, docs, tests, and at least one example.

### Cross-Cutting Resource Management

**Goal:** Make Scenix usable in long-running apps, editors, mobile apps, and asset-heavy scenes without leaking ownership or hiding GPU cost.

- [ ] Stable resource lifecycle policy for CPU data, GPU data, asset cache entries, render targets, shadow maps, post targets, and browser fallback resources.
- [ ] Dirty/version tracking for geometry, textures, materials, lights, transforms, skeletons, morph weights, and animation-driven data.
- [ ] GPU memory accounting for vertex buffers, index buffers, textures, uniform buffers, render targets, shadow atlases, and post scratch targets.
- [ ] Resource budget controls for geometry memory, texture memory, post-processing targets, shadow maps, and asset cache size.
- [ ] Explicit cleanup APIs for hot reload, editor delete operations, scene unload, device loss, and mobile suspend/resume.
- [ ] Clear error categories for unsupported formats, unsupported GPU features, stale handles, invalid IDs, budget exceeded, upload failure, and device/surface loss.

### Cross-Cutting Validation And Visual Tests

**Goal:** Prove renderer and asset behavior with repeatable tests, not only examples.

- [ ] Golden image tests for reference scenes, material balls, shadows, post-processing, glTF sample models, and selected animation frames.
- [ ] Pixel-diff and perceptual-diff tolerances documented per backend.
- [ ] Headless renderer smoke tests for textures, shadows, environment maps, transparent sorting, post effects, and readback.
- [ ] Browser WebGPU and WebGL smoke scenes with documented fallback behavior.
- [ ] Desktop compatibility table for Linux/Vulkan, Windows/DX12 or Vulkan, and macOS/Metal.
- [ ] Mobile compatibility table for Android/Vulkan and iOS/Metal, including lifecycle, touch, DPI, and compressed textures.
- [ ] Feature support labels: `Full`, `Partial`, `Fallback`, or `Unsupported` for desktop, mobile, WebGPU, and WebGL.
- [ ] Conformance scenes for PBR, physical material extensions, glTF extensions, skinning, morph targets, animation clips, post effects, and picking.
- [ ] Benchmarks for large scenes, texture upload, asset loading, BVH builds, animation sampling, and frame render time.
- [ ] CI gates for CPU/no_std, all-features, wasm compile, renderer smoke, docs, examples, and benchmark compile checks.

### Cross-Platform Example Matrix

**Goal:** Make every supported runtime visible to users through working examples.

| Target | Required Examples |
|--------|-------------------|
| Desktop | winit surface app, egui overlay app, Tauri/WebView app, headless/offscreen capture |
| Mobile | Android Vulkan/wgpu app, iOS Metal/wgpu app, touch orbit controls, packaged asset loading, suspend/resume |
| Web | WebGPU viewer, WebGL fallback viewer, asset loading from URL, graceful unsupported-backend UI |
| Shared | Same scene rendered on desktop, mobile, and web with documented feature differences |

## Future Work Rules

- Every feature must state desktop, mobile, and web support level.
- Heavy systems stay opt-in through focused crates or facade features.
- CPU authoring crates must remain renderer-agnostic.
- Browser-only APIs must not leak into desktop/mobile core APIs.
- Mobile-only constraints such as lifecycle, touch input, DPI, surface recreation, and compressed textures must be handled deliberately.
- Native desktop, Android, iOS, and web examples should be added as features mature instead of being tied to a single release bucket.
- `scenix-input` should grow touch, gesture, gamepad, pointer lock, and high-DPI normalization as cross-platform requirements.
- Texture capability detection for BC, ASTC, ETC2, KTX2/BasisU, and fallback transcodes should be handled by the relevant renderer/asset milestones.
- WebGPU/WebGL parity notes and graceful feature fallback behavior should be documented whenever a renderer feature is added.
- New renderer features need focused examples, tests, and at least one benchmark or smoke test.

---

## Contributing to scenix

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for how to set up the workspace, run tests, and submit pull requests.

The best way to contribute now is to propose a focused post-1.1 planning issue or PR that preserves the stable API contract.

---

*Roadmap version: 1.1.0 + future parity plan — last updated June 3, 2026*
*Next milestone: v1.2.0 renderer and material parity planning*
*Project: Aarambh Dev Hub — github.com/AarambhDevHub/scenix*
*Companion library: animato — github.com/AarambhDevHub/animato*
