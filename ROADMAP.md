# scenix — Project Roadmap

> *Italian: scenix — scene, the stage on which everything appears.*
> A professional-grade, renderer-agnostic 3D scene library for Rust.

This roadmap tracks every planned release from `v0.1.0` through `v1.0.0`.
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
| `v0.3.0` | Geometry | Mesh, all primitives, morph targets, instanced/batched mesh | 📋 |
| `v0.4.0` | Materials & Lights | Material trait, PBR, physical, toon, all light types | 📋 |
| `v0.5.0` | Textures & Camera | Texture loading, sampler, atlas, camera types, controllers | 📋 |
| `v0.6.0` | Renderer | wgpu pipeline, deferred+forward rendering, shadow maps | 📋 |
| `v0.7.0` | Loaders & Post | GLTF/OBJ/STL loaders, post-processing stack | 📋 |
| `v0.8.0` | Raycasting & Helpers | BVH raycaster, debug helpers, input abstraction | 📋 |
| `v0.9.0` | Integration | animato bridge, WASM browser support, framework compat | 📋 |
| `v1.0.0` | Stable | API freeze, full docs, examples, all CI green | 📋 |

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

### Deliverables

**`scenix-mesh`**
- [ ] `Geometry` struct — positions, normals, uvs, uvs2, colors, indices, tangents
- [ ] `geometry.compute_normals()` — face-weighted vertex normals
- [ ] `geometry.compute_tangents()` — MikkTSpace tangent generation
- [ ] `geometry.aabb()`, `geometry.bounding_sphere()`
- [ ] `geometry.merge(other)` — combine geometries
- [ ] `Mesh` struct — `Geometry` + `MaterialId`
- [ ] `BufferLayout`, `VertexAttribute`, `IndexFormat`
- [ ] `MorphTarget` — `name`, `positions_delta`, `normals_delta`, `weight`
- [ ] `InstancedMesh` — `mesh_id`, `material_id`, `transforms: Vec<Mat4>`, `set_transform_at()`
- [ ] `BatchedMesh` — multiple geometries in single draw call

**Primitives (all return `Geometry`)**
- [ ] `box_geometry(w, h, d, w_seg, h_seg, d_seg)`
- [ ] `sphere_geometry(radius, width_seg, height_seg)`
- [ ] `plane_geometry(w, h, w_seg, h_seg)`
- [ ] `cylinder_geometry(top_r, bottom_r, height, radial_seg, height_seg, open_ended)`
- [ ] `cone_geometry(radius, height, radial_seg, height_seg)`
- [ ] `capsule_geometry(radius, height, cap_seg, radial_seg)`
- [ ] `torus_geometry(radius, tube, radial_seg, tubular_seg)`
- [ ] `torus_knot_geometry(radius, tube, tubular_seg, radial_seg, p, q)`
- [ ] `icosphere_geometry(radius, subdivisions)`
- [ ] `circle_geometry(radius, segments, theta_start, theta_length)`
- [ ] `ring_geometry(inner_r, outer_r, theta_seg, phi_seg)`
- [ ] `lathe_geometry(points, segments, phi_start, phi_length)`
- [ ] `extrude_geometry(shape, depth, bevel_thickness, bevel_size, bevel_segments)`
- [ ] `tube_geometry(path, tubular_seg, radius, radial_seg, closed)`
- [ ] `shape_geometry(shape)` — 2D shape triangulation

- [ ] Tests: every primitive produces valid normals (dot(n, face_normal) > 0)
- [ ] Tests: every primitive has correct vertex count for given segment params
- [ ] Tests: UV coordinates in [0, 1] range
- [ ] `benches/mesh_gen_bench.rs` — primitive generation throughput

---

## v0.4.0 — Materials & Lights

**Goal:** The material system and all light types. A developer can create PBR, toon, and custom shader materials, and illuminate scenes with all standard light types.

### Crates shipped

- `scenix-material` `v0.4.0` (new)
- `scenix-light` `v0.4.0` (new)

