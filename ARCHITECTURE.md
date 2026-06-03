# scenix — Full Project Architecture

> *Italian: scenix — scene, the stage on which everything appears.*
>
> A professional-grade, renderer-agnostic 3D scene library for Rust.
> Built as a clean Cargo workspace. Powered by `wgpu`. Animated by `animato`.
> Designed for games, creative tools, data visualization, native desktop, mobile, WASM browsers, and everything in between.

---

## Table of Contents

1. [Project Vision](#1-project-vision)
2. [Why a Workspace — Not a Single Crate](#2-why-a-workspace-not-a-single-crate)
3. [Workspace Layout](#3-workspace-layout)
4. [Crate-by-Crate Specification](#4-crate-by-crate-specification)
   - 4.1 [scenix-math](#41-scenix-math)
   - 4.2 [scenix-core](#42-scenix-core)
   - 4.3 [scenix-scene](#43-scenix-scene)
   - 4.4 [scenix-camera](#44-scenix-camera)
   - 4.5 [scenix-mesh](#45-scenix-mesh)
   - 4.6 [scenix-material](#46-scenix-material)
   - 4.7 [scenix-light](#47-scenix-light)
   - 4.8 [scenix-texture](#48-scenix-texture)
   - 4.9 [scenix-renderer](#49-scenix-renderer)
   - 4.10 [scenix-loader](#410-scenix-loader)
   - 4.11 [scenix-post](#411-scenix-post)
   - 4.12 [scenix-raycaster](#412-scenix-raycaster)
   - 4.13 [scenix-animato](#413-scenix-animato)
   - 4.14 [scenix-wasm](#414-scenix-wasm)
   - 4.15 [scenix-helpers](#415-scenix-helpers)
   - 4.16 [scenix-input](#416-scenix-input)
   - 4.17 [scenix (facade)](#417-scenix-facade)
5. [Data Flow & Render Loop](#5-data-flow--render-loop)
6. [Type System Design](#6-type-system-design)
7. [GPU Architecture](#7-gpu-architecture)
8. [Feature Flag Strategy](#8-feature-flag-strategy)
9. [Error Handling Strategy](#9-error-handling-strategy)
10. [Testing Strategy](#10-testing-strategy)
11. [Performance Guidelines](#11-performance-guidelines)
12. [Integration Targets](#12-integration-targets)
13. [CI / CD Pipeline](#13-ci--cd-pipeline)
14. [Publishing Checklist](#14-publishing-checklist)
15. [Naming & Style Conventions](#15-naming--style-conventions)
16. [Platform Support & Framework Integration](#16-platform-support--framework-integration)
17. [Future Three.js Parity Plan](#17-future-threejs-parity-plan)

---

## 1. Project Vision

scenix is built around one principle: **any 3D object that can be described can be rendered and animated.**

Everything else — scene graphs, cameras, materials, lights, shadows, post-processing, asset loading, GPU batching — is layered cleanly on top of that foundation. Each layer lives in its own crate and can be used standalone or composed with others.

scenix is the **rendering half** of a two-library ecosystem. `animato` handles *how things move*. scenix handles *what things look like and where they are*. Together they form a complete Three.js-equivalent for Rust.

### Design Goals

| Goal | Decision |
|------|----------|
| Three.js ergonomics, Rust performance | Builder pattern everywhere, zero mandatory runtime overhead |
| `wgpu` as the GPU backend | Runs on Vulkan, Metal, DX12, WebGPU — one codebase |
| Renderer-agnostic scene graph | `scenix-scene` and `scenix-math` have zero GPU dependencies |
| Clean crate boundaries | Each concern lives in its own crate |
| Composable, not monolithic | Use only the crates you need |
| Type-safe node hierarchy | `NodeId` newtypes, no raw pointer graphs |
| First-class `animato` integration | Plug animato tweens directly into scene transforms |
| Desktop + mobile + web parity | Same scene/data APIs target Vulkan, Metal, DX12, WebGPU, and WebGL2 fallback paths |
| `no_std`-ready core | `scenix-math` and `scenix-core` compile without `std` or heap |
| Serializable scenes | Optional `serde` feature on all public data types |
| Discoverable | One facade crate (`scenix`) re-exports everything |

### Non-Goals

- scenix does **NOT** implement a game engine ECS. It manages a scene graph, not an entity system.
- scenix does **NOT** own the window or event loop. It accepts a `wgpu::Surface`; the caller manages the window.
- The stable v1 core does **NOT** implement physics simulation. Collision detection via `scenix-raycaster` is for picking only; future physics belongs behind an optional `scenix-physics` bridge.
- The stable v1 core does **NOT** include audio. Future audio belongs behind an optional `scenix-audio` crate.

### Relationship with Animato

```
animato (computes animation values)
    ↓  via scenix-animato bridge
scenix (applies those values to 3D transforms, materials, cameras)
    ↓  via scenix-renderer
wgpu (draws pixels)
```

Animato is an optional dependency. scenix is fully usable without it.

---

## 2. Why a Workspace — Not a Single Crate

A single `src/` crate for a 3D library becomes unmanageable fast. scenix solves this with a Cargo workspace from day one.

**Benefits:**

- **Compile-time isolation.** A change to `scenix-post` does not recompile `scenix-math`.
- **Clear ownership.** Each crate has one job. A contributor working on PBR materials only needs to understand `scenix-material`.
- **Granular dependencies.** A user who only needs a scene graph adds `scenix-scene`. They never download `wgpu` or `gltf`.
- **Parallel compilation.** Cargo compiles independent crates in parallel.
- **Separate ownership.** Every crate ships at the same stable workspace version, but each crate keeps its own dependency surface and implementation boundary.
- **Optional GPU.** The math and scene layers are pure Rust — GPU crates are opt-in.

---

## 3. Workspace Layout

```
scenix/
├── Cargo.toml                          ← workspace root (no [lib] here)
├── README.md
├── ARCHITECTURE.md                     ← this file
├── ROADMAP.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE-MIT
├── LICENSE-APACHE
│
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                      ← lint, test, no_std, WASM, docs, coverage, package, benches
│   │   ├── pages.yml                   ← Leptos/Trunk GitHub Pages deployment
│   │   └── publish.yml                 ← tag-driven cargo publish and GitHub Release
│   └── ISSUE_TEMPLATE/
│       ├── bug_report.md
│       └── feature_request.md
│
├── crates/
│   ├── scenix-math/                     ← Vec2/3/4, Mat4, Quat, Transform, Ray, AABB (no_std)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── vec.rs                  ← Vec2, Vec3, Vec4
│   │       ├── mat.rs                  ← Mat3, Mat4
│   │       ├── quat.rs                 ← Quaternion, rotation helpers
│   │       ├── euler.rs                ← Euler angles (XYZ/YXZ/ZYX order)
│   │       ├── transform.rs            ← Transform (position + rotation + scale)
│   │       ├── ray.rs                  ← Ray3, parametric intersection
│   │       ├── bounds.rs               ← AABB, Sphere bounds
│   │       ├── plane.rs                ← Plane (normal + distance), half-space tests
│   │       ├── spherical.rs            ← Spherical coordinates (radius, phi, theta)
│   │       └── cylindrical.rs          ← Cylindrical coordinates (radius, theta, y)
│   │
│   ├── scenix-core/                     ← Traits, IDs, errors, Color (no_std)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs               ← Renderable, Bounded, Resizable, Drawable
│   │       ├── ids.rs                  ← NodeId, MeshId, MaterialId, TextureId, LightId
│   │       ├── color.rs                ← Color (RGBA f32), ColorSpace enum
│   │       └── error.rs                ← ScenixError, LoadError, GpuError
│   │
│   ├── scenix-scene/                    ← SceneGraph, SceneNode, transform hierarchy
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs                ← SceneGraph (slot-map backed node storage)
│   │       ├── node.rs                 ← SceneNode, NodeKind enum
│   │       ├── transform.rs            ← local/world transform propagation
│   │       ├── visitor.rs              ← depth-first traversal, BFS iterators
│   │       ├── fog.rs                  ← Fog (linear), FogExp2 (exponential density)
│   │       ├── lod.rs                  ← LodGroup: distance-based geometry switching
│   │       └── sprite.rs               ← Sprite: camera-facing billboard quad
│   │
│   ├── scenix-camera/                   ← Camera types and projection math
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── perspective.rs          ← PerspectiveCamera (fov, aspect, near, far)
│   │       ├── orthographic.rs         ← OrthographicCamera (left/right/top/bottom)
│   │       ├── cube_camera.rs          ← CubeCamera (6-face capture for environment maps)
│   │       ├── frustum.rs              ← Frustum planes, visibility testing
│   │       └── controller.rs           ← OrbitController, FlyController (std feature)
│   │
│   ├── scenix-mesh/                     ← Geometry buffers and primitive generators
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── geometry.rs             ← Geometry: vertices, normals, UVs, indices
│   │       ├── mesh.rs                 ← Mesh = Geometry + MaterialId
│   │       ├── buffer.rs               ← BufferLayout, VertexAttribute, IndexFormat
│   │       ├── morph.rs                ← MorphTarget: blend shapes for facial/deformation anim
│   │       ├── primitives/
│   │       │   ├── mod.rs
│   │       │   ├── box_prim.rs         ← BoxGeometry(w, h, d, segments)
│   │       │   ├── sphere.rs           ← SphereGeometry(radius, widthSeg, heightSeg)
│   │       │   ├── plane.rs            ← PlaneGeometry(w, h, wSeg, hSeg)
│   │       │   ├── cylinder.rs         ← CylinderGeometry(top, bottom, height, seg)
│   │       │   ├── cone.rs             ← ConeGeometry(radius, height, radialSeg)
│   │       │   ├── capsule.rs          ← CapsuleGeometry(radius, height, rings, seg)
│   │       │   ├── torus.rs            ← TorusGeometry(radius, tube, tubeSeg, radSeg)
│   │       │   ├── torus_knot.rs       ← TorusKnotGeometry(radius, tube, p, q)
│   │       │   ├── icosphere.rs        ← IcosphereGeometry(radius, subdivisions)
│   │       │   ├── circle.rs           ← CircleGeometry(radius, segments, arc)
│   │       │   ├── ring.rs             ← RingGeometry(inner, outer, thetaSeg, phiSeg)
│   │       │   ├── lathe.rs            ← LatheGeometry(points, segments, arc)
│   │       │   ├── extrude.rs          ← ExtrudeGeometry(shape, depth, bevel)
│   │       │   ├── tube.rs             ← TubeGeometry(path, tubularSeg, radius)
│   │       │   └── shape_geom.rs       ← ShapeGeometry(shape) — 2D shape → triangulated mesh
│   │       ├── instanced.rs            ← InstancedMesh (transform array + draw indirect)
│   │       └── batched.rs              ← BatchedMesh (multi-geometry single draw call)
│   │
│   ├── scenix-material/                 ← Material trait and built-in material types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs               ← Material trait, ShaderSource, PipelineKey
│   │       ├── pbr.rs                  ← PbrMaterial (albedo, metallic, roughness, ao)
│   │       ├── physical.rs             ← PhysicalMaterial (clearcoat, sheen, transmission, IOR)
│   │       ├── unlit.rs                ← UnlitMaterial (color/texture, no lighting)
│   │       ├── lambert.rs              ← LambertMaterial (diffuse only, faster than PBR)
│   │       ├── toon.rs                 ← ToonMaterial (cel-shading, gradient map)
│   │       ├── normal.rs               ← NormalMaterial (debug: surface normals → RGB)
│   │       ├── wireframe.rs            ← WireframeMaterial
│   │       ├── depth.rs                ← DepthMaterial (for shadow passes)
│   │       ├── line.rs                 ← LineMaterial (width, dash, color)
│   │       ├── points.rs               ← PointsMaterial (point size, attenuation)
│   │       └── shader.rs               ← ShaderMaterial (custom WGSL, uniform slots)
│   │
│   ├── scenix-light/                    ← Light types and shadow map management
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ambient.rs              ← AmbientLight (color, intensity)
│   │       ├── directional.rs          ← DirectionalLight (dir, color, intensity, shadow)
│   │       ├── point.rs                ← PointLight (position, color, intensity, decay)
│   │       ├── spot.rs                 ← SpotLight (position, target, angle, penumbra)
│   │       ├── hemisphere.rs           ← HemisphereLight (sky color, ground color)
│   │       ├── area.rs                 ← AreaLight (rect emitter, LTC approximation)
│   │       ├── probe.rs                ← LightProbe (SH-based environment lighting from raw samples in v0.4)
│   │       └── shadow.rs               ← ShadowMap, ShadowConfig (PCF, bias, cascades)
│   │
│   ├── scenix-texture/                  ← Texture loading, sampling, atlases
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── texture.rs              ← Texture2D, TextureCube, Texture3D
│   │       ├── sampler.rs              ← Sampler (filter, wrap, anisotropy)
│   │       ├── atlas.rs                ← TextureAtlas (sprite sheet, UV rect packing)
│   │       ├── video.rs                ← VideoTexture (frame-by-frame update from video source)
│   │       ├── mipmap.rs               ← CPU mipmap generation
│   │       └── format.rs               ← TextureFormat enum, compression (BC, ASTC, ETC2)
│   │
│   ├── scenix-renderer/                 ← wgpu render pipeline and frame loop
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs             ← Renderer: owns Device, Queue, Surface
│   │       ├── pipeline.rs             ← RenderPipeline cache keyed by PipelineKey
│   │       ├── pass/
│   │       │   ├── mod.rs
│   │       │   ├── shadow_pass.rs      ← depth-only pass for shadow maps
│   │       │   ├── geometry_pass.rs    ← G-buffer pass (deferred path)
│   │       │   ├── lighting_pass.rs    ← deferred lighting resolve
│   │       │   └── forward_pass.rs     ← forward+ pass (default for transparent)
│   │       ├── gpu_scene.rs            ← uploads SceneGraph data to GPU buffers
│   │       ├── culling.rs              ← frustum + occlusion culling
│   │       ├── sort.rs                 ← depth sort for transparent objects
│   │       └── frame.rs                ← FrameContext, per-frame uniform buffers
│   │
│   ├── scenix-loader/                   ← Asset loaders for 3D formats and images
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── gltf.rs                 ← GLTF 2.0 loader (meshes, materials, skins, anims)
│   │       ├── obj.rs                  ← Wavefront OBJ + MTL loader
│   │       ├── stl.rs                  ← STL loader (3D printing format)
│   │       ├── fbx.rs                  ← FBX loader (Autodesk interchange)
│   │       ├── draco.rs                ← Draco mesh decompression (Google)
│   │       ├── image.rs                ← PNG/JPEG/WebP/KTX2 → Texture2D
│   │       ├── hdr.rs                  ← HDR/EXR → TextureCube for IBL
│   │       └── cache.rs                ← AssetCache (dedup, async loading, hot-reload)
│   │
│   ├── scenix-post/                     ← Post-processing effect pipeline
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── stack.rs                ← PostStack: ordered chain of effects
│   │       ├── bloom.rs                ← Bloom (threshold, intensity, blur passes)
│   │       ├── ssao.rs                 ← SSAO (screen-space ambient occlusion)
│   │       ├── tonemap.rs              ← ToneMapper (ACES, Reinhard, Filmic, AgX)
│   │       ├── fxaa.rs                 ← FXAA (fast approximate anti-aliasing)
│   │       ├── taa.rs                  ← TAA (temporal anti-aliasing, jitter matrix)
│   │       ├── smaa.rs                 ← SMAA (enhanced subpixel morphological AA)
│   │       ├── dof.rs                  ← Depth of Field (bokeh, aperture, focus dist)
│   │       ├── fog.rs                  ← Volumetric Fog (exponential, height-based)
│   │       ├── outline.rs              ← Outline effect (selected object highlighting)
│   │       └── motion_blur.rs          ← Per-object motion blur (velocity buffer)
│   │
│   ├── scenix-raycaster/                ← Ray–scene intersection and BVH acceleration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── raycaster.rs            ← Raycaster: casts rays into SceneGraph
│   │       ├── intersection.rs         ← Intersection result (node, distance, UV, normal)
│   │       ├── bvh.rs                  ← BVH (bounding volume hierarchy, SAH build)
│   │       └── tests.rs                ← ray-AABB, ray-triangle, ray-sphere tests
│   │
│   ├── scenix-animato/                  ← Bridge: animato animations → scenix transforms
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── values.rs               ← AnimVec3, AnimQuat, AnimColor wrappers
│   │       ├── tracks.rs               ← Scalar/Vec3/Quat/Color/Bool tracks
│   │       ├── scene.rs                ← NodeAnimator: binds tracks to NodeId
│   │       ├── camera.rs               ← CameraAnimator and CameraStoreMut
│   │       ├── material.rs             ← MaterialAnimator for PBR fields
│   │       ├── skeleton.rs             ← SkinnedMeshAnimator: drives bone transforms
│   │       └── driver.rs               ← ScenixAnimationDriver: ticks all bound animators
│   │
│   ├── scenix-wasm/                     ← WebGPU / WebGL2 browser integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── input.rs                ← DOM key/button mapping helpers
│   │       └── web.rs                  ← BrowserRenderer, WebRenderer, WebGlRenderer, generated scene
│   │
│   ├── scenix-helpers/                  ← Debug visualization helpers
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── grid.rs                 ← GridHelper (configurable grid plane)
│   │       ├── axes.rs                 ← AxesHelper (RGB XYZ axis lines)
│   │       ├── bounding_box.rs         ← BoundingBoxHelper (wireframe AABB)
│   │       ├── arrow.rs                ← ArrowHelper (directional arrow mesh)
│   │       ├── light_helper.rs         ← SpotLightHelper, PointLightHelper, DirLightHelper
│   │       ├── camera_helper.rs        ← CameraHelper (frustum wireframe)
│   │       └── skeleton_helper.rs      ← SkeletonHelper (bone visualization)
│   │
│   ├── scenix-input/                    ← Shared input state types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pointer.rs              ← PointerState, PointerButton, PointerEvent
│   │       ├── keyboard.rs             ← KeyboardState, KeyCode, Modifiers
│   │       ├── touch.rs                ← TouchState, TouchPoint, pinch/rotate gesture
│   │       └── gamepad.rs              ← GamepadState, GamepadButton, axes
│   │
│   └── scenix/                          ← facade crate — the one users add to Cargo.toml
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs                  ← pub use everything from every sub-crate
│
├── examples/
│   ├── hello_cube.rs                   ← rotating box, unlit material
│   ├── pbr_sphere.rs                   ← PBR sphere with image-based lighting
│   ├── physical_material.rs             ← clearcoat car paint + glass transmission
│   ├── toon_shading.rs                 ← ToonMaterial with custom gradient map
│   ├── gltf_scene.rs                   ← load and display a GLTF file
│   ├── shadow_demo.rs                  ← directional light + PCF shadow map
│   ├── raycasting.rs                   ← mouse picking with BVH
│   ├── post_processing.rs              ← full PostStack: SSAO + Bloom + ToneMap + TAA
│   ├── instanced_mesh.rs               ← 10,000 instanced cubes
│   ├── animato_integration.rs          ← spring-driven camera + tween material color
│   ├── orbit_camera.rs                 ← OrbitController with mouse input
│   ├── lod_demo.rs                     ← LodGroup with distance-based geometry swap
│   ├── morph_targets.rs                ← facial blend shapes from GLTF
│   ├── fog_demo.rs                     ← scene fog + volumetric post-process fog
│   ├── helpers_demo.rs                 ← GridHelper + AxesHelper + LightHelpers
│   ├── sprite_particles.rs             ← billboard particle system with Sprites
│   ├── environment_map.rs              ← CubeCamera IBL capture + reflections
│   └── wasm_viewer/                    ← generated-scene browser viewer
│       ├── src/lib.rs
│       └── www/index.html
│
├── website/                            ← Leptos CSR site deployed at /scenix/
│   ├── Cargo.toml                      ← standalone workspace to isolate website deps
│   ├── Trunk.toml
│   ├── index.html
│   ├── public/
│   └── src/
│
├── docs/                               ← stable v1 user docs and release notes
│
├── benches/
│   ├── scene_graph_bench.rs            ← 10K node graph traversal + transform propagation
│   ├── render_bench.rs                 ← frame time with 1K / 10K / 100K triangles
│   ├── bvh_bench.rs                    ← BVH build + 1K ray queries
│   ├── mesh_gen_bench.rs               ← primitive generation throughput
│   └── culling_bench.rs                ← frustum culling 10K objects
│
└── tests/
    ├── scene_hierarchy.rs              ← parent/child, world transform correctness
    ├── camera_frustum.rs               ← frustum plane extraction, visibility test
    ├── mesh_primitives.rs              ← vertex count, normal validity, UV range
    ├── material_pipeline.rs            ← pipeline cache hit/miss correctness
    ├── loader_gltf.rs                  ← round-trip load of reference GLTF assets
    └── raycaster_correctness.rs        ← ray-triangle intersection precision
```

### Future Workspace Expansion

The layout above describes the shipped v1.1 workspace. Future systems such as audio, physics, XR, editor tooling, particles, terrain, sky, water, and shader nodes should be added as **new optional crates** only when implementation begins. They should not be listed as active workspace members in `Cargo.toml` until the crate directory, tests, docs, examples, and feature flags exist.

Planned future crates may extend the workspace like this:

```text
crates/
├── scenix-nodes/       ← optional shader graph and node material system
├── scenix-particles/   ← optional CPU/GPU particle systems
├── scenix-terrain/     ← optional heightmap terrain and chunked LOD
├── scenix-sky/         ← optional procedural sky and atmosphere
├── scenix-water/       ← optional water, reflection, and refraction helpers
├── scenix-xr/          ← optional WebXR/OpenXR integration
├── scenix-audio/       ← optional spatial audio bridge
├── scenix-physics/     ← optional Rapier/Jolt physics bridge
├── scenix-editor/      ← optional visual editor shell and tools
└── scenix-ui/          ← optional cross-platform debug UI overlays
```

Rules for adding a future crate:

- keep the crate optional behind a focused facade feature;
- keep heavy third-party dependencies out of default features;
- document desktop, mobile, WebGPU, and WebGL support level;
- add at least one example, one API doc page, and focused tests before listing it as shipped;
- avoid moving stable v1 APIs into the new crate unless a deprecation path is documented.

## v1.0.0 Stable Contract

The stable API contract keeps scenix modular:

- default facade features are CPU authoring plus BVH raycasting and helper geometry;
- loader, renderer, post-processing, Animato, and WASM paths remain optional;
- scene data stays renderer-agnostic and GPU resources stay renderer-owned;
- public API changes should be additive, with deprecations before removals;
- desktop and mobile applications are first-class runtime targets through `wgpu` surfaces;
- the website is a static Leptos CSR app built by Trunk and deployed to GitHub Pages at `/scenix/`, but it is only one deployment target.

### Root `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/scenix-math",
    "crates/scenix-core",
    "crates/scenix-scene",
    "crates/scenix-camera",
    "crates/scenix-mesh",
    "crates/scenix-material",
    "crates/scenix-light",
    "crates/scenix-texture",
    "crates/scenix-renderer",
    "crates/scenix-loader",
    "crates/scenix-post",
    "crates/scenix-raycaster",
    "crates/scenix-animato",
    "crates/scenix-wasm",
    "crates/scenix-helpers",
    "crates/scenix-input",
    "crates/scenix",
]

[workspace.package]
version      = "1.1.0"
edition      = "2024"
license      = "MIT OR Apache-2.0"
repository   = "https://github.com/AarambhDevHub/scenix"
authors      = ["Aarambh Dev Hub"]
rust-version = "1.89"

[workspace.dependencies]
# internal crates — version pinned to workspace
scenix-math       = { path = "crates/scenix-math",       version = "1" }
scenix-core       = { path = "crates/scenix-core",       version = "1" }
scenix-scene      = { path = "crates/scenix-scene",      version = "1" }
scenix-camera     = { path = "crates/scenix-camera",     version = "1" }
scenix-mesh       = { path = "crates/scenix-mesh",       version = "1" }
scenix-material   = { path = "crates/scenix-material",   version = "1" }
scenix-light      = { path = "crates/scenix-light",      version = "1" }
scenix-texture    = { path = "crates/scenix-texture",    version = "1" }
scenix-loader     = { path = "crates/scenix-loader",     version = "1" }
scenix-post       = { path = "crates/scenix-post",       version = "1" }
scenix-renderer   = { path = "crates/scenix-renderer",   version = "1" }
scenix-raycaster  = { path = "crates/scenix-raycaster",  version = "1" }
scenix-animato    = { path = "crates/scenix-animato",    version = "1" }
scenix-wasm       = { path = "crates/scenix-wasm",       version = "1" }
scenix-helpers    = { path = "crates/scenix-helpers",    version = "1" }
scenix-input      = { path = "crates/scenix-input",      version = "1" }

# external crates — shared version pins
wgpu             = { version = "29.0.3" }
bytemuck         = { version = "1",   features = ["derive"] }
serde            = { version = "1",   features = ["derive"] }
image            = { version = "0.25.10", default-features = false }
gltf             = { version = "1.4.1",   default-features = false }
ktx2             = { version = "0.4.0" }
tobj             = { version = "4.0.3", default-features = false }
stl_io           = { version = "0.11.0" }
reqwest          = { version = "0.12", default-features = false }
slotmap          = { version = "1" }
ahash            = { version = "0.8" }
log              = { version = "0.4" }
winit            = { version = "0.30.13" }
raw-window-handle = { version = "0.6" }
pollster         = { version = "0.4" }
wasm-bindgen     = { version = "0.2" }
js-sys           = { version = "0.3" }
web-sys          = { version = "0.3", features = ["HtmlCanvasElement", "Window"] }
animato          = { version = "1.4.0", default-features = false }
criterion        = { version = "0.5", features = ["html_reports"] }
approx           = { version = "0.5" }
thiserror        = { version = "2" }
```

---

## 4. Crate-by-Crate Specification

---

### 4.1 `scenix-math`

**Responsibility:** All 3D math primitives. This is the foundation every other crate builds on. Must compile in `no_std` environments with zero external dependencies.

**Dependency rule:** This crate depends on NOTHING except `libcore` and optionally `libm` for `no_std` trigonometry.

#### `src/vec.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 { pub x: f32, pub y: f32 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4 { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

impl Vec3 {
    pub const ZERO: Self;
    pub const ONE:  Self;
    pub const X:    Self;    // (1, 0, 0)
    pub const Y:    Self;    // (0, 1, 0)
    pub const Z:    Self;    // (0, 0, 1)
    pub const UP:   Self;    // (0, 1, 0) — world up

    pub fn dot(self, rhs: Self) -> f32;
    pub fn cross(self, rhs: Self) -> Self;
    pub fn length(self) -> f32;
    pub fn length_squared(self) -> f32;
    pub fn normalize(self) -> Self;
    pub fn lerp(self, rhs: Self, t: f32) -> Self;
    pub fn distance(self, rhs: Self) -> f32;
    pub fn reflect(self, normal: Self) -> Self;
    pub fn angle_between(self, rhs: Self) -> f32;      // radians
}
```

#### `src/mat.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 { cols: [Vec4; 4] }    // column-major, matches wgpu/WGSL convention

impl Mat4 {
    pub const IDENTITY: Self;

    pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self;
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self;
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self;
    pub fn from_translation(v: Vec3) -> Self;
    pub fn from_rotation(q: Quat) -> Self;
    pub fn from_scale(v: Vec3) -> Self;
    pub fn from_trs(t: Vec3, r: Quat, s: Vec3) -> Self;    // compose TRS in one call

    pub fn mul_mat4(self, rhs: Self) -> Self;
    pub fn mul_vec4(self, rhs: Vec4) -> Vec4;
    pub fn mul_vec3(self, rhs: Vec3) -> Vec3;               // applies homogeneous divide
    pub fn inverse(self) -> Option<Self>;
    pub fn transpose(self) -> Self;
    pub fn to_cols_array(self) -> [f32; 16];                // for wgpu buffer upload
}
```

#### `src/quat.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

impl Quat {
    pub const IDENTITY: Self;

    pub fn from_axis_angle(axis: Vec3, angle_rad: f32) -> Self;
    pub fn from_euler_xyz(x: f32, y: f32, z: f32) -> Self;    // angles in radians
    pub fn from_rotation_arc(from: Vec3, to: Vec3) -> Self;    // minimal rotation between two directions

    pub fn mul_quat(self, rhs: Self) -> Self;
    pub fn mul_vec3(self, rhs: Vec3) -> Vec3;
    pub fn conjugate(self) -> Self;
    pub fn inverse(self) -> Self;
    pub fn normalize(self) -> Self;
    pub fn slerp(self, rhs: Self, t: f32) -> Self;             // spherical linear interpolation
    pub fn to_mat4(self) -> Mat4;
    pub fn to_euler_xyz(self) -> Vec3;                          // extract Euler angles
    pub fn angle_between(self, rhs: Self) -> f32;
}
```

#### `src/transform.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation:    Quat,
    pub scale:       Vec3,
}

impl Transform {
    pub const IDENTITY: Self;

    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self;
    pub fn from_translation(v: Vec3) -> Self;
    pub fn from_rotation(q: Quat) -> Self;
    pub fn looking_at(eye: Vec3, target: Vec3, up: Vec3) -> Self;

    pub fn to_mat4(self) -> Mat4;
    pub fn mul_transform(self, rhs: Self) -> Self;    // compose two transforms
    pub fn inverse(self) -> Self;
    pub fn forward(self) -> Vec3;     // -Z in local space, transformed to world
    pub fn right(self) -> Vec3;       //  X in local space
    pub fn up(self) -> Vec3;          //  Y in local space

    pub fn translate_by(self, delta: Vec3) -> Self;
    pub fn rotate_by(self, q: Quat) -> Self;
    pub fn scale_by(self, s: Vec3) -> Self;
}
```

#### `src/ray.rs` and `src/bounds.rs`

```rust
pub struct Ray3 {
    pub origin:    Vec3,
    pub direction: Vec3,    // always normalized
}

impl Ray3 {
    pub fn at(&self, t: f32) -> Vec3;
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<f32>;
    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32>;
    pub fn intersect_triangle(&self, a: Vec3, b: Vec3, c: Vec3) -> Option<(f32, Vec2)>;
    // returns (t, barycentric UV) or None
}

pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn from_points(points: &[Vec3]) -> Self;
    pub fn center(&self) -> Vec3;
    pub fn half_extents(&self) -> Vec3;
    pub fn contains_point(&self, p: Vec3) -> bool;
    pub fn intersects_aabb(&self, other: &Self) -> bool;
    pub fn transform(&self, mat: Mat4) -> Self;    // conservative transform
    pub fn merge(&self, other: &Self) -> Self;
    pub fn surface_area(&self) -> f32;             // used by SAH BVH builder
}
```

#### `src/euler.rs`

```rust
/// Rotation order for Euler angle decomposition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RotationOrder { XYZ, YXZ, ZXY, ZYX, YZX, XZY }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Euler {
    pub x: f32,    // pitch, radians
    pub y: f32,    // yaw, radians
    pub z: f32,    // roll, radians
    pub order: RotationOrder,
}

impl Euler {
    pub fn new(x: f32, y: f32, z: f32, order: RotationOrder) -> Self;
    pub fn from_quat(q: Quat, order: RotationOrder) -> Self;
    pub fn from_mat4(m: Mat4, order: RotationOrder) -> Self;
    pub fn to_quat(self) -> Quat;
}
```

#### `src/plane.rs`

```rust
/// A plane defined by a unit normal and signed distance from origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub normal:   Vec3,    // unit-length
    pub distance: f32,     // signed distance from origin
}

impl Plane {
    pub fn from_normal_and_point(normal: Vec3, point: Vec3) -> Self;
    pub fn from_three_points(a: Vec3, b: Vec3, c: Vec3) -> Self;
    pub fn signed_distance(&self, p: Vec3) -> f32;
    pub fn project_point(&self, p: Vec3) -> Vec3;
    pub fn intersect_ray(&self, ray: &Ray3) -> Option<f32>;
    pub fn intersect_line(&self, a: Vec3, b: Vec3) -> Option<Vec3>;
}
```

#### `src/spherical.rs` and `src/cylindrical.rs`

```rust
/// Spherical coordinates — used by OrbitController internally.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spherical {
    pub radius: f32,
    pub phi:    f32,    // polar angle from Y axis (0..π)
    pub theta:  f32,    // azimuthal angle in XZ plane (0..2π)
}

impl Spherical {
    pub fn from_vec3(v: Vec3) -> Self;
    pub fn to_vec3(self) -> Vec3;
    pub fn clamp_phi(self, min: f32, max: f32) -> Self;
}

/// Cylindrical coordinates — useful for radial placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cylindrical {
    pub radius: f32,
    pub theta:  f32,    // angle in XZ plane
    pub y:      f32,    // height
}

impl Cylindrical {
    pub fn from_vec3(v: Vec3) -> Self;
    pub fn to_vec3(self) -> Vec3;
}
```

**`Cargo.toml`:**

```toml
[package]
name        = "scenix-math"
description = "3D math primitives for the scenix rendering library."

[features]
default = ["std"]
std     = []
libm    = ["dep:libm"]    # enables no_std trig via libm
serde   = ["dep:serde"]
approx  = ["dep:approx"]  # approx::AbsDiffEq impls for tests

[dependencies]
libm  = { version = "0.2", optional = true }
serde = { workspace = true, optional = true }
approx = { version = "0.5", optional = true }
```

---

### 4.2 `scenix-core`

**Responsibility:** Shared traits, ID newtypes, color type, and error types. Every other crate imports from here but this crate imports from nothing except `scenix-math`.

**Depends on:** `scenix-math`

#### `src/ids.rs`

```rust
// All IDs are Copy newtypes over u64 — zero-cost, hash-friendly.
// Generated by SlotMap in scenix-scene / scenix-renderer; never constructed by users directly.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MaterialId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LightId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CameraId(u64);
```

#### `src/traits.rs`

```rust
pub trait Bounded {
    fn aabb(&self) -> Aabb;
    fn bounding_sphere(&self) -> (Vec3, f32);    // center, radius
}

// Only available with the "gpu" feature (bytemuck is no_std but optional)
#[cfg(feature = "gpu")]
pub trait GpuUpload {
    type GpuData: bytemuck::Pod;
    fn to_gpu(&self) -> Self::GpuData;
}

pub trait Named {
    fn name(&self) -> &str;
    fn set_name(&mut self, name: impl Into<String>);
}
```

#### `src/color.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,    // 0.0..=1.0
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self;
    pub const BLACK: Self;
    pub const TRANSPARENT: Self;
    pub const RED: Self;
    pub const GREEN: Self;
    pub const BLUE: Self;

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self;
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self;
    pub fn from_hex(hex: u32) -> Self;              // e.g. 0xFF8800FF
    pub fn from_srgb_u8(r: u8, g: u8, b: u8) -> Self;

    pub fn to_linear(self) -> Self;                 // sRGB → linear (for PBR)
    pub fn to_srgb(self) -> Self;
    pub fn lerp(self, rhs: Self, t: f32) -> Self;
    pub fn to_array(self) -> [f32; 4];
}
```

---

### 4.3 `scenix-scene`

**Responsibility:** The scene graph. Owns the hierarchy of nodes, their transforms, and their attached resources (mesh, light, camera). Zero GPU dependency.

**Depends on:** `scenix-math`, `scenix-core`

#### `src/graph.rs`

```rust
pub struct SceneGraph {
    nodes:       SlotMap<PrivateSceneKey, NodeRecord>,
    roots:       Vec<NodeId>,                  // top-level nodes (no parent)
    id_to_key:   Vec<Option<PrivateSceneKey>>, // graph-local public handles
    next_id:     u64,                          // never reused within a graph
    dirty_roots: Vec<NodeId>,                  // dirty subtree entry points
    fog:         Option<Fog>,
}

impl SceneGraph {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;

    // Node management
    pub fn add(&mut self, node: SceneNode) -> NodeId;
    pub fn add_child(&mut self, parent: NodeId, node: SceneNode) -> Result<NodeId, ValidationError>;
    pub fn remove(&mut self, id: NodeId) -> Result<(), ValidationError>;
    pub fn get(&self, id: NodeId) -> Option<&SceneNode>;
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut SceneNode>;

    // Transform queries — updated by explicit dirty subtree propagation
    pub fn update_world_transforms(&mut self);
    pub fn world_transform(&self, id: NodeId) -> Option<Transform>;
    pub fn world_matrix(&self, id: NodeId) -> Option<Mat4>;
    pub fn set_local_transform(&mut self, id: NodeId, t: Transform) -> Result<(), ValidationError>;

    // Hierarchy
    pub fn parent(&self, id: NodeId) -> Option<NodeId>;
    pub fn children(&self, id: NodeId) -> Option<&[NodeId]>;
    pub fn roots(&self) -> &[NodeId];
    pub fn reparent(&mut self, node: NodeId, new_parent: Option<NodeId>) -> Result<(), ValidationError>;

    // Traversal
    pub fn iter_depth_first(&self) -> DepthFirstIter<'_>;
    pub fn iter_breadth_first(&self) -> BreadthFirstIter<'_>;

    // Querying
    pub fn find_by_name(&self, name: &str) -> Option<NodeId>;
}
```

`NodeId` remains the public `u64` handle from `scenix-core`. `scenix-scene`
uses private SlotMap keys internally and keeps a graph-local handle table, so
SlotMap key layout never becomes public API. Mutating hierarchy operations return
`ValidationError::InvalidId` for missing IDs and `ValidationError::InvalidState`
for cycle-creating reparents.

#### `src/node.rs`

```rust
pub struct SceneNode {
    pub name:      String,
    pub transform: Transform,              // local transform
    pub visible:   bool,
    pub layer:     u32,                    // bitmask for camera culling layers
    pub kind:      NodeKind,
}

pub enum NodeKind {
    Empty,
    Group,    // logical grouping, no render data
    Mesh   { mesh_id: MeshId, material_id: MaterialId },
    Light  { light_id: LightId },
    Camera { camera_id: CameraId },
    Sprite(Sprite),
}

// Builder pattern for ergonomic construction:
let node = SceneNode::new("Sword")
    .transform(Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)))
    .kind(NodeKind::Mesh { mesh_id, material_id })
    .visible(true)
    .layer(0b0001);
```

---

### 4.4 `scenix-camera`

**Responsibility:** Camera types, projection matrices, frustum culling, and optional interactive controllers.

**Depends on:** `scenix-math`, `scenix-core`

#### `src/perspective.rs`

```rust
pub struct PerspectiveCamera {
    pub fov_y:  f32,     // vertical field of view, radians
    pub aspect: f32,     // width / height
    pub near:   f32,
    pub far:    f32,
    pub position: Vec3,
    pub target:   Vec3,
    pub up:       Vec3,
}

impl PerspectiveCamera {
    pub fn new(fov_y_deg: f32, aspect: f32, near: f32, far: f32) -> Self;
    pub fn projection_matrix(&self) -> Mat4;
    pub fn view_matrix(&self) -> Mat4;
    pub fn view_projection(&self) -> Mat4;
    pub fn frustum(&self) -> Frustum;
    pub fn screen_to_ray(&self, ndc: Vec2) -> Ray3;    // for raycasting from mouse position
}
```

#### `src/frustum.rs`

```rust
pub struct Frustum {
    planes: [Vec4; 6],    // [left, right, bottom, top, near, far] — normal + offset
}

impl Frustum {
    pub fn from_view_projection(vp: Mat4) -> Self;    // Gribb/Hartmann extraction

    pub fn contains_point(&self, p: Vec3) -> bool;
    pub fn contains_aabb(&self, aabb: &Aabb) -> Visibility;
    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> Visibility;
}

pub enum Visibility { Outside, Intersects, Inside }
```

#### `src/controller.rs` (std feature)

```rust
pub struct OrbitController {
    pub target:       Vec3,
    pub radius:       f32,
    pub theta:        f32,    // azimuth angle
    pub phi:          f32,    // polar angle
    pub min_radius:   f32,
    pub max_radius:   f32,
    pub damping:      f32,    // 0.0 = instant, 1.0 = frozen
}

impl OrbitController {
    pub fn on_drag(&mut self, delta: Vec2, dt: f32);
    pub fn on_scroll(&mut self, delta: f32, dt: f32);
    pub fn on_pan(&mut self, delta: Vec2, dt: f32);
    pub fn update(&mut self, dt: f32);
    pub fn camera_transform(&self) -> Transform;
}
```

---

### 4.5 `scenix-mesh`

**Responsibility:** CPU-side geometry buffers and primitive generators. This crate knows nothing about the GPU.

**Depends on:** `scenix-math`, `scenix-core`

#### `src/geometry.rs`

```rust
pub struct Geometry {
    pub positions:  Vec<Vec3>,       // always required
    pub normals:    Vec<Vec3>,       // optional — auto-generated if absent
    pub tangents:   Vec<Vec4>,       // optional — for normal mapping
    pub uvs:        Vec<Vec2>,       // UV channel 0
    pub uvs2:       Vec<Vec2>,       // UV channel 1 (lightmaps)
    pub colors:     Vec<Color>,      // per-vertex color
    pub indices:    Option<Vec<u32>>,
    pub topology:   PrimitiveTopology,
}

impl Geometry {
    pub fn compute_normals(&mut self);       // flat or smooth based on indexed/non-indexed
    pub fn compute_tangents(&mut self);      // MikkTSpace algorithm
    pub fn center(&self) -> Vec3;
    pub fn aabb(&self) -> Aabb;
    pub fn merge(&self, other: &Self) -> Self;
    pub fn vertex_count(&self) -> usize;
    pub fn triangle_count(&self) -> usize;
}
```

#### Primitive generators

```rust
// All constructors return a fully valid Geometry with positions, normals, and UVs.

pub fn box_geometry(width: f32, height: f32, depth: f32,
                    width_segs: u32, height_segs: u32, depth_segs: u32) -> Geometry;

pub fn sphere_geometry(radius: f32, width_segs: u32, height_segs: u32) -> Geometry;

pub fn plane_geometry(width: f32, height: f32, width_segs: u32, height_segs: u32) -> Geometry;

pub fn cylinder_geometry(top_radius: f32, bottom_radius: f32, height: f32,
                         radial_segs: u32, height_segs: u32, open_ended: bool) -> Geometry;

pub fn torus_geometry(radius: f32, tube: f32,
                      radial_segs: u32, tubular_segs: u32) -> Geometry;

pub fn icosphere_geometry(radius: f32, subdivisions: u32) -> Geometry;

pub fn capsule_geometry(radius: f32, height: f32, rings: u32, segments: u32) -> Geometry;
```

#### `src/instanced.rs`

```rust
pub struct InstancedMesh {
    pub mesh_id:     MeshId,
    pub material_id: MaterialId,
    pub transforms:  Vec<Mat4>,    // one per instance — uploaded to GPU as storage buffer
    pub count:       u32,
}

impl InstancedMesh {
    pub fn new(mesh_id: MeshId, material_id: MaterialId, capacity: u32) -> Self;
    pub fn set_transform_at(&mut self, index: u32, t: Transform);
    pub fn push(&mut self, t: Transform);
    pub fn clear(&mut self);
}
```

---

### 4.6 `scenix-material`

**Responsibility:** Material trait and all built-in material types. Defines the `PipelineKey` used by the renderer to cache compiled pipelines.

**Depends on:** `scenix-math`, `scenix-core`

> **Design decision:** The `Material` trait has NO wgpu dependency.
> GPU-specific methods (`bind_group_layout`, `to_uniform_bytes`) live in
> `GpuMaterial` — a trait extension defined in `scenix-renderer`. This keeps
> `scenix-material` GPU-free and testable without a graphics context.

#### `src/traits.rs`

```rust
/// CPU-side material description — zero GPU dependencies.
pub trait Material: Send + Sync + 'static {
    fn pipeline_key(&self) -> PipelineKey;     // determines which WGSL pipeline to use
    fn is_transparent(&self) -> bool;          // affects render order and blending
    fn double_sided(&self) -> bool;
    fn alpha_cutoff(&self) -> Option<f32>;     // for AlphaMode::Mask
}
```

#### `src/pbr.rs`

```rust
pub struct PbrMaterial {
    pub name:                 String,
    pub albedo:               Color,           // base color (linear)
    pub albedo_texture:       Option<TextureId>,
    pub metallic:             f32,             // 0.0 = dielectric, 1.0 = metal
    pub roughness:            f32,             // 0.0 = mirror, 1.0 = matte
    pub metallic_roughness_texture: Option<TextureId>,
    pub normal_texture:       Option<TextureId>,
    pub occlusion_texture:    Option<TextureId>,
    pub emissive:             Vec3,            // emissive color (linear)
    pub emissive_texture:     Option<TextureId>,
    pub alpha_mode:           AlphaMode,       // Opaque / Mask(f32) / Blend
    pub double_sided:         bool,
}

pub enum AlphaMode {
    Opaque,
    Mask(f32),     // cutoff threshold
    Blend,
}
```

#### `src/shader.rs`

```rust
pub struct ShaderMaterial {
    pub name:         String,
    pub vertex_wgsl:  String,       // custom vertex shader source
    pub fragment_wgsl: String,      // custom fragment shader source
    pub uniforms:     Vec<u8>,      // raw uniform buffer bytes
    pub textures:     Vec<TextureId>,
    pub transparent:  bool,
    pub double_sided: bool,
}
```

#### `src/physical.rs`

```rust
/// Physically-based material with advanced surface effects.
/// Equivalent to Three.js MeshPhysicalMaterial.
pub struct PhysicalMaterial {
    // Inherits all PbrMaterial fields, plus:
    pub base:            PbrMaterial,
    pub clearcoat:       f32,            // 0.0..=1.0, clear lacquer layer strength
    pub clearcoat_roughness: f32,        // roughness of the clearcoat layer
    pub clearcoat_normal_texture: Option<TextureId>,
    pub sheen:           f32,            // 0.0..=1.0, fabric-like sheen
    pub sheen_color:     Color,
    pub sheen_roughness: f32,
    pub transmission:    f32,            // 0.0..=1.0, glass-like transparency
    pub thickness:       f32,            // volume thickness for transmission
    pub ior:             f32,            // index of refraction (default: 1.5)
    pub iridescence:     f32,            // thin-film interference (soap bubble)
    pub iridescence_ior: f32,
}
```

#### `src/toon.rs`

```rust
/// Cel-shading material with discrete shading bands.
pub struct ToonMaterial {
    pub name:           String,
    pub color:          Color,
    pub color_texture:  Option<TextureId>,
    pub gradient_map:   Option<TextureId>,  // 1D ramp texture for shading steps
    pub steps:          u32,                // fallback: number of discrete bands
    pub outline_width:  f32,                // 0.0 = no outline
    pub outline_color:  Color,
}
```

#### `src/normal.rs`

```rust
/// Debug material: renders surface normals as RGB colors.
pub struct NormalMaterial {
    pub opacity:     f32,
    pub flat_shading: bool,
    pub world_space:  bool,    // false = view-space normals, true = world-space
}
```

---

### 4.7 `scenix-light`

**Responsibility:** All light types, shadow map configuration, and the light uniform structs uploaded to the GPU.

**Depends on:** `scenix-math`, `scenix-core`

```rust
pub struct AmbientLight {
    pub color:     Color,
    pub intensity: f32,
}

pub struct DirectionalLight {
    pub direction: Vec3,
    pub color:     Color,
    pub intensity: f32,
    pub shadow:    Option<ShadowConfig>,
}

pub struct PointLight {
    pub color:     Color,
    pub intensity: f32,
    pub range:     f32,     // max distance beyond which intensity is zero
    pub decay:     f32,     // physically: 2.0 = inverse square
    pub shadow:    Option<ShadowConfig>,
}

pub struct SpotLight {
    pub color:        Color,
    pub intensity:    f32,
    pub range:        f32,
    pub angle:        f32,     // outer cone half-angle, radians
    pub penumbra:     f32,     // 0.0..=1.0 fraction of angle that softens
    pub shadow:       Option<ShadowConfig>,
}

pub struct ShadowConfig {
    pub map_size:   u32,      // texel resolution (512, 1024, 2048, 4096)
    pub near:       f32,
    pub far:        f32,
    pub bias:       f32,      // prevents shadow acne
    pub pcf_radius: u32,      // PCF kernel radius in texels (0 = hard shadows)
    pub cascades:   u8,       // for directional: number of CSM cascades (1..=4)
}
```

#### `src/hemisphere.rs`

```rust
/// Sky/ground gradient light — simulates outdoor ambient lighting.
pub struct HemisphereLight {
    pub sky_color:    Color,
    pub ground_color: Color,
    pub intensity:    f32,
}
```

#### `src/probe.rs`

```rust
/// Spherical harmonics environment light — baked IBL for static scenes.
pub struct LightProbe {
    pub sh_coefficients: [Vec3; 9],    // 3rd-order SH (9 coefficients × RGB)
    pub intensity:       f32,
}

impl LightProbe {
    pub fn from_coefficients(sh_coefficients: [Vec3; 9], intensity: f32) -> Self;
    pub fn from_cube_faces(faces: [&[Vec3]; 6], face_size: u32, intensity: f32) -> Result<Self, ValidationError>;
    pub fn from_equirectangular_samples(samples: &[Vec3], width: u32, height: u32, intensity: f32) -> Result<Self, ValidationError>;
}
```

Texture-backed probe constructors remain a renderer/loader integration concern.
The current API projects linear RGB raw samples directly; cube face order is
`+X, -X, +Y, -Y, +Z, -Z`.

---

### 4.8 `scenix-texture`

**Responsibility:** CPU-side texture data and sampler configuration. GPU upload happens in `scenix-renderer`.

**Depends on:** `scenix-core`

```rust
pub struct Texture2D {
    pub width:   u32,
    pub height:  u32,
    pub format:  TextureFormat,
    pub data:    Vec<u8>,
    pub mip_levels: u32,    // 1 = no mipmaps, 0 = auto-generate
    pub label:   Option<String>,
}

pub struct Sampler {
    pub mag_filter:   FilterMode,     // Linear / Nearest
    pub min_filter:   FilterMode,
    pub mip_filter:   FilterMode,
    pub address_u:    AddressMode,    // Repeat / MirrorRepeat / ClampToEdge
    pub address_v:    AddressMode,
    pub anisotropy:   u8,             // 1..=16, clamped to device limit
}

pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba16Float,
    Depth32Float,
    Bc7RgbaUnorm,      // compressed — desktop only
    Astc4x4RgbaUnorm,  // compressed — mobile / Apple Silicon
}
```

---

### 4.9 `scenix-renderer`

**Responsibility:** The GPU layer. Owns the `wgpu::Device`, `wgpu::Queue`, and all GPU resources. Consumes a `SceneGraph` and produces a rendered frame.

**Depends on:** all other crates except `scenix-animato`, `scenix-wasm`

scenix uses a **hybrid forward+ / deferred** pipeline:

#### GPU-side material trait (lives HERE, not in `scenix-material`)

```rust
/// Extension trait — bridges CPU-side Material → GPU bind groups.
/// Implemented by PbrMaterial, PhysicalMaterial, ToonMaterial, etc.
pub trait GpuMaterial: Material {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
    where Self: Sized;
    fn to_uniform_bytes(&self) -> Vec<u8>;    // serialized uniform buffer content
    fn create_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout,
                         textures: &TextureStore) -> wgpu::BindGroup;
}
```

#### Render architecture

- **Opaque geometry** → deferred G-buffer pass → deferred lighting resolve
- **Transparent geometry** → forward pass with OIT (order-independent transparency)
- **Shadow geometry** → depth-only shadow passes (before main pass)
- **Post-processing** → full-screen quad passes

```rust
pub struct Renderer {
    device:         wgpu::Device,
    queue:          wgpu::Queue,
    surface:        wgpu::Surface<'static>,
    config:         RendererConfig,
    pipeline_cache: PipelineCache,
    gpu_scene:      GpuScene,
    shadow_maps:    ShadowMapAtlas,
    gbuffer:        GBuffer,
    post_stack:     Option<PostStack>,
}

pub struct RendererConfig {
    pub width:          u32,
    pub height:         u32,
    pub sample_count:   u32,      // MSAA (1, 4)
    pub vsync:          bool,
    pub hdr:            bool,
    pub present_mode:   wgpu::PresentMode,
    pub backends:       wgpu::Backends,   // Vulkan | Metal | DX12 | WebGPU
}

impl Renderer {
    pub async fn new(window: &dyn raw_window_handle::HasWindowHandle,
                     config: RendererConfig) -> Result<Self, ScenixError>;

    pub fn render(&mut self, scene: &SceneGraph,
                  camera: &PerspectiveCamera) -> Result<(), ScenixError>;

    pub fn resize(&mut self, width: u32, height: u32);
    pub fn set_post_stack(&mut self, stack: Option<PostStack>);
}
```

#### `src/pass/shadow_pass.rs`

The shadow pass renders each shadow-casting light's depth buffer into the `ShadowMapAtlas` — a single large texture array that all lights share.

```
For each DirectionalLight with shadow:
    1. Compute light-space view-projection (CSM cascade splits)
    2. Cull scene against light frustum
    3. Render depth-only with DepthMaterial into shadow map slice
    4. Upload ShadowMatrix to uniform buffer
```

#### `src/gpu_scene.rs`

Responsible for uploading CPU scene data to GPU buffers every frame. Uses double-buffered storage buffers to avoid stalls.

```rust
pub struct GpuScene {
    // Mesh data (uploaded once on creation, updated on mesh change)
    vertex_buffers:  HashMap<MeshId, wgpu::Buffer>,
    index_buffers:   HashMap<MeshId, wgpu::Buffer>,

    // Per-frame data (re-uploaded every frame)
    transform_buffer: wgpu::Buffer,    // Mat4[] — one per visible node
    light_buffer:     wgpu::Buffer,    // LightUniform[] — all scene lights
    camera_buffer:    wgpu::Buffer,    // CameraUniform — view/proj matrices

    // Texture data (uploaded on first use, cached)
    textures:   HashMap<TextureId, (wgpu::Texture, wgpu::TextureView)>,
    samplers:   HashMap<SamplerKey, wgpu::Sampler>,
    bind_groups: HashMap<MaterialId, wgpu::BindGroup>,
}
```

---

### 4.10 `scenix-loader`

**Responsibility:** Load 3D assets from disk or bytes into `SceneGraph`, `Geometry`, and `Texture2D` objects. Zero GPU dependency — loaders produce CPU-side data only.

**Status in v0.7.0:** shipped as an optional `std` crate. It decodes into CPU-side scenix data only; renderer users still register loaded meshes, materials, textures, and lights with `Renderer`.

**Depends on:** `scenix-math`, `scenix-core`, `scenix-scene`, `scenix-camera`, `scenix-mesh`, `scenix-material`, `scenix-light`, `scenix-texture`

#### `src/gltf.rs`

```rust
pub struct GltfLoader;

pub struct GltfAsset {
    pub scene:     SceneGraph,
    pub meshes:    BTreeMap<MeshId, Geometry>,
    pub materials: BTreeMap<MaterialId, PbrMaterial>,
    pub textures:  BTreeMap<TextureId, Texture2D>,
    pub samplers:  BTreeMap<TextureId, Sampler>,
    pub lights:    BTreeMap<LightId, LoadedLight>,
    pub cameras:   BTreeMap<CameraId, LoadedCamera>,
}

impl GltfLoader {
    pub fn load(&self, path: impl AsRef<Path>) -> Result<GltfAsset, ScenixError>;
    pub fn load_file(&self, path: impl AsRef<Path>) -> Result<GltfAsset, ScenixError>;
    pub fn load_bytes(&self, bytes: &[u8], base_dir: Option<PathBuf>) -> Result<GltfAsset, ScenixError>;
    pub async fn load_url(&self, url: &str) -> Result<GltfAsset, ScenixError>; // behind "http"
}
```

#### `src/cache.rs`

```rust
pub struct AssetCache<T> {
    assets: BTreeMap<PathBuf, Arc<T>>,
}

impl<T> AssetCache<T> {
    pub fn get_or_load(&mut self, path: impl AsRef<Path>, load: impl FnOnce(&Path) -> Result<T, ScenixError>) -> Result<Arc<T>, ScenixError>;
    pub fn invalidate(&mut self, path: impl AsRef<Path>) -> bool;
    pub fn clear(&mut self);
    pub fn len(&self) -> usize;
    pub fn contains(&self, path: impl AsRef<Path>) -> bool;
}
```

---

### 4.11 `scenix-post`

**Responsibility:** Full-screen post-processing effects, composited in a configurable stack.

**Status in v0.7.0:** shipped as an optional `std` + `wgpu` crate. `scenix-post` depends on `wgpu`, `scenix-core`, and `scenix-math`; `scenix-renderer` optionally depends on `scenix-post` behind its `post` feature. This dependency inversion avoids a renderer/post Cargo cycle while preserving `Renderer::set_post_stack`.

**Depends on:** `wgpu`, `scenix-core`, `scenix-math`

```rust
pub struct PostStack {
    effects: Vec<PostEffect>,
}

impl PostStack {
    pub fn new() -> Self;
    pub fn with_bloom(self, config: BloomConfig) -> Self;
    pub fn with_ssao(self, config: SsaoConfig) -> Self;
    pub fn with_tonemap(self, mapper: ToneMapper) -> Self;
    pub fn with_fxaa(self, config: FxaaConfig) -> Self;
    pub fn with_taa(self, config: TaaConfig) -> Self;
    pub fn with_smaa(self, config: SmaaConfig) -> Self;
    pub fn with_dof(self, config: DofConfig) -> Self;
    pub fn with_fog(self, config: FogPostConfig) -> Self;
    pub fn with_outline(self, config: OutlineConfig) -> Self;
    pub fn with_motion_blur(self, config: MotionBlurConfig) -> Self;
    pub fn apply_to_view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue,
                         input: &wgpu::TextureView, output: &wgpu::TextureView,
                         context: PostContext) -> Result<PostStats, ScenixError>;
}

pub struct BloomConfig {
    pub threshold:  f32,    // luminance threshold above which bloom applies
    pub intensity:  f32,    // bloom strength multiplier
    pub radius:     f32,    // blur spread in UV space
}

pub enum ToneMapper {
    None,
    Reinhard,
    Aces,
    Exposure(f32),
}

impl Renderer {
    pub fn set_post_stack(&mut self, stack: Option<PostStack>);
    pub fn post_stack(&self) -> Option<&PostStack>;
    pub fn post_stack_mut(&mut self) -> Option<&mut PostStack>;
}
```

---

### 4.12 `scenix-raycaster`

**Responsibility:** Ray–scene intersection for mouse picking, click detection, and line-of-sight queries. Uses a BVH for sub-millisecond queries on large scenes.

**Depends on:** `scenix-math`, `scenix-core`, `scenix-scene`, `scenix-mesh`

**Status in v0.8.0:** shipped as a default facade CPU crate. It builds a node-level SAH BVH over visible mesh-node world AABBs, then performs exact world-space triangle intersection for candidate nodes supplied by the caller's geometry store.

```rust
pub struct Raycaster {
    bvh:    Option<Bvh>,
    layers: u32,    // bitmask — only test nodes matching this layer mask
}

pub struct Intersection {
    pub node_id:     NodeId,
    pub mesh_id:     MeshId,
    pub material_id: MaterialId,
    pub distance:    f32,       // parametric t along ray
    pub point:       Vec3,      // world-space hit point
    pub normal:      Vec3,      // world-space surface normal at hit
    pub uv:          Vec2,      // UV coordinates at hit (for texture lookups)
}

pub trait GeometryProvider {
    fn geometry(&self, mesh_id: MeshId) -> Option<&Geometry>;
}

impl Raycaster {
    pub fn new() -> Self;
    pub fn with_layers(layers: u32) -> Self;
    pub fn build_bvh(
        &mut self,
        scene: &SceneGraph,
        meshes: &impl GeometryProvider,
    ) -> Result<(), ValidationError>;
    // Call build_bvh once after scene load, and again after structural changes.

    pub fn cast_ray(&self, ray: Ray3, scene: &SceneGraph, meshes: &impl GeometryProvider) -> Option<Intersection>;
    pub fn cast_ray_all(&self, ray: Ray3, scene: &SceneGraph, meshes: &impl GeometryProvider) -> Vec<Intersection>;
    pub fn cast_ray_all_bruteforce(&self, ray: Ray3, scene: &SceneGraph, meshes: &impl GeometryProvider) -> Vec<Intersection>;
    // cast_ray_all returns all intersections sorted by distance ascending.

    pub fn from_camera_ndc(camera: &PerspectiveCamera, ndc: Vec2) -> Ray3;
    // Convenience: build a Ray3 from a normalized device coordinate (mouse position).
}
```

#### `src/bvh.rs`

SAH (surface area heuristic) BVH — the industry standard for fast ray traversal:

```
Build:  O(N log N) — splits leaf sets at minimum SAH cost boundary
Query:  O(log N) average — skips entire subtrees when AABB misses
Memory: ~48 bytes per node (AABB + child indices)
```

---

### 4.13 `scenix-animato`

**Responsibility:** The bridge between Animato 1.4.0 animation values and scenix data. Allows Animato `Tween` and `Spring` values to drive scene node transforms, camera fields, PBR material fields, and explicit skeleton pose arrays.

**Depends on:** `scenix-math`, `scenix-core`, `scenix-scene`, `scenix-camera`, `scenix-material`, `animato = "1.4.0"`

**Status in v1.0.0:** shipped as an optional facade feature. The bridge uses local `AnimVec3`, `AnimQuat`, and `AnimColor` wrappers so scenix math/color types can participate in Animato interpolation without changing the underlying CPU crates.

```rust
pub struct NodeAnimator {
    pub node_id: NodeId,
    pub target:  NodeAnimationTarget,
}

pub enum NodeAnimationTarget {
    Translation(Vec3Track),
    Rotation(QuatTrack),
    Scale(Vec3Track),
    Visibility(BoolTrack),
}

pub struct MaterialAnimator {
    pub material_id: MaterialId,
    pub target:      MaterialAnimationTarget,
}

pub enum MaterialAnimationTarget {
    Albedo(ColorTrack),
    Opacity(ScalarTrack),
    Emissive(Vec3Track),
    Roughness(ScalarTrack),
    Metallic(ScalarTrack),
}

pub struct ScenixAnimationDriver {
    node_animators:     Vec<NodeAnimator>,
    camera_animators:   Vec<CameraAnimator>,
    material_animators: Vec<MaterialAnimator>,
    skeleton_animators: Vec<SkinnedMeshAnimator>,
}

impl ScenixAnimationDriver {
    pub fn tick(&mut self, dt: f32, scene: &mut SceneGraph, cameras: &mut impl CameraStoreMut, materials: &mut impl PbrMaterialStoreMut, skeletons: &mut [SkeletonPose]);
    pub fn add_node(&mut self, animator: NodeAnimator);
    pub fn add_camera(&mut self, animator: CameraAnimator);
    pub fn add_material(&mut self, animator: MaterialAnimator);
    pub fn add_skeleton(&mut self, animator: SkinnedMeshAnimator);
    pub fn pause(&mut self);
    pub fn resume(&mut self);
}
```

**Usage example:**

```rust
use scenix_animato::{NodeAnimator, NodeAnimationTarget, ScenixAnimationDriver, Vec3Track};
use scenix_math::Vec3;

let mut driver = ScenixAnimationDriver::new();
driver.add_node(NodeAnimator::new(
    sword_node,
    NodeAnimationTarget::Translation(Vec3Track::tween(Vec3::ZERO, Vec3::Y, 0.5)),
));
```

---

### 4.14 `scenix-wasm`

**Responsibility:** Browser-specific integrations: selecting a canvas backend, creating a `wgpu::Surface` from a `<canvas>` when WebGPU is safe, using a WebGL compatibility renderer when WebGPU is unavailable, forwarding DOM input events into scenix input types, clamping canvas sizes, and providing generated-scene browser wrappers.

**Depends on:** `scenix-renderer`, `scenix-scene`, `scenix-camera`, `scenix-input`, `wasm-bindgen`, `web-sys`

**Status in v1.1.0:** shipped as an optional facade feature. The browser wrapper powers the generated "Scenix Engine Lab" website demo with cube, sphere, torus, floor, helper visuals, DOM input, raycast selection, animation toggles, WebGPU/WebGL backend selection, and no network asset loading.

```rust
#[wasm_bindgen]
pub struct WebRenderer {
    renderer: Renderer,
    scene:    SceneGraph,
    camera:   PerspectiveCamera,
    pointer:  PointerState,
    keyboard: KeyboardState,
}

#[wasm_bindgen]
pub struct BrowserRenderer {
    // WebGPU first when safe, WebGL fallback otherwise.
}

#[wasm_bindgen]
pub struct WebGlRenderer {
    // WebGL1-compatible compatibility backend.
}

#[wasm_bindgen]
impl WebRenderer {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<WebRenderer, JsValue>;

    pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue>;
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue>;
    pub fn on_pointer_move(&mut self, x: f32, y: f32);
    pub fn on_pointer_down(&mut self, button: i16, x: f32, y: f32);
    pub fn on_pointer_up(&mut self, button: i16, x: f32, y: f32);
    pub fn on_wheel(&mut self, delta_y: f32);
    pub fn on_key_down(&mut self, code: &str);
    pub fn on_key_up(&mut self, code: &str);
}
```

---

### 4.15 `scenix-helpers`

**Responsibility:** Debug visualization helpers. Generate line geometry for grids, axes, bounding boxes, light cones, camera frustums, and skeleton bones. Essential for scene debugging — Three.js ships 12+ helpers; scenix matches them.

**Depends on:** `scenix-math`, `scenix-core`, `scenix-light`, `scenix-camera`

**Status in v0.8.0:** shipped as a default facade CPU crate. Helpers output `LineGeometry` instead of weakening `scenix-mesh::Geometry` triangle validation.

```rust
pub struct LineGeometry {
    pub positions: Vec<Vec3>,
    pub colors:    Vec<Color>,
    pub indices:   Vec<u32>,    // optional line-list indices
}

pub struct GridHelper {
    pub size:      f32,       // total grid extent
    pub divisions: u32,
    pub color1:    Color,     // center line color
    pub color2:    Color,     // grid line color
}

pub struct AxesHelper {
    pub size: f32,            // length of each axis line
    // X = red, Y = green, Z = blue — standard convention
}

pub struct BoundingBoxHelper {
    pub aabb:  Aabb,
    pub color: Color,
}

pub struct ArrowHelper {
    pub origin:    Vec3,
    pub direction: Vec3,
    pub length:    f32,
    pub color:     Color,
    pub head_length: f32,
    pub head_width:  f32,
}

impl GridHelper {
    pub fn to_geometry(&self) -> LineGeometry;
}

impl AxesHelper {
    pub fn to_geometry(&self) -> LineGeometry;
}

impl ArrowHelper {
    pub fn to_geometry(&self) -> LineGeometry;
}
```

---

### 4.16 `scenix-input`

**Responsibility:** Shared input state types used by `scenix-camera` controllers and `scenix-wasm` event forwarding. Platform-agnostic — no dependency on winit, web-sys, or any windowing library.

**Depends on:** `scenix-math`

```rust
pub struct PointerState {
    pub position:  Vec2,       // current position in pixels
    pub delta:     Vec2,       // movement since last frame
    pub buttons:   u8,         // bitmask: bit 0 = left, bit 1 = right, bit 2 = middle
    pub pressed:   bool,
}

pub struct KeyboardState {
    pressed: HashSet<KeyCode>,
    modifiers: Modifiers,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    KeyW, KeyA, KeyS, KeyD, KeyQ, KeyE,
    Space, ShiftLeft, ControlLeft,
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    // ... additional keys
}

pub struct Modifiers {
    pub shift:   bool,
    pub ctrl:    bool,
    pub alt:     bool,
    pub meta:    bool,
}

impl KeyboardState {
    pub fn is_pressed(&self, key: KeyCode) -> bool;
    pub fn on_key_down(&mut self, key: KeyCode);
    pub fn on_key_up(&mut self, key: KeyCode);
}
```

---

### 4.17 `scenix` (facade)

**Responsibility:** The single crate users add to `Cargo.toml`. Re-exports everything from every sub-crate behind feature flags. Users never need to add the individual crates directly.

```toml
[dependencies]
scenix = "1"

# Or with specific features:
scenix = { version = "1", features = ["loader", "renderer", "post"] }
```

```rust
// scenix/src/lib.rs — re-exports grouped by feature
pub use scenix_math::*;
pub use scenix_core::*;
pub use scenix_scene::*;
pub use scenix_camera::*;
pub use scenix_mesh::*;
pub use scenix_material::*;
pub use scenix_light::*;
pub use scenix_texture::*;
pub use scenix_input::*;

#[cfg(feature = "renderer")]  pub use scenix_renderer::*;
#[cfg(feature = "loader")]    pub use scenix_loader::*;
#[cfg(feature = "post")]      pub use scenix_post::*;
#[cfg(feature = "raycaster")] pub use scenix_raycaster::*;
#[cfg(feature = "animato")]   pub use scenix_animato::*;
#[cfg(feature = "helpers")]   pub use scenix_helpers::*;
#[cfg(feature = "wasm")]      pub use scenix_wasm::*;
```

---

## 5. Data Flow & Render Loop

```
User code
    │
    ├─ build SceneGraph (nodes, transforms, mesh/light/camera attachments)
    ├─ load assets via scenix-loader (→ Geometry, Texture2D, PbrMaterial)
    ├─ register meshes/materials/textures with Renderer
    │
    ▼
Every frame:
    │
    ├─[1] ScenixAnimationDriver::tick(dt)      ← update all animato-driven properties
    │        └─ writes back to SceneGraph transforms and MaterialStore
    │
    ├─[2] SceneGraph::propagate_transforms()  ← recompute dirty world matrices
    │
    ├─[3] Frustum culling                     ← filter visible nodes per camera
    │        └─ produces RenderList (sorted opaque + transparent)
    │
    ├─[4] Shadow pass                         ← render depth for each shadow light
    │        └─ updates ShadowMapAtlas
    │
    ├─[5] GpuScene::upload(frame)             ← upload transform/light uniforms to GPU
    │
    ├─[6] Geometry pass (deferred)            ← write G-buffer (albedo, normal, metallic, depth)
    │
    ├─[7] Lighting pass                       ← resolve G-buffer with light data + shadow maps
    │
    ├─[8] Forward pass (transparent)          ← render alpha-blended objects over deferred result
    │
    ├─[9] Post-processing stack               ← SSAO → Bloom → TAA → ToneMap → FXAA
    │
    └─[10] surface.present()                  ← swap buffers
```

---

## 6. Type System Design

### ID Newtypes vs Pointers

scenix never stores raw pointers or `Arc<Mutex<T>>` for scene nodes. All cross-crate references use `NodeId`, `MeshId`, `MaterialId`, etc. — typed newtypes over `u64` backed by `SlotMap`.

Benefits:
- IDs are `Copy` — zero-cost to pass around
- No lifetime parameters on `SceneGraph` or `Renderer`
- Safe across threads when IDs are sent to a render thread
- Serializable without ptr fixup

### Material Trait Object vs Enum

Materials use a `Box<dyn Material>` trait object, not an enum, because:
- Users need to define custom `ShaderMaterial` types
- The material set is open — not closed like easings in animato
- The renderer needs to downcast to extract uniform bytes without knowing every material type

### Transform Propagation — Lazy, Not Immediate

World transforms are NOT recomputed when `set_local_transform` is called. Instead, the node is marked dirty. Propagation runs as a single batched pass at the start of each frame (step 2 above). This avoids cascading recomputes when many children move together.

---

## 7. GPU Architecture

### Buffer Strategy

| Buffer | Type | Update frequency |
|--------|------|-----------------|
| Vertex positions | `VERTEX` | Once on upload |
| Index buffer | `INDEX` | Once on upload |
| Instance transforms | `STORAGE` | Every frame |
| Light uniforms | `UNIFORM` | Every frame |
| Camera uniforms | `UNIFORM` | Every frame |
| Shadow matrices | `UNIFORM` | Every frame |
| Material uniforms | `UNIFORM` | On material change |

### Pipeline Cache

Every material type + topology + blend mode combination produces a unique `PipelineKey`. Compiled `wgpu::RenderPipeline` objects are cached behind this key. Pipeline compilation (which stalls the GPU) only happens the first time a new combination is seen.

### WGSL Shaders

All shaders are written in WGSL (WebGPU Shading Language) for full portability across Vulkan, Metal, DX12, and WebGPU. Shaders are embedded at compile time via `include_str!`.

```
shaders/
├── mesh.vert.wgsl          ← vertex shader (used by all opaque materials)
├── pbr.frag.wgsl           ← PBR fragment shader (GGX BRDF, IBL)
├── unlit.frag.wgsl         ← unlit fragment shader
├── shadow_depth.vert.wgsl  ← shadow pass vertex shader
├── deferred_resolve.wgsl   ← lighting pass full-screen quad
├── post/
│   ├── bloom_down.wgsl
│   ├── bloom_up.wgsl
│   ├── ssao.wgsl
│   ├── tonemap.wgsl
│   └── fxaa.wgsl
```

### Deferred vs Forward

| Path | When used | Why |
|------|-----------|-----|
| Deferred G-buffer | Opaque geometry | Efficient with many lights (O(lights) not O(lights×triangles)) |
| Forward+ | Transparent geometry | Required — G-buffer cannot handle blending |
| Depth-only shadow | Shadow casters | No fragment work needed |

---

## 8. Feature Flag Strategy

```toml
# scenix/Cargo.toml features
[features]
default  = ["std", "scene", "camera", "mesh", "material", "light", "texture", "raycaster", "helpers"]
std      = []                               # enables std-dependent types
scene    = ["dep:scenix-scene"]
camera   = ["dep:scenix-camera"]
mesh     = ["dep:scenix-mesh"]
material = ["dep:scenix-material"]
light    = ["dep:scenix-light"]
texture  = ["dep:scenix-texture"]
renderer = ["dep:scenix-renderer"]
loader   = ["dep:scenix-loader"]
post     = ["dep:scenix-post", "scenix-renderer?/post"]
raycaster = ["dep:scenix-raycaster"]
animato  = ["dep:scenix-animato"]
helpers  = ["dep:scenix-helpers"]
wasm     = ["dep:scenix-wasm"]
serde    = ["scenix-math/serde", "scenix-core/serde", "scenix-input/serde",
            "scenix-scene?/serde", "scenix-camera?/serde", "scenix-mesh?/serde",
            "scenix-material?/serde", "scenix-light?/serde", "scenix-texture?/serde",
            "scenix-loader?/serde", "scenix-post?/serde", "scenix-renderer?/serde",
            "scenix-raycaster?/serde", "scenix-helpers?/serde",
            "scenix-animato?/serde", "scenix-wasm?/serde"]
```

**Minimum useful combination** — scene graph and authoring data only, zero GPU:

```toml
scenix = { version = "1", default-features = false, features = ["scene", "camera", "mesh", "material", "light", "texture", "raycaster", "helpers"] }
```

**Renderer opt-in** — add the `wgpu` layer only when an application needs GPU rendering:

```toml
scenix = { version = "1", features = ["renderer"] }
```

**Loader + post opt-in** — load CPU assets and enable renderer post effects:

```toml
scenix = { version = "1", features = ["loader", "renderer", "post"] }
```

**Integration opt-in** — add Animato or browser wrappers without changing the default CPU authoring set:

```toml
scenix = { version = "1", features = ["animato"] }
scenix = { version = "1", features = ["wasm"] }
```

---

## 9. Error Handling Strategy

All fallible operations return `Result<T, ScenixError>`. There are no panics in library code except on logic errors that are programmer mistakes (e.g. passing an invalid `NodeId` to `get`).

```rust
#[derive(Debug, thiserror::Error)]
pub enum ScenixError {
    #[error("GPU initialization failed: {0}")]
    GpuInit(String),

    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),

    #[error("Asset load error: {0}")]
    Load(#[from] LoadError),

    #[error("Invalid node ID: {0:?}")]
    InvalidNode(NodeId),

    #[error("Shader compilation error: {0}")]
    Shader(String),
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("GLTF parse error: {0}")]
    Gltf(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}
```

---

## 10. Testing Strategy

### Unit tests (in crate source)

- `scenix-math`: every operation has a unit test. Uses `approx::assert_abs_diff_eq!` with `1e-6` epsilon.
- `scenix-scene`: tests for parent/child correctness, world transform propagation, dirty flag.
- `scenix-mesh`: vertex count, normal validity (normalized), UV range [0,1], AABB correctness for each primitive.
- `scenix-camera`: frustum plane extraction, contains/intersects/outside for all visibility cases.

### Integration tests (`tests/`)

- `scene_hierarchy.rs` — build a 10-deep parent chain, verify world transforms compose correctly.
- `camera_frustum.rs` — known points inside/outside frustum, sphere/AABB tests.
- `mesh_primitives.rs` — generate every primitive, check vertex count formula, check normals are unit length.
- `loader_gltf.rs` — load the Khronos reference GLTF sample assets, verify node counts and material properties.
- `raycaster_correctness.rs` — cast known rays at known triangles, verify hit distance within 1e-5.

### GPU tests (headless `wgpu`)

Run on CI with `wgpu`'s `Vulkan` backend via `softbuffer` or Vulkan lavapipe:
- Render a single triangle, readback pixel, verify non-black output.
- Render PBR sphere under directional light, verify pixel luminance in expected range.

### Snapshot tests

For visual regression: render reference scenes to PNG, compare against committed reference images with a pixel tolerance. Uses `image::DiffOptions`.

---

## 11. Performance Guidelines

### Transform propagation

- Use a dirty bitset — never walk the full tree on every frame.
- Process nodes in topological order (parents before children) using a pre-sorted index.
- For > 10K nodes, use parallel iteration with Rayon behind a `parallel` feature flag.

### GPU buffer uploads

- Double-buffer transform and light data — write frame N+1 while GPU renders frame N.
- Use `wgpu::Buffer::map_async` only when the buffer size changes; otherwise use `queue.write_buffer`.
- Batch all per-node transform uploads into a single `write_buffer` call per frame.

### Mesh GPU memory

- Interleave vertex attributes (position, normal, UV) in one buffer — improves cache coherence.
- Use `u16` indices when vertex count ≤ 65535, `u32` otherwise — halves index buffer size for small meshes.
- Share vertex buffers for instanced meshes — the instance buffer stores only transforms.

### Frustum culling

- Test AABB against frustum before any mesh submission — eliminate invisible objects early.
- Sort opaque draw calls front-to-back (depth pre-rejection by GPU).
- Sort transparent draw calls back-to-front (correct blending order).

### Pipeline caching

- Cache `wgpu::RenderPipeline` by `PipelineKey` in a `HashMap` — compilation is expensive.
- Log a warning when a new pipeline is compiled mid-frame (signals a missing warm-up pass).

---

## 12. Integration Targets

scenix has three runtime families. The same CPU scene, material, camera, loader, animation, raycasting, and helper crates should remain portable across all of them.

| Runtime Family | Primary Targets | Renderer Path | Notes |
|----------------|-----------------|---------------|-------|
| Native desktop | Linux, Windows, macOS | `scenix-renderer` on `wgpu` | Window/event loop supplied by app frameworks such as winit, Tauri, egui, Iced, or custom shells. |
| Native mobile | Android, iOS | `scenix-renderer` on `wgpu` | App lifecycle, touch/gamepad input, surface loss, DPI, and compressed texture support must be treated as core product requirements. |
| Web/browser | WASM browsers and WebView apps | `scenix-wasm` with WebGPU first, WebGL2 fallback | The website demo is one deployment target, not the whole platform strategy. |

### Native Desktop (winit + wgpu)

```rust
use scenix::{Renderer, RendererConfig, SceneGraph, PerspectiveCamera};
use winit::event_loop::EventLoop;

let event_loop = EventLoop::new().unwrap();
let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
let config = RendererConfig::default();
let mut renderer = pollster::block_on(Renderer::new(&window, config)).unwrap();
```

### Native Mobile (Android / iOS)

Mobile uses the same `SceneGraph`, camera, material, texture, loader, animation, raycasting, and renderer concepts as desktop. The integration layer is responsible for creating a native window/surface and forwarding app lifecycle events into resize, suspend, resume, and input handlers.

Required mobile design rules:

- handle surface loss and recreation as a normal path;
- keep all heavy systems behind feature flags so CPU-only authoring still compiles for constrained targets;
- support touch, gesture, keyboard, and gamepad input through `scenix-input`;
- prefer ASTC/ETC2 texture paths where the target GPU supports them;
- avoid browser-only assumptions in renderer, loader, scene, animation, and material APIs.

### WASM / Browser (WebGPU With WebGL Fallback)

```rust
use scenix_wasm::BrowserRenderer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() {
    let canvas = web_sys::window().unwrap()
        .document().unwrap()
        .get_element_by_id("canvas").unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let renderer = BrowserRenderer::new(canvas).await.unwrap();
    // drive from requestAnimationFrame
}
```

### With animato

```rust
use scenix::*;
use scenix_animato::{NodeAnimator, NodeAnimationTarget, ScenixAnimationDriver, Vec3Track};

let mut anim_driver = ScenixAnimationDriver::new();

anim_driver.add_node(NodeAnimator::new(
    camera_node,
    NodeAnimationTarget::Translation(Vec3Track::tween(Vec3::ZERO, Vec3::Y, 0.5)),
));

// In render loop:
anim_driver.tick(dt, &mut scene, &mut cameras, &mut materials, &mut skeletons)?;
renderer.render(&scene, &camera)?;
```

### Scene-graph only (no GPU)

```rust
use scenix::{SceneGraph, SceneNode, Transform};

// Build a scene without ever touching wgpu — useful for server-side, testing, or tools.
let mut scene = SceneGraph::new();
let root = scene.add(SceneNode::new("Root"));
let child = scene.add_child(root, SceneNode::new("Child")
    .transform(Transform::from_translation(Vec3::new(1.0, 0.0, 0.0))));

let world = scene.world_transform(child);
```

---

## 13. CI / CD Pipeline

### `ci.yml` — runs on every PR and push to main

```
Jobs:
  test:
    matrix: [stable, beta, nightly]
    steps:
      - cargo test --workspace --all-features
      - cargo test --workspace --no-default-features
      - cargo clippy --workspace --all-features -- -D warnings
      - cargo fmt --check

  docs:
    - cargo doc --workspace --all-features --no-deps

  wasm:
    - wasm-pack test --headless --chrome --features wasm

  gpu-headless:
    - cargo test --workspace --features renderer (with softbuffer / lavapipe)

  bench:
    - cargo bench --workspace --no-run
```

### `publish.yml` — runs on version tags (`v*`)

```
Steps:
  - Verify tag matches version in each Cargo.toml
  - cargo publish -p scenix-math
  - cargo publish -p scenix-core
  - ... (in dependency order)
  - cargo publish -p scenix
```

---

## 14. Publishing Checklist

Before `cargo publish` for any crate:

- [ ] All `pub` items have `///` doc comments with at least one example
- [ ] `README.md` has a quick-start example that compiles with `cargo test --doc`
- [ ] `CHANGELOG.md` has an entry for this version
- [ ] `LICENSE-MIT` and `LICENSE-APACHE` are present at workspace root
- [ ] `cargo test --workspace` passes — zero warnings
- [ ] `cargo test --workspace --no-default-features` passes
- [ ] `cargo test --workspace --all-features` passes
- [ ] `cargo clippy --workspace --all-features -- -D warnings` is clean
- [ ] `cargo doc --workspace --all-features --open` renders correctly
- [ ] `cargo bench --workspace --no-run` compiles without errors
- [ ] Version in `Cargo.toml` matches git tag and `CHANGELOG.md` entry
- [ ] `cargo publish --dry-run` succeeds for the crate being released

### Publish order (dependency chain)

```
scenix-math → scenix-core → scenix-input → scenix-scene → scenix-camera → scenix-mesh
          → scenix-material → scenix-light → scenix-texture
          → scenix-loader → scenix-post → scenix-renderer
          → scenix-raycaster → scenix-helpers → scenix-animato → scenix-wasm → scenix
```

---

## 15. Naming & Style Conventions

### Crate naming

`scenix-{concern}` — Italian prefix, lowercase, hyphen-separated.
The facade crate is simply `scenix`.

### Type naming

| Type | Convention | Example |
|------|------------|---------|
| Math types | `PascalCase`, concise | `Vec3`, `Mat4`, `Quat`, `Transform`, `Euler` |
| Scene types | `PascalCase` | `SceneGraph`, `SceneNode`, `NodeKind`, `LodGroup` |
| Camera types | `{Type}Camera` | `PerspectiveCamera`, `OrthographicCamera`, `CubeCamera` |
| Material types | `{Type}Material` | `PbrMaterial`, `PhysicalMaterial`, `ToonMaterial` |
| Light types | `{Type}Light` | `DirectionalLight`, `HemisphereLight` |
| Helper types | `{Type}Helper` | `GridHelper`, `AxesHelper`, `ArrowHelper` |
| Config structs | `{Type}Config` | `RendererConfig`, `ShadowConfig`, `BloomConfig` |
| ID newtypes | `{Type}Id` over `u64` | `NodeId`, `MeshId`, `MaterialId` |
| Enums | `PascalCase`, descriptive | `NodeKind`, `AlphaMode`, `ToneMapper`, `Visibility` |
| Traits | `PascalCase`, noun or adjective | `Material`, `GpuMaterial`, `Bounded`, `Named` |
| Errors | `{Domain}Error` | `ScenixError`, `LoadError` |

### Public vs private fields

| Field type | Visibility |
|------------|------------|
| Configuration (`fov_y`, `intensity`, `roughness`) | `pub` — users inspect and set |
| Internal GPU state (`wgpu::Buffer`, `wgpu::Pipeline`) | Private — managed by renderer |
| Cached/derived data (`world_cache`, `dirty`) | Private — managed by graph |

### Module-level documentation

Every `lib.rs` must have a crate-level `//!` doc block with:
1. One-sentence summary
2. Quick-start example (compiles as `cargo test --doc`)
3. Feature flags table
4. Link to the `scenix` facade crate and `animato`

---

## 16. Platform Support & Framework Integration

scenix runs on every platform that `wgpu` supports. The architecture splits GPU-free crates from GPU-dependent crates, enabling use on platforms without a GPU.

### Supported Platforms

| Platform | GPU Backend | Crates Available | How to Build |
|----------|-------------|------------------|-------------|
| **Linux** | Vulkan | All 17 crates | `cargo build` |
| **Windows** | Vulkan / DX12 | All 17 crates | `cargo build` |
| **macOS** | Metal | All 17 crates | `cargo build` |
| **Android** | Vulkan | All 17 crates | `cargo ndk -t arm64-v8a build` |
| **iOS** | Metal | All 17 crates | `cargo build --target aarch64-apple-ios` |
| **Web (WASM)** | WebGPU / WebGL2 | All 17 crates | `wasm-pack build --target web` |
| **Embedded** | None | `scenix-math`, `scenix-core`, `scenix-scene`, `scenix-input` | `cargo build --no-default-features --features libm` |

### Platform Layering

```
┌─────────────────────────────────────────────┐
│       Application / Game Logic              │  ← user code
├─────────────────────────────────────────────┤
│  scenix-scene, scenix-mesh, scenix-material,   │
│  scenix-camera, scenix-light, scenix-math,     │  ← GPU-free (runs everywhere)
│  scenix-core, scenix-input, scenix-raycaster,  │
│  scenix-helpers                                │
├─────────────────────────────────────────────┤
│       scenix-renderer (wgpu)                 │  ← needs GPU
├──────────┬──────────┬──────────┬────────────┤
│  Vulkan  │  Metal   │  DX12    │  WebGPU    │  ← wgpu auto-selects
├──────────┼──────────┼──────────┼────────────┤
│  Linux   │  macOS   │ Windows  │  Browser   │
│  Android │  iOS     │          │  Mobile    │
└──────────┴──────────┴──────────┴────────────┘
```

### Rust Framework Integration

scenix integrates with any Rust framework that provides a `<canvas>` element (web) or a window handle (native):

| Framework | Platform | Integration Method |
|-----------|----------|-------------------|
| **winit** | Desktop + Mobile | `Renderer::new(window)` — direct wgpu surface |
| **Tauri** | Desktop (WebView) | `scenix-wasm` renders in `<canvas>` inside WebView |
| **Leptos** | Web (WASM) | `scenix-wasm` in Leptos `<canvas>` component |
| **Dioxus** | Web + Desktop + Mobile | `scenix-wasm` in `<canvas>` element |
| **Yew** | Web (WASM) | `scenix-wasm` in Yew `<canvas>` component |
| **egui** | Desktop + Web | Shared wgpu device — egui overlay on scenix viewport |
| **Iced** | Desktop + Web | scenix renders into an Iced widget surface |
| **Bevy** | Desktop + Web + Mobile | `scenix-scene` as data source, Bevy as renderer |

### Cross-Platform One-Codebase Pattern

```rust
// src/lib.rs — 95% of your code lives here, platform-agnostic
pub struct MyApp {
    scene:    SceneGraph,
    camera:   PerspectiveCamera,
    renderer: Renderer,
}

impl MyApp {
    pub fn setup(&mut self) { /* build scene */ }
    pub fn update(&mut self, dt: f32, input: &PointerState) {
        self.renderer.render(&self.scene, &self.camera);
    }
}

// Desktop:  5 lines with winit
// Browser:  5 lines with scenix-wasm
// Mobile:   same as desktop via winit + NDK/Metal
```

### `no_std` Support Matrix

| Crate | `no_std` | Notes |
|-------|----------|-------|
| `scenix-math` | ✅ | Uses `libm` for trig when `std` is disabled |
| `scenix-core` | ✅ | IDs, traits, Color — zero allocations |
| `scenix-scene` | ✅ | Uses `alloc` for graph storage |
| `scenix-input` | ✅ | Pure data types |
| CPU authoring crates | ✅ | `scenix-camera`, `scenix-mesh`, `scenix-material`, `scenix-light`, and `scenix-texture` compile without default features |
| `scenix-loader` | ❌ | Requires `std` and parser/decoder crates |
| `scenix-renderer` | ❌ | Requires `std` + `wgpu` |
| `scenix-post` | ❌ | Requires `std` + `wgpu` |
| `scenix-raycaster` | ✅ | Uses `alloc` for BVH and hit vectors |
| `scenix-helpers` | ✅ | Uses `alloc` for `LineGeometry` |
| `scenix-animato` | ✅ | Uses `alloc` and Animato 1.4.0 tween/spring features |
| `scenix-wasm` | ❌ | Browser-only wrapper requiring `std`, `wasm-bindgen`, `web-sys`, WebGL bindings, and `wgpu` for the WebGPU path |

---

## 17. Future Three.js Parity Plan

This section records the long-term feature direction. It does not change the stable v1 contract: future work should remain modular, additive, feature-gated, and usable across desktop, mobile, and web wherever the underlying platform allows it.

### Future Version Architecture Map

The roadmap assigns future work to concrete planning versions so contributors know which crates to touch. These are planning buckets, not release promises.

| Version | Architecture Theme | New Optional Crate(s) | Existing Crates Updated |
|---------|--------------------|-----------------------|-------------------------|
| `v1.2.0` | Renderer and material parity | None | `scenix-renderer`, `scenix-material`, `scenix-texture`, `scenix-light`, `scenix-post`, `scenix-wasm`, `scenix` |
| `v1.3.0` | Asset pipeline | None | `scenix-loader`, `scenix-mesh`, `scenix-material`, `scenix-texture`, `scenix-scene`, `scenix-camera`, `scenix-light`, `scenix-animato`, `scenix-renderer`, `scenix` |
| `v1.4.0` | Animation runtime | None | `scenix-animato`, `scenix-loader`, `scenix-mesh`, `scenix-scene`, `scenix-material`, `scenix-camera`, `scenix-light`, `scenix-renderer`, `scenix-helpers`, `scenix` |
| `v1.5.0` | Controls, interaction, and editor primitives | None | `scenix-camera`, `scenix-input`, `scenix-raycaster`, `scenix-helpers`, `scenix-scene`, `scenix-renderer`, `scenix-wasm`, `scenix` |
| `v1.6.0` | Shader nodes and node materials | `scenix-nodes` | `scenix-material`, `scenix-renderer`, `scenix-post`, `scenix-wasm`, `scenix` |
| `v1.7.0` | Particles | `scenix-particles` | `scenix-scene`, `scenix-mesh`, `scenix-material`, `scenix-texture`, `scenix-renderer`, `scenix-wasm`, `scenix` |
| `v1.8.0` | Terrain, sky, and water | `scenix-terrain`, `scenix-sky`, `scenix-water` | `scenix-renderer`, `scenix-material`, `scenix-texture`, `scenix-light`, `scenix-scene`, `scenix` |
| `v1.9.0` | XR, audio, and physics bridges | `scenix-xr`, `scenix-audio`, `scenix-physics` | `scenix-scene`, `scenix-input`, `scenix-camera`, `scenix-helpers`, `scenix-renderer`, `scenix-wasm`, `scenix` |
| `v1.10.0` | Editor and UI tooling | `scenix-editor`, `scenix-ui` or `scenix-egui` | `scenix-scene`, `scenix-renderer`, `scenix-material`, `scenix-loader`, `scenix-animato`, `scenix-helpers`, `scenix-wasm`, `scenix` |

Future crates become active workspace members only when their crate directory, docs, tests, examples, and facade feature exist. Until then, the architecture treats them as planned extension points.

### Planning Priorities

| Priority | Theme | Why It Comes First |
|----------|-------|--------------------|
| P0 | Renderer and material parity | The current renderer is stable for previews and examples; production parity needs real texture binding, lights, shadows, IBL, and physical shading. |
| P1 | Asset pipeline depth | A Three.js-like ecosystem depends on loading, exporting, compressing, caching, and hot-reloading common 3D formats. |
| P2 | Controls, interaction, and editor primitives | Product viewers, tools, games, and editors need transform gizmos, drag controls, pointer lock, selection volumes, and inspectors. |
| P3 | Scene effects and simulation bridges | Sky, water, terrain, particles, audio, physics, and XR should arrive as optional crates or optional renderer modules. |
| P4 | Resource lifecycle and quality gates | Production users need predictable upload, update, cleanup, diagnostics, visual tests, and platform compatibility notes. |

### Renderer And Material Parity

The renderer roadmap should move from preview shading to production shading without breaking the renderer-agnostic data crates.

Required future capabilities:

- real `PbrMaterial` GPU path with albedo, metallic-roughness, normal, occlusion, emissive, vertex color, alpha mask, alpha blend, and double-sided support;
- real `PhysicalMaterial` GPU path with clearcoat, sheen, transmission, thickness, IOR, iridescence, attenuation, and environment response;
- GPU texture allocation and binding for `Texture2D`, `TextureCube`, `Texture3D`, video textures, compressed textures, mipmaps, samplers, and render targets;
- image-based lighting with environment maps, irradiance, prefiltered radiance, BRDF lookup, PMREM-style filtering, and optional light probes;
- shadow pipeline implementation for directional, point, and spot lights, including atlas management, PCF, cascades, and future SDSM/CSM improvements;
- color management and tone mapping policy that is consistent across desktop, mobile, WebGPU, and WebGL fallback paths;
- render target family covering 2D, cube, 3D, array, depth, multisampled, HDR, readback, and offscreen capture targets;
- renderer diagnostics for frame stats, GPU timings where supported, resource counts, shader/pipeline cache stats, and debug labels;
- GPU-driven rendering path for indirect draws, GPU culling, clustered/forward+ lighting, and high-instance-count scenes.

### Resource Lifecycle And Scene Synchronization

Scenix should make the common path easy while keeping explicit ownership. CPU crates own authoring data; renderer crates own GPU resources; helper APIs should connect them without hiding cost.

Required future capabilities:

- `register_*`, `update_*`, `unregister_*`, and `clear_*` APIs for meshes, materials, textures, lights, cameras, render targets, and post resources;
- bulk scene upload helpers that convert loaded assets or application stores into renderer registrations with stable ID mapping;
- dirty/version tracking for geometry, material fields, textures, transforms, lights, skeletons, morph weights, and animation-driven data;
- scene-to-renderer sync plan that reports created, updated, reused, skipped, and removed resources before or after upload;
- explicit GPU memory accounting for vertex buffers, index buffers, textures, uniform buffers, render targets, shadow maps, and post scratch targets;
- resource budget controls for texture memory, geometry memory, shadow atlases, post targets, and asset caches;
- deterministic cleanup through Rust `Drop` plus explicit dispose/unregister APIs for long-running editors and hot-reload workflows;
- asset-to-GPU convenience helpers for glTF/OBJ/STL/image assets while preserving lower-level manual registration;
- clear error categories for unsupported format, unsupported GPU feature, invalid resource ID, stale handle, budget exceeded, upload failure, and device/surface loss;
- reload and hot-swap paths for meshes, textures, materials, shaders, and scene assets.

### Shader Graph And Node Materials

`ShaderMaterial` is the low-level escape hatch. A future node material system should provide a safer, reusable shader composition layer.

Potential crate: `scenix-nodes`.

Planned capabilities:

- typed shader nodes for constants, uniforms, attributes, varyings, textures, math, color space, lighting, fog, tone mapping, and post-processing;
- WGSL backend first, with a constrained WebGL/GLSL-compatible subset for browser fallback;
- reusable node materials for PBR, physical, toon, matcap, sprite, points, line, SSS, volume, and post effects;
- compile-time validation where possible and runtime diagnostics where platform limits vary;
- stable serialization for editor-generated materials.

### Asset Pipeline

`scenix-loader` should grow from common CPU importers into a full asset pipeline while keeping GPU upload explicit.

Future loader targets:

- glTF 2.0 extensions: skins, animation clips, morph target import, KHR_lights_punctual, KTX2/BasisU, meshopt, Draco, clearcoat, transmission, volume, sheen, specular, IOR, iridescence, emissive strength, texture transforms, and variants;
- additional loaders: FBX, Collada, PLY, SVG, USD/USDZ, 3MF, VOX, VTK, Rhino 3DM, LDraw, TTF/font, IES, DDS, TGA, TIFF, EXR, UltraHDR, LUT 3DL/CUBE/image;
- exporters: glTF/GLB, OBJ, STL, PLY, USDZ, KTX2, EXR/HDR, and scene JSON;
- asset manager with dependency graphs, async loading, progress callbacks, cancellation, reload, memory budget tracking, and optional hot reload on desktop;
- compression utilities for geometry and textures, with desktop/mobile/web capability checks.

### Animation System

Animato remains the value animation engine. Scenix still needs an asset-animation layer for imported clips and runtime blending.

Future capabilities:

- `AnimationClip`, `AnimationAction`, and `AnimationMixer` equivalents for loaded glTF/FBX-style clips;
- property binding for node transforms, visibility, morph weights, material fields, cameras, lights, and skeleton bones;
- crossfade, additive blending, time scaling, loop modes, events, markers, and deterministic sampling;
- skeletal animation import, skinning data model, GPU skinning path, retargeting helpers, inverse kinematics helpers, and skeleton utilities;
- animation path helpers and visual debugging.

### Interaction, Controls, And Editor Primitives

Scenix should support viewers, editors, CAD-like tools, games, and data visualization without making those applications implement every interaction primitive from scratch.

Future controls and helpers:

- Arcball, Trackball, Map, FirstPerson, PointerLock, Drag, and Transform controls;
- translation, rotation, scale, bounds, camera, light, and skeleton gizmos;
- selection box/frustum picking, marquee selection, hover/active/selected states, drag planes, snapping, grid constraints, and layer masks;
- HTML/CSS overlay integration on web and egui/Iced overlay integration on native;
- inspector data model for scene graph, materials, lights, cameras, animations, renderer stats, and GPU resources.

### Scene Features And Geometry Extras

The current mesh crate covers the common primitive set. Future extras should be optional and focused.

Future candidates:

- polyhedron, tetrahedron, octahedron, dodecahedron, rounded box, text, decal, convex, parametric, NURBS curve/surface/volume, edges, and wireframe geometry;
- geometry modifiers: simplify, tessellate, edge split, curve flow, mesh surface sampling, convex hull, OBB, octree, and mesh utilities;
- scene objects: sky, grounded skybox, water, reflector, refractor, lens flare, marching cubes, shadow catcher, impostors, billboards, decals, and volume slices;
- particle systems: CPU particles first, then GPU compute particles where supported;
- terrain: heightmap terrain, chunked LOD, splat maps, and streaming.

### Post-Processing And Effects

`scenix-post` already has a useful base stack. Future work should evolve it into an EffectComposer-style system while preserving direct renderer integration.

Future passes:

- SSR, SSGI, GTAO, SAO, LUT, film grain, vignette, chromatic aberration, glitch, halftone, pixelation, afterimage, transition, mask, color correction, brightness/contrast, hue/saturation, sepia, RGB shift, sharpen, denoise, bilateral blur, god rays, lens flare, and retro/stereo/anaglyph passes;
- object ID, normal, depth, motion vector, and material ID buffers for advanced effects;
- post node graph integration through the future node material system.

### Validation, Visual Tests, And Compatibility Matrix

Renderer features need stronger proof than normal unit tests. Future Scenix releases should define capability support and visual expectations before claiming parity.

Required future capabilities:

- golden image tests for reference scenes, material balls, shadow scenes, post-processing scenes, animation frames, and glTF sample models;
- pixel-diff and perceptual-diff tolerances documented per backend because Vulkan, Metal, DX12, WebGPU, and WebGL can differ slightly;
- headless render smoke tests for renderer, post, shadows, textures, environment maps, transparent sorting, and readback;
- WebGPU and WebGL browser smoke scenes with a documented fallback path when a feature is unavailable;
- desktop compatibility matrix for Linux/Vulkan, Windows/DX12 or Vulkan, and macOS/Metal;
- mobile compatibility matrix for Android/Vulkan and iOS/Metal, including touch input, DPI, surface lifecycle, and compressed texture support;
- feature support table showing `Full`, `Partial`, `Fallback`, or `Unsupported` for desktop, mobile, WebGPU, and WebGL;
- conformance scenes for PBR, physical material features, glTF extensions, skinning, morph targets, animation clips, post effects, and picking;
- benchmark gates for large scenes, texture upload, asset loading, BVH builds, animation sampling, and frame render time;
- CI lanes for CPU/no_std, all-features, wasm compile, renderer smoke, docs, examples, and benchmark compile checks.

### XR, Audio, Physics, And Editor

These should remain optional modules so applications can stay small when they do not need them.

| Future Crate | Scope |
|--------------|-------|
| `scenix-nodes` | Typed shader graph, node materials, serialized material graphs, WGSL generation, and WebGL-compatible subset validation. |
| `scenix-particles` | CPU particles, GPU particles where supported, sprite batching, emitter modules, flipbooks, and particle examples. |
| `scenix-terrain` | Heightmaps, chunked LOD, splat maps, terrain collision data, streaming, and terrain examples. |
| `scenix-sky` | Procedural sky, atmosphere scattering, sun/sky lighting helpers, environment map generation, and grounded skybox support. |
| `scenix-water` | Water surfaces, reflection/refraction helpers, waves, foam, fresnel behavior, and underwater helpers. |
| `scenix-xr` | WebXR and OpenXR integration, VR/AR sessions, controller models, hand tracking, hit testing, anchors, planes, depth sensing, estimated lighting, and mobile XR lifecycle. |
| `scenix-audio` | Audio listener, spatial audio source, streaming audio, analyser data, and scene-node attachment. |
| `scenix-physics` | Rapier/Jolt bridge, rigid bodies, colliders, character controller helpers, debug visualization, and scene transform synchronization. |
| `scenix-editor` | Visual editor shell, scene inspector, asset browser, gizmos, material editor, animation timeline, and import/export workflow. |
| `scenix-ui` or `scenix-egui` | Cross-platform debug UI overlays for desktop, mobile, and web. |

### Platform Requirements For Future Features

Every new feature should declare where it works.

| Feature Class | Desktop | Mobile | Web |
|---------------|---------|--------|-----|
| CPU scene/data APIs | Required | Required | Required |
| `wgpu` rendering | Required | Required | WebGPU where available |
| WebGL fallback | Not required | WebView optional | Required for broad browser demos |
| XR | OpenXR where available | OpenXR/ARKit/ARCore where bridged | WebXR where available |
| Compute-heavy effects | Vulkan/Metal/DX12 where supported | Capability-gated | WebGPU only; WebGL fallback disabled or approximated |
| Compressed textures | BC on desktop, ASTC where supported | ASTC/ETC2 priority | BasisU/KTX2 transcoding priority |
| Editor tooling | Full support | Viewer/light editor support | Browser editor support through WASM |

---

*Document version: 1.1.0 + future parity plan — updated June 3, 2026*
*Project: Aarambh Dev Hub — github.com/AarambhDevHub/scenix*
*Companion library: animato — github.com/AarambhDevHub/animato*
*Total crates: 17 shipped through v1.1.0; future optional crates are listed in Section 17*
