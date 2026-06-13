# Changelog

All notable changes to scenix will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2026-06-13

### Added

- Added renderer-owned GPU texture upload for `Texture2D`, `TextureCube`, and
  `Texture3D`, including mip-aware byte ranges, sampler conversion, and
  compressed-format capability checks.
- Added additive renderer lifecycle APIs for updating, unregistering, and
  clearing texture resources, plus render-target creation, render-to-texture,
  texture readback, environment descriptors, diagnostics, resource stats, and
  pipeline-cache stats.
- Added renderer registration for hemisphere, area, and light-probe data
  alongside existing ambient, directional, point, and spot lights.
- Added `examples/render_target_capture.rs` and expanded renderer examples to
  exercise textured PBR, toon gradient textures, environment maps, and render
  targets.
- Added `docs/release-v1.2.0.md` with renderer parity migration notes and a
  small textured material code example.

### Changed

- Bumped all workspace crates and internal dependency requirements to `1.2.0`.
- Updated `scenix-animato` to use Animato `1.5.0`.
- Reworked the active renderer draw path to batch per-draw uniform writes and
  bind real material texture and light uniforms.
- Upgraded the browser fallback to prefer a real WebGL2 renderer path when
  WebGPU is unavailable, including texture sampling, material uniforms,
  directional/point lighting, toon/physical approximations, animation, picking,
  and explicit WebGL2/WebGL1 parity diagnostics.
- Updated README, API docs, examples docs, feature matrix, workflows, and
  GitHub Release automation for the Renderer And Material Parity release.

## [1.1.0] - 2026-05-31

### Added

- Added `scenix-wasm::BrowserRenderer` for automatic browser backend
  selection between WebGPU and WebGL.
- Added `scenix-wasm::WebGlRenderer` as a browser fallback renderer for
  generated Scenix Engine Lab scenes when WebGPU is unavailable or unsuitable.
- Added `BrowserBackendPreference` and `BrowserBackendKind` so applications can
  force WebGPU, force WebGL, or report the active backend.
- Added facade and `scenix-wasm` tests for the new browser backend enums.

### Changed

- Bumped all workspace crates and internal dependency requirements to `1.1.0`.
- Updated the website demo bridge to use `BrowserRenderer` and try WebGL before
  the Canvas2D fallback.
- Updated README, architecture notes, WASM docs, release notes, and feature
  matrix text for WebGPU-to-WebGL browser fallback.
- Updated CI, Pages, and publish website builds to pass `NO_COLOR=false` for
  Trunk compatibility.

## [1.0.0] - 2026-05-27

### Added

- Added the stable v1 documentation set under `docs/`, including getting
  started, concepts, materials, platform, benchmark, and release-note guides.
- Added a standalone Leptos CSR website under `website/` with a generated
  Scenix Engine Lab demo, controls, crate map, examples, SEO metadata, and
  GitHub Pages deployment support.
- Added the remaining architecture examples: physical material, toon shading,
  instancing, LOD, morph targets, fog, sprite particles, and environment maps.
- Added GitHub Pages, coverage, package-check, website-build, and release-note
  workflow coverage for the stable release.

### Changed

- Bumped all workspace crates and internal dependency requirements to `1.0.0`.
- Stabilized the renderer frame path around camera view-projection matrices,
  per-draw world matrices, reusable uniform buffers, material preview uniforms,
  and cached pipeline layouts.
- Expanded renderer material registration to cover PBR, Physical, Unlit,
  Lambert, Toon, Wireframe/debug, and Normal preview materials.
- Optimized scene transform propagation by deduplicating dirty roots and
  avoiding child-vector clones during subtree traversal.
- Expanded the browser wrapper with generated demo content, toggles, selection
  state, FPS/material getters, and non-panicking fallback behavior for the site.
- Rewrote README, architecture notes, roadmap, publish automation, and release
  notes for the stable v1 API contract.

### Migration Notes

- Replace `0.9` dependency requirements with `1`.
- Keep explicit feature flags for loader, renderer, post, Animato, and WASM
  integrations; those heavy paths remain optional.
