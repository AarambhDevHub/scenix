# `scenix`

## Role

Facade crate for applications that want stable v1 imports from one package.

## Dependency Weight

Default CPU authoring plus optional heavy systems.

## Install

```toml
[dependencies]
scenix = "1"
```

## Key Public API

`InputState`, `SceneGraph`, `SceneNode`, `PerspectiveCamera`, camera controllers,
`Geometry`, `PbrMaterial`, `Raycaster`, drag/transform controls, gizmos,
`Inspectable`, `Renderer`, `GltfLoader`, `PostStack`, `AnimationMixer`,
`BrowserRenderer`, and `WebRenderer`. With the `nodes` feature the facade also
re-exports `NodeMaterial`, `ShaderGraph`, the node kind vocabulary, backend
generators, and `PostNodeEffect` by name (see
[scenix-nodes](scenix-nodes.md)). With the `particles` feature the facade
re-exports `ParticleEmitter`, `ParticleSystem`, the modifier vocabulary, and
`ParticleRenderBatch` by name (see [scenix-particles](scenix-particles.md)).
With the `terrain`, `sky`, `water`, and `environment` features the facade
re-exports `Heightmap`, `TerrainGrid`, `SkyModel`, `SunPosition`,
`WaveField`, `WaterSurface`, and friends by name (see
[scenix-terrain](scenix-terrain.md), [scenix-sky](scenix-sky.md), and
[scenix-water](scenix-water.md)). With the `xr`, `audio`, and `physics`
features (or `bridges` for all three) the facade re-exports `XrSessionPlan`,
`XrControllerBay`, `XrHandFrame`, `AudioListener`, `AudioSource`,
`compute_spatial_decision`, `PhysicsWorld`, `CharacterState`,
`PhysicsTransformSync`, `ColliderDebugHelper`, and friends by name (see
[scenix-xr](scenix-xr.md), [scenix-audio](scenix-audio.md), and
[scenix-physics](scenix-physics.md)). With the `ui` feature the facade
re-exports the renderer-agnostic UI layer — `PanelStack`, `panel_placement`,
`RendererStats`, `compose_panel_overlay`, the widget vocabulary, and friends
(see [scenix-ui](scenix-ui.md)). The `editor` feature (which implies `ui`)
re-exports `EditorSession`, `EditCommand`, the gizmo vocabulary
(`GizmoMode`, `GizmoAxis`, `apply_axis_delta`, `axis_pick`),
`scene_rows`, the `InspectorEdit` surface, and the material field editors;
asset-browser rows additionally arrive under `loader` and timeline previews
under `animato` (see [scenix-editor](scenix-editor.md)).

## Common Use

```rust
use scenix::{MaterialId, MeshId, SceneGraph, SceneNode, box_geometry};

let mesh_id = MeshId::new(1);
let material_id = MaterialId::new(1);
let geometry = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);

let mut scene = SceneGraph::new();
scene.add(SceneNode::mesh("cube", mesh_id, material_id));
scene.update_world_transforms();
# let _ = geometry;
```

## Notes

v1.8.0 adds the `terrain`, `sky`, `water`, and `environment` features; curated re-exports keep the `lod`/`sky`/`sun` module names free for the scene, material, and light globs.

v1.9.0 adds the `xr`, `audio`, `physics`, and `bridges` features; the bridge re-exports are curated lists because their module namespaces (`xr`, `audio`, `physics`, `world`, `sync`) would otherwise shadow the scene/input globs.

v1.10.0 adds the `ui` feature and extends `editor`: the UI re-exports are curated because widget/panel names invite future collisions with runtime crates, and the editor surface needs `ui` anyway since sessions own the standard panel stack.

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Interaction and editor primitives](../concepts/interaction-and-editor.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
