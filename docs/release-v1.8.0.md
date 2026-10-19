# Scenix v1.8.0 — Terrain, Sky, And Water

Version 1.8 gives real-world environments first-class paths: three optional,
`no_std`-capable CPU crates (`scenix-terrain`, `scenix-sky`, `scenix-water`)
that simulate on the host as the source of truth, plus a GPU environment draw
path in `scenix-renderer` behind its `environment` feature and the material
descriptors that drive the WGSL pipelines. Authoring, headless tests, and GPU
uploads always agree.

## Data Flow

```text
 Heightmap (seeded noise / from_fn)          SkyModel (clear_day, haze)
        |                                            |
        v                                            v
 TerrainGrid::generate (chunks, LOD bands)   SunPosition (elevation/azimuth)
        |                                            |
        +--> chunk_geometry_with_splat               +--> bake_equirectangular
        |       (weights on RGBA channels)           |      (v = 0 zenith rows)
        |                                            +--> bake_light_probe / ambient
        +--> TerrainCollider probe (rays)
                                                     WaveField (Gerstner stack)
        ---- renderer environment slots ----                 |
        register_environment(keyed by MaterialId)            v
        |                                        WaterSurface::geometry(t)
        |                                            |  (matches height_at)
        |                                            v
        +--> sky draw  (depth-no-write dome)   SeaState → Material ids
        +--> opaque meshes                          |
        +--> terrain draw (splat blend)             v
        +--> transparent meshes                WaterMaterial (deep/shallow)
        +--> water draw (alpha blend,
             crest displacement on GPU)
```

The CPU crates never touch wgpu: terrain geometry, sky bakes, and water
surfaces are built and verified on the host before one byte uploads. The
renderer's `environment` feature owns the translation — pre-packed chunk
geometry uploads, procedural sky gradients, and a GPU wave paver that
replays the same GPU Gems math as `WaveField`.

## New Public Surfaces

- `scenix-terrain` (new crate): `Heightmap` (`from_fn`, `from_noise`,
  `remap_range`, `sample_bilinear`, `normal_at`), `TerrainGrid`
  (`generate`, `chunk_geometry`, `chunk_geometry_with_splat`,
  `level_for_distance`), `TerrainConfig`/`TerrainChunk`, `LodPolicy`/
  `LodBand`, `NoiseConfig`/`noise_heights`, `SplatLayer`/`SplatMap`,
  `TerrainCollider` (ray probes with `height_at`/`normal_at`), and
  `ChunkStreamer` (`poll` → `StreamDiff` over wrapping `ChunkCoord`
  lattices).
- `scenix-sky` (new crate): `SkyModel` (`clear_day`, `overcast_haze`,
  turbidity/rayleigh/mie tuning), `SunPosition` (`direction`,
  `sun_light`, `directional_light`), `bake_equirectangular`
  (deterministic, row 0 = zenith), `bake_light_probe` (sh_coefficients),
  `bake_ambient` (sky/ground hemisphere averages), and `GroundedSkybox`
  (dome fades below the horizon line).
- `scenix-water` (new crate): `WaveField` (`ocean_shore`/`calm` presets,
  `MAX_WAVES`), `GerstnerWave` (`wave_number`, `phase_speed`),
  `WaterSurface` (`with_config`, `geometry(t)`, `height_at`),
  `WaterConfig`, `FoamParams` (threshold gate), `FresnelParams` (Schlick
  reflectance), `UnderwaterView` (red-first absorption, murk drift), and
  the reflection helpers `mirror_transform`/`reflected_camera`/
  `ReflectionSetup`.
- `scenix-material`: `TerrainMaterial` (four splat layers, optional splat
  + detail textures), `SkyMaterial` (gradients, sun glow, fade band,
  atmosphere), `WaterMaterial` (deep/shallow tints, foam, reflection/
  refraction textures), `ShaderKind::{Terrain, Sky, Water}`, and the
  `FEATURE_SPLAT_MAP`/`FEATURE_ATMOSPHERE`/`FEATURE_FRESNEL`/
  `FEATURE_FOAM`/`FEATURE_REFRACTION` bits.