- Renderer users can register advanced preview/debug materials through the new
  stable `register_*_material` methods.

## [0.9.0] - 2026-05-26

### Added

- Added the `scenix-animato` crate with Animato 1.4.0-compatible wrappers for
  `Vec3`, `Quat`, and `Color`, plus scalar, vector, quaternion, color, and
  boolean tracks.
- Added node, camera, PBR material, skeleton pose, and deterministic driver
  animation APIs that apply Animato tweens and springs to existing scenix data.
- Added the `scenix-wasm` crate with DOM key/pointer mapping helpers, panic
  hook setup, valid canvas-size clamping, and a browser `WebRenderer` wrapper
  around the existing renderer and a generated cube scene.
- Added `examples/animato_integration.rs`, `examples/wasm_viewer/`, and
  `benches/animato_bridge_bench.rs`.
- Added integration tests for node, camera, material, skeleton, driver, serde,
  WASM helper mappings, and facade exports.

### Changed

- Bumped all workspace crates to `0.9.0`.
- Updated the `scenix` facade crate with optional `animato` and `wasm` features
  while keeping v0.8 default CPU authoring, raycaster, and helper features
  unchanged.
- Updated README, roadmap, architecture notes, CI checks, publish ordering, and
  generated GitHub Release notes for the Integration release.

## [0.8.0] - 2026-05-26

### Added

- Added the `scenix-raycaster` crate with `Raycaster`, `Bvh`,
  `GeometryProvider`, camera NDC ray helpers, and exact world-space mesh
  triangle intersections.
- Added node-level SAH BVH build/traversal over visible scene mesh AABBs, with
  layer filtering and brute-force validation support.
- Added the `scenix-helpers` crate with validated `LineGeometry`, grid, axes,
  bounding box, arrow, light, camera, and skeleton debug helpers.
- Added `examples/raycasting.rs`, `examples/helpers_demo.rs`,
  `benches/bvh_bench.rs`, and `benches/helpers_bench.rs`.
- Added integration tests for ray primitives, camera rays, BVH-vs-brute-force
  picking, layer/visibility filtering, helper geometry validation, helper
  output counts, serde round trips, and facade exports.

### Changed

- Bumped all workspace crates to `0.8.0`.
- Updated the `scenix` facade crate to enable and re-export `scenix-raycaster`
  and `scenix-helpers` by default.
- Updated README, roadmap, architecture notes, CI checks, publish ordering, and
  generated GitHub Release notes for the Raycasting & Helpers release.

## [0.7.0] - 2026-05-26

### Added

- Added the `scenix-loader` crate with CPU-side glTF/GLB, OBJ/MTL, STL,
  PNG/JPEG/WebP, KTX2, HDR/EXR, and path-cache loading APIs.
- Added `GltfLoader`, `GltfAsset`, `LoadedCamera`, `LoadedLight`,
  `LoaderOptions`, and `AssetCache` for renderer-agnostic asset import.
- Added the `scenix-post` crate with `PostStack`, `PostEffect`, `PostTarget`,
  `PostContext`, bloom, SSAO, tonemap, FXAA, TAA, SMAA, depth of field, fog,
  outline, and motion blur configuration.
- Added optional renderer post-stack integration through
  `Renderer::set_post_stack`, `Renderer::post_stack`, and
  `Renderer::post_stack_mut`.
- Added `examples/gltf_scene.rs`, `examples/post_processing.rs`,
  `benches/loader_bench.rs`, and `benches/post_bench.rs`.
- Added integration tests for generated loader fixtures, image/KTX2/STL/OBJ
  parsing, cache behavior, post stack ordering, post config clamping, facade
  exports, and optional GPU post smoke coverage.

### Changed

- Bumped all workspace crates to `0.7.0`.
- Updated the `scenix` facade crate with optional `loader` and `post` features.
- Updated README, roadmap, architecture notes, CI checks, publish ordering, and
  generated GitHub Release notes for the Loaders & Post-Processing release.