### Deliverables

**`scenix-material`**
- [ ] `Material` trait — `pipeline_key()`, `is_transparent()`, `double_sided()`, `alpha_cutoff()`
- [ ] `PipelineKey` struct — determines which shader pipeline to use
- [ ] `AlphaMode` enum — `Opaque`, `Mask(f32)`, `Blend`
- [ ] `PbrMaterial` — albedo, metallic, roughness, normal/ao/emissive textures, alpha mode
- [ ] `PhysicalMaterial` — clearcoat, sheen, transmission, thickness, IOR, iridescence
- [ ] `UnlitMaterial` — color + optional texture, no lighting
- [ ] `LambertMaterial` — diffuse-only, faster than PBR
- [ ] `ToonMaterial` — gradient map, discrete steps, outline
- [ ] `NormalMaterial` — debug normals → RGB
- [ ] `WireframeMaterial` — wireframe overlay
- [ ] `DepthMaterial` — shadow pass depth output
- [ ] `LineMaterial` — width, dash pattern, color
- [ ] `PointsMaterial` — point size, attenuation
- [ ] `ShaderMaterial` — custom WGSL vertex/fragment, raw uniforms
- [ ] Tests: `PipelineKey` uniqueness for different material configs

**`scenix-light`**
- [ ] `AmbientLight` — color, intensity
- [ ] `DirectionalLight` — direction, color, intensity, optional `ShadowConfig`
- [ ] `PointLight` — color, intensity, range, decay, optional `ShadowConfig`
- [ ] `SpotLight` — color, intensity, range, angle, penumbra, optional `ShadowConfig`
- [ ] `HemisphereLight` — sky_color, ground_color, intensity
- [ ] `AreaLight` — width, height, color, intensity (LTC approximation)
- [ ] `LightProbe` — 9-coefficient SH, `from_cube_texture()`, `from_equirectangular()`
- [ ] `ShadowConfig` — map_size, near, far, bias, pcf_radius, cascades
- [ ] Tests: SH projection from cubemap produces non-zero coefficients

---

## v0.5.0 — Textures & Camera

**Goal:** CPU-side texture management and camera system. A developer can load textures, configure samplers, and set up perspective/orthographic/cube cameras with orbit controls.

### Crates shipped

- `scenix-texture` `v0.5.0` (new)
- `scenix-camera` `v0.5.0` (new)

### Deliverables

**`scenix-texture`**
- [ ] `Texture2D` — width, height, format, data, mip_levels
- [ ] `TextureCube` — 6 faces, format, mip_levels
- [ ] `Texture3D` — width, height, depth, format, data
- [ ] `VideoTexture` — frame-by-frame update interface
- [ ] `Sampler` — mag/min/mip filter, address_u/v/w, anisotropy, compare
- [ ] `TextureAtlas` — rect packing, UV lookup by name
- [ ] `TextureFormat` enum — Rgba8, Rgba16Float, Depth32Float, Bc7, Astc, Etc2
- [ ] `mipmap::generate(data, width, height) -> Vec<Vec<u8>>` — CPU mipmap generation
- [ ] Tests: atlas packing fits expected number of rects, UV coords are valid

**`scenix-camera`**
- [ ] `PerspectiveCamera` — fov_y, aspect, near, far, `projection_matrix()`, `view_matrix()`
- [ ] `OrthographicCamera` — left, right, top, bottom, near, far
- [ ] `CubeCamera` — 6-face render for environment maps
- [ ] `Frustum` — 6 planes extracted from VP matrix, `contains_point()`, `intersects_aabb()`
- [ ] `OrbitController` — target, distance, min/max polar angle, zoom, damping
- [ ] `FlyController` — speed, sensitivity, WASD movement
- [ ] Controllers consume `PointerState` + `KeyboardState` from `scenix-input`
- [ ] Tests: frustum correctly culls points outside the view volume
- [ ] Tests: orbit controller clamps polar angle to [min, max]

---