- `scenix-renderer` (behind `environment`): `register_environment` /
  `unregister_environment` over `EnvironmentMaterial::{Terrain, Sky,
  Water}`, three WGSL pipelines (`terrain_splat`, `sky_dome`,
  `water_surface`). Terrain blends four splat layers from vertex channels
  or the splat texture; the sky dome ignores depth
  (depth-no-write, compare-Always); water alpha-blends with the GPU wave
  paver and optional reflection/refraction textures. Draw order runs sky →
  opaque → terrain → transparent → water; `CullingStats`/`FrameStats`
  report scene_environments and visible_environments.
- `scenix-core`/`scenix-scene`: `TerrainId`/`SkyId`/`WaterId` newtypes,
  `NodeKind::{Terrain, Sky, Water}` with constructor (`SceneNode::terrain`,
  `sky`, `water`) and inspector labels.
- `scenix-texture`: `EquirectSampler` for baked sky maps and
  `height_to_normal_map` for heights.
- `scenix-wasm`: `environment_fallback_plan(WebGlCapabilityLevel)` →
  `EnvironmentFallbackPlan` (`Full` on WebGPU, `StaticUniforms` on
  WebGL 2, `VertexColorTint` on WebGL 1).
- `scenix` facade: the `terrain`, `sky`, `water`, and `environment`
  features (terrain/sky re-export lists are curated; `lod`/`sky`/`sun`
  module names collide with the scene, material, and light globs).

## First Render

```rust
use scenix::{
    EnvironmentMaterial, Heightmap, LodBand, LodPolicy, MaterialId, Renderer,
    RendererConfig, SceneGraph, SceneNode, SkyId, SkyMaterial, TerrainConfig,
    TerrainGrid, TerrainId, TerrainMaterial,
};

// 1. Terrain: seed, chunk, splat.
let map = Heightmap::from_fn(17, 17, 1.0, |ix, iz| ((ix + iz) as f32) * 0.5);
let policy = LodPolicy::new(&[LodBand::new(40.0, 0)])?;
let chunk = TerrainGrid::generate(&map, TerrainConfig::new(2, 8, policy))
    .chunk_geometry(0, 0)?;

// 2. Register the environment slots (terrain + sky, keyed by material id).
let terrain = TerrainMaterial::default_palette();
let sky = SkyMaterial::clear_day();
// renderer.register_environment(MaterialId::new(1), EnvironmentMaterial::Terrain(&terrain, &chunk))?;
// renderer.register_environment(MaterialId::new(2), EnvironmentMaterial::Sky(&sky))?;

// 3. Scene nodes key the slots through the schema.
let mut scene = SceneGraph::new();
scene.add(SceneNode::terrain("hills", TerrainId::new(1), MaterialId::new(1)));
scene.add(SceneNode::sky("backdrop", SkyId::new(1), MaterialId::new(2)));
scene.update_world_transforms();
// stats.scene_environments == stats.visible_environments == 2
```

Water adds `EnvironmentMaterial::Water(&WaterMaterial::clear(), &WaveField)`
and a `SceneNode::water` node; the GPU paver replays the wave math.

## Migration

No v1.7 API changed. `NodeKind` gained the additive `Terrain`, `Sky`, and
`Water` variants, which scene traversal and editor code already match
exhaustively in their own fallback arms. Default facade features are
unchanged, so existing apps compile identically. Terrain, sky, and water
materials live in `scenix-material` (available with the default features);
the depth-colliding module names stay out of the facade globs while the
typed surface arrives through `terrain`/`sky`/`water`.

## Validation

The release is covered by the `scenix-terrain`/`scenix-sky`/
`scenix-water` crate suites (deterministic noise replay, LOD resampling,
splat normalization, collider probes, wave math, foam/fresnel shapes,
underwater murk, and no_std lanes), `benches/terrain_bench.rs` hot paths,
the facade `scenix_v18` suite, the `terrain_lod`/`sky_sun`/
`water_fresnel`/`environment_demo` examples, and the GPU-gated
`environment_gpu` suite (uniform packing, draw counts, sky-dome pixel
probe, and slot rejection paths on lavapipe-friendly software platforms).
CI runs the facade feature lanes
(`--no-default-features --features terrain|sky|water|environment` with and
without `renderer`/`material`), no-default CPU builds, rustdoc
warnings-as-errors, packaging, and benchmark compilation.

Related reading:

- [scenix-terrain crate reference](api/scenix-terrain.md)
- [scenix-sky crate reference](api/scenix-sky.md)
- [scenix-water crate reference](api/scenix-water.md)
- [Examples](examples/README.md)
- [GitHub release notes](../.github/release-notes/1.8.0.md)