## [0.6.0] - 2026-05-25

### Added

- Added the `scenix-renderer` crate with optional `wgpu` rendering,
  headless and surface targets, renderer-owned mesh/material/texture/light
  registries, frame stats, G-buffer targets, shadow-map atlas allocation, and
  render target resize support.
- Added `GpuMaterial` implementations for `PbrMaterial`, `UnlitMaterial`, and
  `LambertMaterial`, plus stable material uniform byte packing.
- Added `PipelineCache`, renderer pipeline keys, culling helpers, transparent
  and opaque draw sorting, and embedded WGSL shader entry points for the first
  renderer pass structure.
- Added renderer examples for a headless cube, PBR sphere, and shadow scene.
- Added CPU integration tests for renderer config validation, geometry packing,
  resource registry errors, format mapping, culling errors, sorting, material
  uniform bytes, facade exports, and renderer serde.
- Added GPU-gated tests for pipeline cache reuse, headless framebuffer smoke
  rendering, and resize target recreation.
- Added `benches/render_bench.rs` for 1K, 10K, and 100K triangle scene render
  submissions.

### Changed

- Bumped all workspace crates to `0.6.0`.
- Updated the `scenix` facade crate to expose renderer APIs behind the
  optional `renderer` feature while keeping default features CPU-only.
- Updated README, roadmap, architecture notes, CI checks, publish ordering, and
  generated GitHub Release notes for the Renderer release.

## [0.5.0] - 2026-05-23

### Added

- Added the `scenix-texture` crate with raw CPU `Texture2D`,
  `TextureCube`, `Texture3D`, `VideoTexture`, `Sampler`, deterministic
  `TextureAtlas` packing, `TextureFormat` byte-size helpers, and RGBA8 CPU
  mipmap generation.
- Added the `scenix-camera` crate with perspective, orthographic, and cube
  cameras, WebGPU-depth frustum extraction, screen-to-ray helpers, and orbit
  and fly controllers that consume `scenix-input` state.
- Added CPU-side examples for texture mipmaps with camera rays and orbit camera
  controls.
- Added integration tests for texture validation, atlas packing, mipmaps,
  video-frame updates, camera projection/view behavior, frustum visibility,
  cube camera matrices, controller clamps, facade exports, and serde round
  trips.
- Added compile-only benches for texture mipmap/atlas work and camera
  frustum/controller work.

### Changed

- Bumped all workspace crates to `0.5.0`.
- Updated the `scenix` facade crate to enable and re-export `scenix-camera`
  and `scenix-texture` behind the default `camera` and `texture` features.
- Updated README, roadmap, architecture notes, CI checks, publish ordering, and
  generated GitHub Release notes for the Textures & Camera release.

## [0.4.0] - 2026-05-20

### Added

- Added the `scenix-material` crate with the `Material` trait, compact
  `PipelineKey`, `AlphaMode`, PBR, physical, unlit, Lambert, toon, normal,
  wireframe, depth, line, points, and custom WGSL shader materials.
- Added the `scenix-light` crate with ambient, directional, point, spot,
  hemisphere, and area lights, validated `ShadowConfig`, and raw-sample
  spherical-harmonics `LightProbe` projection.
- Added CPU-side examples for material/light scene setup, material pipeline
  keys, and light probes.
- Added integration tests for material pipeline key uniqueness, alpha behavior,
  material/light serde round trips, light constructors, shadow validation, scene
  light attachment, and SH projection validation.

### Changed

- Bumped all workspace crates to `0.4.0`.
- Updated the `scenix` facade crate to enable and re-export `scenix-material`
  and `scenix-light` behind the default `material` and `light` features.
- Updated README, roadmap, architecture notes, CI checks, publish ordering, and
  generated GitHub Release notes for the Materials & Lights release.

## [0.3.0] - 2026-05-17

### Added

- Added the `scenix-mesh` crate with CPU-side `Geometry`, `Mesh`,
  `MorphTarget`, `InstancedMesh`, `BatchedMesh`, buffer layout metadata, and
  renderer-agnostic primitive generation.
