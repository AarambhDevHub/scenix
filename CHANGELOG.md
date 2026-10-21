# Changelog

All notable changes to scenix will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.9.1] - 2026-10-21

### Fixed

- Bumped the upstream Animato bridge dependency from `1.7.2` to `1.7.3`
  (AarambhDevHub/animato#56): the Animato facade now declares
  `#![cfg_attr(not(feature = "std"), no_std)]`, so `scenix-animato` with the
  `animato` facade feature and `std` disabled compiles for `no_std`
  bare-metal targets (verified for `thumbv7m-none-eabi`; scenix issue #21).
  The upstream `timeline` and `path` facade features remain `std`-gated in
  Animato itself, unchanged in this release.
- Updated the `XR_VERSION`, `AUDIO_VERSION`, and `PHYSICS_VERSION` crate
  version markers to `1.9.1`.
- Refreshed the Animato version references across the README feature
  flags/table, the ARCHITECTURE `scenix-animato` section, and the animation
  concepts and FAQ docs to the new `1.7.3` bridge target.

## [1.9.0] - 2026-10-20

### Added

- Added the optional `scenix-xr` crate (`no_std`-capable on `std`/`serde`,
  depending on core/input/math/camera): `XrSessionPlan` validation with
  `XrSessionPlanError` diagnostics and the explicit
  Idle→Requesting→Starting→Active→Paused/Ending→Ended lifecycle
  (`XrSessionState::can_transition`); `XrControllerBay` pointer/grip pose
  tracking for both hands with kinematic prediction; `XrHandFrame` over the
  26-slot hand profile (`JOINT_WRIST`, `JOINT_THUMB_TIP`, `JOINT_INDEX_TIP`,
  `XR_HAND_JOINT_COUNT`) with pinch distance and strength classifiers;
  `XrAnchorStore` age-pruned anchors, `XrPlaneStore` plane reconciliation,
  `XrHitTestProbe` floor/wall/window casts, and ambient/light-estimation
  planning; `XrFramePlan` frame stats with the swapchain `XrViewport`
  (`XR_MAX_VIEWPORT_SCALE` clamp) and `XR_VERSION`.
- Added the optional `scenix-audio` crate (core/math): `AudioListener`
  positional/orientation state (`AUDIO_LISTENER_MAX_SOURCES`), the
  `compute_spatial_decision` spatial mix folding pan weights
  (`AttenuationModel::{None, Linear, Inverse, InverseSquare}` with gain
  clamps, `SpeakerLayout`, `blend_pan`), `DopplerModel::shift` over
  `SPEED_OF_SOUND` clamped to `0.25..=4`, `AudioSource`/`AudioLoop`
  pass scheduling (`has_next_pass`, `total_passes`), `AudioStreamPlan`
  chunk-buffer health (`accept_chunk`, `promote_pending`, `consume_chunk`,
  underrun accounting, `STREAM_MAX_CHUNKS`), and `AudioAnalyser` RMS/peak
  metering with three energy bands (`AnalyserBand::{Bass, Mid, Treble}`,
  `ANALYSER_MIN_FFT_SIZE..ANALYSER_MAX_FFT_SIZE` snapping).
- Added the optional `scenix-physics` crate (core/math/mesh): the
  deterministic penalty-based rigid-body solver — `PhysicsWorld::earth`
  O(n²) AABB broad phase with `ColliderGroup` filtering, analytic narrow
  phase over sphere/box/capsule/plane pairs (world-axis-aligned),
  restitution with Coulomb friction plus the Baumgarte positional push
  (`CONTACT_RESOLVE_PERCENT`, `CONTACT_SLOP`), real substeps (`MAX_SUBSTEPS`
  at `≤ MAX_SUBSTEP_DT`), speed clamping, and frame-threshold sleeping with
  `WorldEvent::{ContactStarted, ContactEnded, BodySlept, BodyWoke}`;
  sensor collider support via `PhysicsWorld::insert_collider`; spatial
  queries (`cast_ray`, `overlap_sphere`); the kinematic capsule
  `CharacterState::move_and_slide` controller (slope limits, step offsets,
  jump launch, probe-based ground snapping with
  `PHYSICS_GROUND_REATTACH_SPEED`); and `PhysicsTransformSync` mapping body
  transforms onto scene node keys.
- Added `scenix-core` `XrSessionId`, `XrAnchorId`, `AudioSourceId`,
  `AudioListenerId`, `PhysicsWorldId`, `PhysicsBodyId`, and `ColliderId`
  newtypes; added `scenix-scene` `NodeKind::{XrAnchor, AudioSource,
  PhysicsBody}` rider nodes with constructors and inspector labels; the
  renderer culling pass treats them as CPU-side payloads that submit no
  draws.
- Added `scenix-camera` stereo support: `XrFov` four-angle eye frusta,
  `XrEyeProjection` (WebXR asymmetric layouts plus a symmetric helper) with
  WebGPU-convention projection matrices, and `XrCameraRig` head/eye
  coupling into left/right view and view-projection matrices.
- Added the `scenix-wasm` `xr` feature: `webxr_fallback_plan(granted)` →
  `WebXrFallbackPlan` (`RuntimeGranted` vs `EmulatedDesktop`),
  `emulated_headset_rig` (standing-desk stereo rig at 64 mm separation),
  and `emulated_controller_pose` for missing-`navigator.xr` demos.
- Added `scenix-helpers` debug surfaces: always-on `PoseAxesHelper`
  (oriented triad plus `-Z` forward stick) and `HandSkeletonHelper`
  (`HAND_LINKS` over the 26-slot hand profile), and the `physics` feature's
  `ColliderDebugHelper` wireframe pass (`debug_wireframe` for one shape,
  `debug_wireframe_world` for the world, sensor/solid color split).
- Added `scenix-math::float` `floor`/`ceil`/`round` shims (pure arithmetic,
  NaN/inf passthrough, 2^24 integral passthrough) so the physics substep
  clock and `XrViewport` scaling round identically on every target.
- Added the facade `xr`, `audio`, `physics`, and `bridges` features with
  curated bridge re-exports (the crates' module namespaces would shadow the
  scene/input globs), the facade `scenix_v19` test suite, and the
  `xr_viewer`, `xr_controllers`, `audio_positions`, `physics_sync`, and
  `character_controller` examples.
- Added CI/publish lanes for the bridge surface: no-default CPU crate
  tests for the three crates, the v1.9 facade feature checks and test
  surface, bridge example checks, and the publish order placement for
  `scenix-xr`/`scenix-audio`/`scenix-physics` after `scenix-camera` and
  `scenix-mesh`.

## [1.8.0] - 2026-10-19

### Added

- Added the optional `scenix-terrain` crate (`no_std`-capable on
  `std`/`serde`, depending on core/math/mesh): seeded deterministic
  `Heightmap::from_noise` plus `from_fn` with `remap_range` refitting,
  `TerrainGrid::generate` chunking from `TerrainConfig` (chunks per axis ×
  chunk cells × `LodBand` policy) with per-LOD `chunk_geometry` and
  `chunk_geometry_with_splat` weight packing, `SplatLayer`/`SplatMap`
  normalization, `TerrainCollider` ray probes (`with_step`, `with_max_steps`,
  `height_at`, `normal_at`), and a deterministic `ChunkStreamer`
  poll/unload loop over wrapping `ChunkCoord` lattices.
- Added the optional `scenix-sky` crate (core/light/math): `SkyModel`
  profiles (`clear_day`, `overcast_haze`) with turbidity/rayleigh/mie tuning
  and pure direction taps, `SunPosition` (degrees/radians) driving
  `SunLight`/`DirectionalLight`, deterministic `bake_equirectangular` (row 0
  = zenith), `bake_light_probe` SH taps, `bake_ambient` sky/ground averages,
  and `GroundedSkybox` horizon fades.
- Added the optional `scenix-water` crate (camera/core/math/mesh):
  `WaveField` presets (`ocean_shore`, `calm`) holding up to `MAX_WAVES`
  stacked `GerstnerWave`s, `WaterSurface::geometry(t)` displaced meshes that
  match `height_at` exactly with unit normals, `WaterConfig`, `FoamParams`
  threshold gating, `FresnelParams` Schlick reflectance, `UnderwaterView`
  red-first absorption with murk drift, and planar reflection helpers
  (`mirror_transform`, `reflected_camera`, `ReflectionSetup`).
- Added `scenix-material` environment descriptors: `TerrainMaterial` (four
  splat layers with albedo/roughness/tiling plus optional splat and detail
  textures), `SkyMaterial` (gradient triple, sun direction/color/strength,
  fade band, atmosphere flag), and `WaterMaterial` (deep/shallow tints,
  Schlick base reflectance, optional foam threshold, optional
  reflection/refraction textures); `ShaderKind` gained the `Terrain`, `Sky`,
  and `Water` variants plus the `FEATURE_SPLAT_MAP` (1 << 22),
  `FEATURE_ATMOSPHERE` (1 << 23), `FEATURE_FRESNEL` (1 << 24),
  `FEATURE_FOAM` (1 << 25), and `FEATURE_REFRACTION` (1 << 26) bits.
- Added the `scenix-renderer` `environment` feature: an environment slot
  registry keyed by node material ids
  (`register_environment`/`unregister_environment` over
  `EnvironmentMaterial::{Terrain, Sky, Water}`), per-kind uniform packing
  into a dedicated 128-byte environment belt, three WGSL pipelines
  (`terrain_splat`, `sky_dome`, `water_surface`), sky → opaque → terrain →
  transparent → water draw ordering, dirty-flag bind-group rebuilds, and
  `scene_environments`/`visible_environments` counters in `CullingStats` and
  `FrameStats`.
- Added `scenix-core` `TerrainId`, `SkyId`, and `WaterId` newtypes; added
  `scenix-scene` `NodeKind::{Terrain, Sky, Water}` nodes with constructors
  and inspector labels; added `scenix-texture`
  `EquirectSampler`/`height_to_normal_map` (moved under the
  `environment_maps` module before release); added `scenix-light`
  `SunLight::from_direction`/`from_elevation`, `sun_direction`, and
  `sun_color_from_elevation` in `sun.rs`.
- Added `scenix-wasm` `environment_fallback_plan(WebGlCapabilityLevel)` →
  `EnvironmentFallbackPlan`: `Full` WGSL parity on WebGPU, `StaticUniforms`
  on WebGL 2, and CPU-baked `VertexColorTint` without screen-space water on
  WebGL 1.
- Added the facade `terrain`, `sky`, `water`, and `environment` features
  with curated terrain/sky re-exports (the `lod`/`sky`/`sun` module names
  collide with the scene, material, and light crate globs), the facade
  `scenix_v18` test suite, and the `terrain_lod`, `sky_sun`,
  `water_fresnel`, and `environment_demo` examples plus the
  `benches/terrain_bench.rs` hot paths.
- Added CI/publish lanes for the environment surface: no-default CPU crate
  tests, v1.8 facade feature checks, the v1.8 environment test surface, and
  the GPU-gated renderer `environment` lane (lavapipe).

## [1.7.0] - 2026-10-18

### Added

- Added the optional `scenix-particles` crate (`no_std`-capable):
  `ParticleEmitter` configuration, deterministic seeded simulation over
  pools up to `PARTICLE_POOL_LIMIT` (262,144 particles), burst and rate
  spawning, per-particle records (`position`, `velocity`, `age`, `lifetime`,
  `size`, `rotation`, flipbook frame), and one reusable
  `ParticleRenderBatch` per `ParticleSystem::update` call.
- Added the modifier vocabulary behind `ParticleModifier`: `Gravity`, `Drag`,
  `ColorOverLifetime` over deterministic `ColorCurve`/`ColorKey` data,
  `SizeOverLifetime`, `SpinOverLife`, and `ConfineBox`; `ScalarCurve`
  covers size/rotation falloffs. `EmissionShape` covers point, `Sphere`,
  `Hemisphere`, `Disc`, `Cone`, and `Box` spawns with an `at`/`toward`
  builder shortcut.
- Added `scenix-texture::FlipbookGrid`: shared frame-index → UV math for
  flipbook sheets (`columns × rows`, top-left origin, clamp on the last
  frame, `uv`/`uv_frames`), mirrored on the particle side by
  `FlipbookConfig` with fps/looping.
- Added `scenix-mesh::quad_geometry`: the CCW +Z billboard quad with the
  scenix plane UV convention for sprite draws and the WebGL-expanded
  fallback.
- Added `scenix-material::ParticleMaterial` (tint, optional sprite texture,
  optional flipbook, soft edge, `ParticleBlend::{Alpha, Additive}`), the
  `FEATURE_SOFT_PARTICLES` (`1 << 20`) and `FEATURE_FLIPBOOK` (`1 << 21`)
  bits, and `ShaderKind::Particles` for the billboard pipeline key.
- Added the scenix-renderer GPU particle path (feature `particles`): a
  per-system `ParticleRegistry` with draw + compute slots, an instanced
  64-byte billboard instance record (center/size, premultiplied color, UV
  rect, rotation), shared frame + per-material sprite bind groups, and
  `Additive`/`Alpha` pipelines with depth-test-on/depth-write-off blending.
  `Renderer::register_particles`/`update_particles` upload CPU batches;
  `scenix-wasm` particle capability notes document the WebGL fallback plan.
- Added the optional compute simulation: `Renderer::register_particle_compute`
  seeds a 64-byte-per-slot GPU storage pool from a CPU snapshot,
  `simulate_particles_compute` dispatches an integrate pass (gravity, drag,
  size/color fade over `ParticleComputeParams`) and a deterministic
  single-invocation ring-window respawn pass, and
  `read_particle_pool`/`read_particle_ring` copy results to the host. CPU
  updates always convert a slot back to the draw path.
- Added `scenix_math::float`: the public `libm`-compatible float shim module
  (`sqrt`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan2`, `cbrt`, plus
  `PI`/`FRAC_PI_2`) so CPU sims compile identically on std and `libm`
  builds.
- Added `NodeKind::Particles { system_id }` (with the `SceneNode::particles`
  constructor and inspector `"particles"` label), the `ParticleSystemId`
  core newtype, and the facade `particles` feature with curated re-exports.
- Added `tests/scenix_v17.rs` (facade CPU coverage), `tests/particles_gpu.rs`
  (GPU-gated billboard + compute lifecycle), and the `particles_basic`
  example with an optional headless compute tail.

## [1.6.0] - 2026-10-17

### Added

- Added the optional `scenix-nodes` crate (`no_std`-capable): typed shader
  graphs with three output ports (`fragment_color`, `fragment_alpha`, and a
  vertex `vertex_offset` displacement port), `ShaderNodeKind` node vocabulary,
  `NodeInput`/`NodeValue` typed values, stage gating (`Stage::Vertex`,
  `Stage::Fragment`, `Stage::Post`) that renders fragment-only or
  post-only nodes as specific `NodeDiagnostic`s, validation over the live
  subgraph only, dead-node release, determinism-ordered ABA problem checks,
  and `NODE_GRAPH_LIMIT` bounds.
- Added the material node vocabulary: uniforms (`BaseColor`, `Emissive`,
  `Metallic`, `Roughness`, `Time`), vertex attributes, world/view fragment
  sources, texture sampling, splat/component shaping, arithmetic and
  `Min`/`Max`/`Dot`/`Cross`/`Power`/`Step`, `Mix`/`Clamp`/`Smoothstep`, five-slot
  `Remap`, `Fresnel`, sRGB/linear conversions, lighting styles (Lambert, Phong,
  Toon, Cel), tone mappers, and fog; color-like nodes accept rgba base colors
  and drop alpha in codegen.
- Added the WGSL backend: generated modules follow the renderer bind contract
  (group 0 frame, group 1 object, group 2 material with uniform/sampler/
  texture_2d, group 3 fragment-only lights) with six varyings matching the
  built-in vertex layout, `Time` resolved to the frame elapsed slot, and
  common-subexpression elimination plus dead-node pruning in codegen.
- Added the GLSL ES 3.00 backend with flattened plain uniforms for WebGL
  tooling. Both backends duplicate the `Stage` availability matrix so graphs
  cannot reach a backend with a stage-incompatible node list.
- Added `NodeMaterial`: graph plus CPU state (base color, emissive, metallic,
  roughness, alpha mode/cutoff, double-sided, texture id), `serde` support,
  `FEATURE_NODE_GRAPH` (`1 << 19`), a `Custom` shader-kind pipeline key, and a
  stable graph hash that ignores live CPU state.
- Added `PostNodeEffect` post graphs with atomic full-bandwidth presets
  (`passthrough`, `vignette`, `desaturate`, `posterize`, `pixelate`,
  `color_grade`, `exposure`), shared stable parameter slots in the
  `PostParams`/`[f32; 8]` contract, and post WGSL/GLSL generators.
- Added `scenix-renderer` node integration (behind its `nodes` feature):
  feature-free `NodeMaterialSlot` registry (`register_node_material`,
  `unregister_node_material`, `node_material`, `node_material_count`),
  per-draw node pipeline creation and `set_pipeline` switching, node uniform
  packing in the standard material block, transparent bucketing from the slot's
  alpha mode, and rejection of unregistered node materials during culling.
- Added `scenix-post::NodeEffectProgram` (behind the `nodes` feature): owns one
  generated post module, lazily builds per-format pipelines, runs the
  `[f32; 8]` parameter contract with a full-screen triangle pass.
- Added `scenix-wasm` browser preflight (behind the `nodes` feature): compile
  a serialized `NodeMaterial` into GLSL and a diagnostic payload
  (`compile_node_material_glsl`) or validate-only (`validate_node_material_json`)
  before anything is uploaded.
- Added the facade `nodes` feature with curated re-exports (node vocabulary,
  materials, generators, diagnostics, slots) and feature forwarding over
  `renderer`, `post`, and `wasm`.
- Added `examples/nodes_material.rs`, `examples/nodes_wgsl.rs`, and
  `examples/node_post_effect.rs`, the `tests/scenix_v16.rs` facade surface
  suite, GPU-gated `tests/nodes_gpu.rs` (headless node rendering, mixed
  node/builtin draws, culling rejection, and registration validation), and
  `benches/nodes_bench.rs` for graph build/lower hot paths.
- Added `docs/api/scenix-nodes.md` and `.github/release-notes/1.6.0.md` as the
  GitHub Release body.

### Changed

- Bumped all 18 workspace crates and internal dependency requirements to
  `1.6.0`; Rust 1.98, edition 2024, and existing default features are retained.
- Bumped Animato to `1.7.2`; the previous `1.7.0` gate is superseded with no
  scenix-side code changes.
- `scenix-renderer`'s frame uniform now packs resolution as
  `[width, height, elapsed seconds, 1 / height]`; node shaders read `Time`
  from `frame.resolution.z`, and the renderer tracks elapsed time per surface.
- CI and publish workflows gained node-surface lanes (tests, facade feature
  lanes, `scenix-nodes` in publish order after `scenix-material`, and the
  release title).
- Website and standalone WASM viewer manifests now match the workspace release.

### Compatibility and Migration Notes

- Everything v1.6 is additive; no existing enum gained variants and default
  feature sets are unchanged, so `scenix = "1"` users see identical behavior
  without `nodes`.
- Node graphs that fail produce a typed `NodeDiagnostic` (`StageUnsupported`,
  port mismatches, cycles, `UnknownNode`, ...) instead of a panicking
  pipeline; `NodeMaterial::validate` covers the fragment stage and
  `ShaderGraph::validate(Stage::Vertex)` covers displacement ports explicitly.
- The frame uniform layout change affects direct `FrameUniform` consumers
  outside scenix; built-in material shaders are unaffected.

## [1.5.0] - 2026-07-14

### Added

- Added allocation-free aggregate input for touch, gestures, four standard
  gamepads, pointer lock, logical/physical viewport metrics, and transient
  keyboard/pointer transitions.
- Added Arcball, Trackball, Map, FirstPerson, and PointerLock controllers plus
  aggregate-input updates for existing Orbit and Fly controls.
- Added scene editor metadata, layer policies, deterministic selection state,
  transform modes/spaces/constraints, and translation/rotation/scale snapping.
- Added BVH reuse APIs, marquee selection for perspective and orthographic
  cameras, drag planes, reversible drag controls, and reversible transform
  controls.
- Added reusable transform/bounds/selection/snap-grid gizmos with analytic
  handle hit testing and retained line buffers.
- Added typed inspector snapshots for scenes, cameras, materials, lights,
  textures, animation mixers, renderer diagnostics, and GPU resources, plus an
  optional egui adapter pinned to `0.33.3`.
- Added on-demand renderer ID/normal/depth picking with dense `NodeId` mapping,
  scissored pixel requests, reusable GPU/readback resources, and world-position
  reconstruction.
- Added DPR-aware browser metrics and primitive forwarding for touch, pointer
  lock, gamepad, transform modes, selection, and inspector JSON.
- Added five v1.5 examples, focused integration tests, GPU picking coverage,
  interaction hot-path benchmarks, release documentation, and version-derived
  publishing automation.

### Changed

- Bumped all 17 workspace crates and internal dependency requirements to
  `1.5.0`; Rust 1.89, edition 2024, and existing default features are retained.
- `LineGeometry` and BVH/raycast paths now expose reusable-output methods to
  avoid repeated hot-loop allocations.
- Website and standalone WASM viewer manifests now match the workspace release.

### Compatibility and Migration Notes

- Existing v1.4 controller, scene, raycaster, and helper APIs remain available;
  v1.5 additions are additive.
- `editor` and `egui` are opt-in facade features. GPU picking requires the
  renderer/WebGPU path; WebGL continues to use CPU BVH picking.
- The egui integration renders the shared inspector model but deliberately does
  not provide the full editor shell planned for v1.10.

## [1.4.0] - 2026-07-09

### Added

- Added the clip-based Animation Runtime: `AnimationClip`, `AnimationAction`,
  `AnimationMixer`, `PropertyBinding`, `ClipTrack` / `ClipChannel`, `LoopMode`
  (Once / Repeat / PingPong), `AnimationMarker` / `AnimationEvent`, crossfade,
  additive blending, and deterministic per-tick sampling.
- Added keyframe tracks `KeyframeScalar`, `KeyframeVec3`, `KeyframeQuat`,
  `KeyframeColor`, and `KeyframeBool` with `Linear`, `Step`, and `CubicSpline`
  interpolation (quaternions use slerp + shortest-arc).
- Added `PropertyBinding` typed targets for nodes, bones, materials, cameras,
  lights, and morph weights, plus `BindingKey` for stable accumulator lookups.
- Added `LightAnimator`, `LightAnimationTarget`, `LightStoreMut`, and
  `LightStores` for animating point / spot / directional light color,
  intensity, range, and spot angle.
- Added `MorphWeightAnimator` and `MorphWeightStoreMut` for animating morph
  target weights.
- Added `RetargetMap` skeleton-bone retargeting by name.
- Added `scenix-mesh` skinning data model: `SkinningAttributes`, `SkinningData`,
  `MorphWeights`, `final_joint_matrices`, `cpu_skin`, and `apply_morph` CPU
  fallback.
- Added `scenix-renderer` GPU skinning + morph-upload hooks: `GpuSkinningRegistry`,
  `register_skin`, `update_bone_matrices`, `register_morph_targets`,
  `update_morph_weights`, and the `SKINNING_WGSL` snippet.
- Added `scenix-helpers` `AnimationPathHelper` and `PoseHelper` debug geometry.
- Added `scenix-loader` decoded animation accessor output bytes into
  `LoadedAnimationChannel::output`.
- Added facade `clip_from_loaded` bridge from imported clips to the runtime.
- Added `examples/animation_runtime.rs`, `examples/animation_mixer.rs`,
  `examples/skeleton_skinning.rs`, and `examples/animation_events.rs`.
- Added `tests/animation_runtime.rs` and `benches/animation_mixer_bench.rs`.
- Added `docs/release-v1.4.0.md` and `docs/examples/animation-runtime.md`.
- Added `.github/release-notes/1.4.0.md` as the GitHub Release body.

### Changed

- Bumped all workspace crates and internal dependency requirements to `1.4.0`.
- Bumped Animato to `1.7.0`; the previous `1.6.0` release gate is resolved.
- `ScenixAnimationDriver::tick` now accepts `lights` and `morphs` stores and
  prunes `light_animators` and `morph_animators` in addition to the existing
  node / camera / material / skeleton animators.
- `DriverStats` gained `light_animators` and `morph_animators` counters.
- `scenix-animato` now depends on `scenix-light` for the light animator.
- Updated README, architecture notes, roadmap, feature matrix, CI and publish
  workflows, examples, and release notes for the Animation Runtime release.

### Migration Notes

- `ScenixAnimationDriver::tick` signature now includes `lights` and `morphs`;
  pass empty `LightStores` / `BTreeMap<MeshId, Vec<f32>>` stores if unused.
- `LoadedAnimationChannel` gained an `output: Vec<f32>` field; update struct
  literals accordingly.
- Animato `1.5.0` → `1.7.0` is a drop-in for the `std` / `tween` / `spring` /
  `serde` feature set; no scenix-side code changes required.

## [1.3.0] - 2026-06-16

### Added

- Added `AssetPackage`, `AssetManager`, asset requests, async load handles,
  dependency graphs, diagnostics, memory budget accounting, and stale-cache
  invalidation for the v1.3 asset pipeline.
- Added `AssetId`, `SkinId`, and `AnimationClipId`.
- Added `GltfLoader::load_package_file`, `load_package_bytes`, and
  `load_package_url` for package imports while preserving `GltfAsset`.
- Added glTF package sidecars for skins, skin attributes, morph targets,
  animation clip metadata, material extension metadata, texture transforms,
  variants, KTX2/BasisU notes, Draco diagnostics, and meshopt diagnostics.
- Added exporter helpers for glTF summary JSON/GLB bytes, OBJ, STL, PLY, and
  scene JSON.
- Added `RendererAssetExt::register_asset_package` through the facade when
  `loader` and `renderer` are enabled.
- Added `asset_pipeline`, `asset_manager`, `export_scene`,
  `animation_import`, and `compressed_assets` examples.
- Added `.github/release-notes/v1.3.0.md` as the GitHub Release body.

### Changed

- Bumped all workspace crates and internal dependency requirements to `1.3.0`.
- Prepared release docs for Animato `1.6.0` as a release gate. Cargo remains on
  Animato `1.5.0` until `1.6.0` is published and resolves.
- Updated README, loader docs, feature matrix, workflows, examples, roadmap,
  and architecture notes for the Asset Pipeline release.

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

[Unreleased]: https://github.com/AarambhDevHub/scenix/compare/v1.9.1...HEAD
[1.9.1]: https://github.com/AarambhDevHub/scenix/compare/v1.9.0...v1.9.1
[1.9.0]: https://github.com/AarambhDevHub/scenix/compare/v1.8.0...v1.9.0
[1.8.0]: https://github.com/AarambhDevHub/scenix/compare/v1.7.0...v1.8.0
[1.7.0]: https://github.com/AarambhDevHub/scenix/compare/v1.6.0...v1.7.0
[1.6.0]: https://github.com/AarambhDevHub/scenix/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/AarambhDevHub/scenix/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/AarambhDevHub/scenix/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/AarambhDevHub/scenix/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/AarambhDevHub/scenix/compare/v1.1.0...v1.2.0
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