## v0.6.0 — Renderer

**Goal:** GPU rendering via wgpu. A developer can render a scene with PBR materials, shadows, and deferred+forward pipeline.

### Crates shipped

- `scenix-renderer` `v0.6.0` (new)

### Deliverables

**`scenix-renderer`**
- [ ] `Renderer` — owns `wgpu::Device`, `Queue`, `Surface`, `PipelineCache`, `GpuScene`
- [ ] `RendererConfig` — width, height, sample_count, vsync, hdr, present_mode, backends
- [ ] `Renderer::new(window, config)` — async initialization
- [ ] `Renderer::render(&scene, &camera)` — full frame render
- [ ] `Renderer::resize(w, h)` — surface reconfiguration
- [ ] `GpuMaterial` trait — `bind_group_layout()`, `to_uniform_bytes()`, `create_bind_group()`
- [ ] `GpuMaterial` impls for `PbrMaterial`, `UnlitMaterial`, `LambertMaterial`
- [ ] `PipelineCache` — keyed by `PipelineKey`, lazy compilation
- [ ] `GpuScene` — uploads scene graph transforms to GPU storage buffers
- [ ] `FrameContext` — per-frame uniform buffers (camera VP, time, resolution)

**Render passes**
- [ ] `shadow_pass.rs` — depth-only pass for shadow maps, `ShadowMapAtlas`
- [ ] `geometry_pass.rs` — G-buffer (albedo, normal, depth, metallic-roughness)
- [ ] `lighting_pass.rs` — deferred lighting resolve full-screen quad
- [ ] `forward_pass.rs` — forward+ for transparent objects, depth-sorted
- [ ] `culling.rs` — frustum culling using scene BVH
- [ ] `sort.rs` — back-to-front sort for transparent objects

**Shaders (WGSL)**
- [ ] `pbr.vert.wgsl`, `pbr.frag.wgsl` — PBR vertex/fragment shaders
- [ ] `unlit.frag.wgsl` — unlit fragment shader
- [ ] `shadow_depth.vert.wgsl` — shadow pass vertex shader
- [ ] `deferred_resolve.wgsl` — deferred lighting full-screen quad

- [ ] Tests: pipeline cache returns same pipeline for same `PipelineKey`
- [ ] Tests: headless render produces non-black framebuffer
- [ ] `benches/render_bench.rs` — frame time with 1K / 10K / 100K triangles
- [ ] `examples/hello_cube.rs` — rotating box with unlit material
- [ ] `examples/pbr_sphere.rs` — PBR sphere with IBL
- [ ] `examples/shadow_demo.rs` — directional light + PCF shadows

---

## v0.7.0 — Loaders & Post-Processing

**Goal:** Asset loading and post-processing effects. A developer can load GLTF files and apply bloom, SSAO, tone mapping, and other effects.

### Crates shipped

- `scenix-loader` `v0.7.0` (new)
- `scenix-post` `v0.7.0` (new)

### Deliverables

**`scenix-loader`**
- [ ] `GltfLoader::load(path) -> Result<SceneGraph>` — meshes, materials, textures, hierarchy
- [ ] `GltfLoader::load_url(url) -> Result<SceneGraph>` — async WASM-compatible
- [ ] `obj::load(path) -> Result<Vec<Geometry>>` — OBJ + MTL parsing
- [ ] `stl::load(path) -> Result<Geometry>` — binary + ASCII STL
- [ ] `image::load(path) -> Result<Texture2D>` — PNG, JPEG, WebP, KTX2
- [ ] `hdr::load(path) -> Result<TextureCube>` — HDR/EXR → cubemap for IBL
- [ ] `AssetCache` — dedup by path, reference counting, optional hot-reload
- [ ] Tests: round-trip load of reference GLTF assets (BoxTextured, DamagedHelmet)
- [ ] Tests: OBJ vertex count matches expected

