# Scenix v1.0.0 Stable

Scenix `1.0.0` is the first stable release of the modular Rust-native 3D scene workspace.

## Highlights

- Stable facade and crate versions: `scenix = "1"`.
- CPU authoring, raycasting, and helper geometry remain default facade features.
- Loader, renderer, post-processing, Animato, and WASM support stay optional.
- Renderer now uses camera view-projection matrices, world matrices, reusable uniform buffers, and cached material preview paths.
- Built-in renderer material registration covers PBR, Physical, Unlit, Lambert, Toon, Wireframe, and Normal preview materials.
- Static Leptos CSR website and demo deploy to GitHub Pages at `/scenix/`.
- Animato bridge stays on `animato = "1.4.0"`.

## Install

```toml
[dependencies]
scenix = "1"
```

Optional full stack:

```toml
[dependencies]
scenix = { version = "1", features = ["loader", "renderer", "post", "animato", "wasm"] }
```

## What Changed From 0.9.0

- Bumped all workspace crates from `0.9.0` to `1.0.0`.
- Added a stable renderer frame path using camera/world uniforms instead of the previous clip-space smoke path.
- Expanded renderer material registration to include advanced preview/debug material types.
- Added the v1 website, docs folder, release notes, GitHub Pages workflow, coverage workflow step, and package checks.
- Added the remaining architecture examples and registered them with the facade crate.
- Expanded the WASM wrapper with demo controls, state getters, generated scene content, and clean browser fallback support.

## Stable API Note

The v1 API is intended to be additive. Public APIs that become obsolete should be deprecated before removal.

## Links

- Website and demo: `https://aarambhdevhub.github.io/scenix/`
- Documentation: `https://docs.rs/scenix`
- Crates: `https://crates.io/crates/scenix`

## Code Example

```rust
use std::collections::BTreeMap;
use scenix::{
    MaterialId, MeshId, NodeAnimationTarget, NodeAnimator, PbrMaterial, SceneGraph, SceneNode,
    ScenixAnimationDriver, Vec3, Vec3Track, box_geometry,
};

# fn run() -> Result<(), scenix::ValidationError> {
let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);
let _cube = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);

let mut scene = SceneGraph::new();
let cube_node = scene.add(SceneNode::mesh("cube", mesh_id, material_id));

let mut driver = ScenixAnimationDriver::new();
driver.add_node(NodeAnimator::new(
    cube_node,
    NodeAnimationTarget::Translation(Vec3Track::tween(Vec3::ZERO, Vec3::X, 0.5)),
));

let mut cameras = scenix::CameraStores {
    perspective: &mut BTreeMap::new(),
    orthographic: &mut BTreeMap::new(),
};
let mut materials = BTreeMap::from([(material_id, PbrMaterial::new())]);
let mut skeletons = Vec::new();

driver.tick(0.5, &mut scene, &mut cameras, &mut materials, &mut skeletons)?;
scene.update_world_transforms();
# Ok(())
# }
```

## Migration Notes

- Replace `0.9` dependency requirements with `1`.
- Keep explicit feature flags for loader/GPU/post/Animato/WASM paths.
- Renderer users should register material variants through the matching stable `register_*_material` method.
- Browser users can read demo state through the new `WebRenderer` getters instead of maintaining duplicate UI state.

## Known Limitations

- Advanced physical material fields render through a stable preview path in v1.
- Loader APIs decode CPU assets but do not automatically upload them to the GPU.
- The website demo avoids large external model assets and uses generated geometry.
- GPU tests depend on a working Vulkan backend or Mesa lavapipe.
