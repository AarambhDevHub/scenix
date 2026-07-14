# Scenix v1.5.0 — Controls, Interaction, And Editor Primitives

Version 1.5 turns the existing scene, camera, raycaster, renderer, and browser
layers into a coherent interaction toolkit. Applications can keep platform
events at their boundary, feed one `InputState`, and share the same controls and
scene selection model across native and browser front ends.

## Data Flow

```text
native/DOM events
        |
        v
   InputState --------> camera controls
        |                    |
        v                    v
 ray/frustum pick ------> SceneGraph selection
        |                    |
        v                    v
 drag/transform <------ gizmo handle
        |
        +----> InspectorSnapshot ----> egui / JSON / custom UI
        |
        +----> optional Renderer ID + normal + depth readback
```

CPU selection and transform operations work without a renderer. GPU picking is
an optional precision path that allocates its targets and staging buffers only
on the first editor request and reuses them afterward.

## New Public Surfaces

- `scenix-input`: `InputState`, `TouchState`, `GestureRecognizer`,
  `GamepadStates`, `PointerLockState`, and `ViewportMetrics`.
- `scenix-camera`: `ArcballController`, `TrackballController`, `MapController`,
  `FirstPersonController`, and `PointerLockController`.
- `scenix-scene`: selection state/modes, editor metadata and policies,
  `LayerMask`, `TransformMode`, `TransformSpace`, `TransformConstraint`, and
  `SnapSettings`.
- `scenix-raycaster`: `SelectionRect`, `SelectionFrustum`, `DragPlane`,
  `DragController`, and `TransformController`.
- `scenix-helpers`: retained transform gizmos, analytic handles, selection and
  bounds helpers, snap grids, and `show_inspector` behind `egui`.
- `scenix-core`: the `Inspectable` trait and typed inspector snapshot tree.
- `scenix-renderer`: `EditorPickRequest`, `EditorPickResult`,
  `EditorBufferStats`, and explicit render/read/pick methods.
- `scenix-wasm`: DPR-aware metrics plus touch, pointer-lock, gamepad, transform
  mode, and inspector JSON forwarding.

## Choosing A Feature

Use `interaction` for controls, CPU picking, dragging, selection, and gizmos.
Use `editor` when a UI needs the typed inspector model. Add `egui` only when the
host UI uses egui. Enable `renderer` independently for rendering and on-demand
GPU picking; enable `wasm` for browser bindings.

```toml
[dependencies]
scenix = { version = "1.5", features = ["editor", "egui", "renderer"] }
```

## Performance Design

- Touch and gamepad storage is fixed-capacity.
- Transient input is cleared in place by `InputState::end_frame`.
- Controls consume borrowed snapshots and do not allocate per update.
- BVH and raycaster APIs can write into caller-owned output buffers.
- Gizmo and helper geometry can be regenerated into retained buffers.
- Scene editor metadata is sparse, so ordinary runtime nodes pay no metadata
  allocation cost.
- GPU picking uses dense temporary object IDs, a one-pixel scissor for `pick`,
  capacity-grown uniforms, and a persistent readback buffer.

## Migration

No v1.4 interaction API was removed. Existing `KeyboardState`, `PointerState`,
Orbit/Fly controllers, scene traversal, and `Raycaster::cast_ray` calls continue
to work. Applications can migrate incrementally by first aggregating platform
events into `InputState`, then adopting selection or transform primitives.

## Validation

The release is covered by unit and integration tests, Rust 1.89 and stable
checks, all facade feature lanes, no-default CPU builds, all examples,
`wasm32-unknown-unknown`, the standalone browser viewer, the Leptos website,
rustdoc warnings-as-errors, packaging, and Vulkan/lavapipe GPU-picking tests.

Related reading:

- [Interaction and editor concepts](concepts/interaction-and-editor.md)
- [Examples](examples/README.md)
- [GitHub release notes](../.github/release-notes/1.5.0.md)
