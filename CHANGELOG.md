# Changelog

All notable changes to scenix will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/AarambhDevHub/scenix/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/AarambhDevHub/scenix/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/AarambhDevHub/scenix/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/AarambhDevHub/scenix/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/AarambhDevHub/scenix/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/AarambhDevHub/scenix/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/AarambhDevHub/scenix/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/AarambhDevHub/scenix/releases/tag/v0.1.0