- Added face-weighted normal generation, UV-derivative tangent generation with
  handedness, geometry bounds, validation, and indexed geometry merging.
- Added standard primitives: box, sphere, plane, cylinder, cone, capsule, torus,
  torus knot, icosphere, circle, ring, lathe, extrude, tube, and shape geometry.
- Added `Shape` support for exterior contours and hole side walls during
  extrusion.
- Added mesh integration tests for validation, normals, tangents, merging,
  bounds, instancing, batching, primitive validity, winding, UV ranges, facade
  exports, and serde round trips.
- Added `benches/mesh_gen_bench.rs` for primitive generation, tangent
  computation, and geometry merge throughput.

### Changed

- Bumped all workspace crates to `0.3.0`.
- Updated the `scenix` facade crate to enable and re-export `scenix-mesh` behind
  the default `mesh` feature.
- Updated README, roadmap, CI checks, publish ordering, and generated GitHub
  Release notes for the Geometry release.

## [0.2.0] - 2026-05-16

### Added

- Added the `scenix-scene` crate with a SlotMap-backed `SceneGraph`, graph-local
  `NodeId` handles, root management, parent-child hierarchy operations, and
  deterministic depth-first and breadth-first traversal.
- Added `SceneNode`, `NodeKind`, `Fog`, `Sprite`, `BillboardMode`, and
  `LodGroup` scene data types.
- Added dirty subtree world-transform propagation with cached `Mat4` world
  matrices and `Transform` world queries.
- Added Result-based hierarchy mutations for invalid IDs and cycle prevention.
- Added `no_std + alloc` support for `scenix-scene` with default `std`.
- Added scene graph integration tests covering hierarchy invariants, transform
  propagation, removal cascades, traversal order, reparenting, cycle prevention,
  scene support types, facade exports, and serde round trips.
- Added a compile-only 10K-node scene graph benchmark target.

### Changed

- Bumped all workspace crates to `0.2.0`.
- Updated the `scenix` facade crate to enable and re-export `scenix-scene` behind
  the default `scene` feature.
- Updated CI and publish workflows for the new scene crate.

## [0.1.0] - 2026-05-15

### Added

- Added the `scenix-math` crate with custom `no_std` scalar `f32` math:
  `Vec2`, `Vec3`, `Vec4`, `Mat3`, `Mat4`, `Quat`, `Euler`,
  `Transform`, `Ray3`, `Aabb`, `Sphere`, `Plane`, `Spherical`, and
  `Cylindrical`.
- Added optional `libm`, `serde`, and `approx` support for `scenix-math`.
- Added the `scenix-core` crate with typed IDs, `Color`, color-space helpers,
  error enums, and shared traits.
- Added optional `gpu` support for `scenix-core::GpuUpload`.
- Added the `scenix-input` crate with fixed-bitset `KeyboardState`,
  `PointerState`, `KeyCode`, `PointerButton`, and `Modifiers`.
- Added the `scenix` facade crate that re-exports the v0.1.0 Foundation APIs.
- Added unit tests for math operations, color conversions, ray intersections,
  bounds, transforms, and input state.
- Added facade and serde integration tests.
- Added a compile-only math benchmark target.
- Added v0.1.0-scoped CI and publish workflows.
- Rewrote the README to document only the shipped Foundation API surface.

[Unreleased]: https://github.com/AarambhDevHub/scenix/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/AarambhDevHub/scenix/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/AarambhDevHub/scenix/compare/v0.9.0...v1.0.0
[0.9.0]: https://github.com/AarambhDevHub/scenix/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/AarambhDevHub/scenix/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/AarambhDevHub/scenix/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/AarambhDevHub/scenix/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/AarambhDevHub/scenix/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/AarambhDevHub/scenix/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/AarambhDevHub/scenix/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/AarambhDevHub/scenix/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/AarambhDevHub/scenix/releases/tag/v0.1.0