**`scenix-post`**
- [ ] `PostStack` — ordered chain of effects, builder pattern
- [ ] `bloom.rs` — threshold, intensity, radius, blur passes (dual kawase)
- [ ] `ssao.rs` — screen-space ambient occlusion (kernel samples + blur)
- [ ] `tonemap.rs` — `ToneMapper` enum: ACES, Reinhard, Filmic, AgX
- [ ] `fxaa.rs` — fast approximate anti-aliasing
- [ ] `taa.rs` — temporal anti-aliasing with jitter matrix
- [ ] `smaa.rs` — enhanced subpixel morphological AA
- [ ] `dof.rs` — depth of field (aperture, focus distance, bokeh)
- [ ] `fog.rs` — volumetric fog (exponential, height-based)
- [ ] `outline.rs` — selected object outline highlighting
- [ ] `motion_blur.rs` — per-object motion blur via velocity buffer
- [ ] Tests: PostStack applies effects in correct order
- [ ] `examples/post_processing.rs` — full stack: SSAO + Bloom + ToneMap + TAA
- [ ] `examples/gltf_scene.rs` — load and display a GLTF file

---

## v0.8.0 — Raycasting & Helpers

**Goal:** BVH-accelerated raycasting and debug visualization. A developer can pick objects with mouse and visualize scene structure.

### Crates shipped

- `scenix-raycaster` `v0.8.0` (new)
- `scenix-helpers` `v0.8.0` (new)

### Deliverables

**`scenix-raycaster`**
- [ ] `Raycaster` — `cast_ray(scene, ray) -> Option<Intersection>`
- [ ] `Raycaster::cast_ray_all(scene, ray) -> Vec<Intersection>` — all hits, sorted by distance
- [ ] `Raycaster::from_camera_ndc(camera, ndc_x, ndc_y) -> Ray3`
- [ ] `Intersection` — `node_id`, `distance`, `point`, `normal`, `uv`
- [ ] `Bvh` — SAH-based build from scene AABB list
- [ ] `Bvh::traverse(ray) -> Vec<NodeId>` — candidate list
- [ ] Tests: ray-AABB, ray-triangle, ray-sphere intersection correctness
- [ ] Tests: BVH produces same results as brute-force (correctness proof)
- [ ] `benches/bvh_bench.rs` — BVH build + 1K ray queries

**`scenix-helpers`**
- [ ] `GridHelper` — `to_geometry()` → line-list grid plane
- [ ] `AxesHelper` — `to_geometry()` → RGB XYZ axis lines
- [ ] `BoundingBoxHelper` — wireframe AABB for a node
- [ ] `ArrowHelper` — directional arrow with configurable head
- [ ] `SpotLightHelper`, `PointLightHelper`, `DirectionalLightHelper`
- [ ] `CameraHelper` — frustum wireframe visualization
- [ ] `SkeletonHelper` — bone visualization
- [ ] `examples/raycasting.rs` — mouse picking with BVH
- [ ] `examples/helpers_demo.rs` — all helpers in one scene

---

## v0.9.0 — Integration

**Goal:** Connect scenix to animato and the browser. A developer can animate scene properties with springs/tweens and run scenix in a web page.

### Crates shipped

- `scenix-animato` `v0.9.0` (new)
- `scenix-wasm` `v0.9.0` (new)

### Deliverables

**`scenix-animato`**
- [ ] `NodeAnimator` — binds `Tween`/`Spring` to `NodeId` transform
- [ ] `NodeAnimationTarget` enum — `Translation`, `Rotation`, `Scale`, `Visibility`
- [ ] `CameraAnimator` — animates fov, position, target
- [ ] `MaterialAnimator` — animates albedo, opacity, emissive
- [ ] `SkinnedMeshAnimator` — drives bone transforms from keyframe data
- [ ] `scenixAnimationDriver` — ticks all bound animators per frame
- [ ] Tests: tween drives node position from A to B correctly

