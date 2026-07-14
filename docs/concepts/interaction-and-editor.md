# Interaction And Editor Primitives

Scenix separates platform input, interaction policy, scene state, UI models,
and GPU acceleration. That keeps camera movement and selection testable without
a window, and it lets native and browser applications share behavior.

## Frame Lifecycle

Forward platform events to `InputState`, update controls and interactions once,
then call `end_frame`. Held keys and buttons survive; deltas and edge
transitions are cleared.

```rust
use scenix::{ArcballController, InputState, PointerButton, Vec2, Vec3};

let mut input = InputState::default();
input.on_pointer_down(PointerButton::Left);
input.on_pointer_move(Vec2::new(18.0, 4.0));

let mut controls = ArcballController::new(Vec3::ZERO, 4.0);
let camera_transform = controls.update_from_input(&input, 1.0 / 60.0);
input.end_frame();
# let _ = camera_transform;
```

Pointer and touch deltas represent event displacement and are not multiplied by
frame time. Continuous keyboard and gamepad movement is frame-time scaled.

## Selection And Policy

`SceneGraph` is the authority for selected, hovered, and active nodes.
`SelectionMode` makes replace/add/toggle/remove operations deterministic.
Sparse `NodeEditorMetadata` adds labels and application data only where needed;
layer masks and `LayerPolicy` gate selection, visibility, and transforms.

## Picking Paths

Use the CPU `Raycaster` for portable exact mesh hits and BVH-accelerated
marquee/frustum selection. Use `Renderer::pick` when WebGPU object ID, normal,
depth, and reconstructed world position are useful. The renderer path is
on-demand and does not replace the CPU fallback.

## Reversible Transforms

`DragController` and `TransformController` capture the starting transform.
`end` commits the operation; `cancel` restores it. Snapping is configured with
`SnapSettings`, while transform mode, coordinate space, and axis/plane
constraint remain explicit values that can be driven by any UI.

## UI Boundary

`Inspectable` returns an owned, typed `InspectorSnapshot`. A host can render it
with the optional egui adapter, serialize browser-facing snapshots as JSON, or
map it into another UI toolkit without coupling CPU crates to that toolkit.