**`scenix-wasm`**
- [ ] `WebRenderer` — wraps `Renderer` for `<canvas>` + `requestAnimationFrame`
- [ ] `WebRenderer::new(canvas) -> Result<WebRenderer, JsValue>` — async init
- [ ] `WebRenderer::tick(timestamp_ms)` — called from rAF
- [ ] `WebRenderer::resize(w, h)`
- [ ] `on_pointer_move/down/up`, `on_wheel` — DOM input forwarding
- [ ] `examples/wasm_viewer/` — GLTF viewer in browser
- [ ] `examples/animato_integration.rs` — spring camera + tween material
- [ ] `examples/orbit_camera.rs` — OrbitController with mouse input

---

## v1.0.0 — Stable

**Goal:** API freeze. Every public item is documented, every example compiles, every feature has tests, CI is fully green on stable + beta + nightly.

### Deliverables

**API Stability**
- [ ] Review every `pub` item — deprecate or stabilize
- [ ] `#[deprecated]` on anything being removed before 1.0
- [ ] No `pub` item without a `///` doc comment and a runnable example

**Documentation**
- [ ] `docs/` folder with:
  - [ ] `getting-started.md` — 5-minute guide from `cargo add` to first render
  - [ ] `concepts.md` — explains SceneGraph, Material, Renderer, PostStack
  - [ ] `materials-guide.md` — visual comparison of all material types
  - [ ] `platform-guide.md` — how to target desktop, mobile, web, embedded
  - [ ] `benchmarks.md` — current benchmark results
- [ ] `cargo doc --all-features` renders zero warnings
- [ ] All examples compile and run

**Testing**
- [ ] ≥ 90% test coverage via `cargo-llvm-cov`
- [ ] Integration test for every platform target (desktop, WASM)
- [ ] Headless GPU tests for renderer correctness
- [ ] Snapshot tests for primitive geometry (vertex counts, normal validity)

**CI**
- [ ] `stable`, `beta`, `nightly` all green
- [ ] WASM build (`wasm-pack test --headless --chrome`) green
- [ ] `no_std` compile check green for math + core + input
- [ ] Clippy `--all-features -- -D warnings` green
- [ ] `cargo fmt --check` green
- [ ] Benchmark regression check — fail if render perf drops > 10%

**Release**
- [ ] `CHANGELOG.md` complete — every change from 0.1.0 → 1.0.0 documented
- [ ] GitHub Release with prebuilt WASM demo hosted on GitHub Pages
- [ ] Announcement post on r/rust and Dev.to

---

## Post-1.0 Ideas (Future / `v1.x`)

These are not committed — they are ideas to revisit after the stable release.

| Idea | Notes |
|------|-------|
| `scenix-audio` | Positional 3D audio via `kira` or `rodio` |
| `scenix-physics` | Collision detection + rigid body via `rapier3d` bridge |
| `scenix-xr` | WebXR / OpenXR support for VR/AR |
| `scenix-particles` | GPU particle system with compute shaders |
| `scenix-terrain` | Height-map terrain with LOD chunking |
| `scenix-sky` | Procedural sky + atmosphere scattering |
| `scenix-water` | Water surface with reflection/refraction |
| `scenix-egui` | egui overlay integration for debug UI |
| `scenix-editor` | Visual scene editor built with Tauri + scenix |
| MatcapMaterial | Matcap material type for stylized rendering |
| GPU-driven rendering | Indirect draw calls, mesh shaders, GPU culling |
| Cluster forward+ | Clustered light assignment for forward rendering |
| Cascaded shadow maps v2 | SDSM (sample distribution shadow maps) |
| Realtime GI | Screen-space global illumination (SSGI) |

---

## Contributing to scenix

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for how to set up the workspace, run tests, and submit pull requests.

The best way to contribute right now is to pick any unchecked item from `v0.1.0` above and open a PR.

---

*Roadmap version: 0.1.0 — last updated May 2026*
*Next milestone: v0.1.0 — Foundation*
*Project: Aarambh Dev Hub — github.com/AarambhDevHub/scenix*
*Companion library: animato — github.com/AarambhDevHub/animato*
