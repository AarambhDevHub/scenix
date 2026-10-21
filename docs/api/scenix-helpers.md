# `scenix-helpers`

## Role

Debug line geometry, retained transform gizmos, editor selection visuals, snap
grids, and an optional egui inspector adapter.

## Dependency Weight

Lightweight `no_std`; default facade feature.

## Install

```toml
[dependencies]
scenix-helpers = "1"
```

## Key Public API

`LineGeometry`, `GridHelper`, `AxesHelper`, `BoundingBoxHelper`, `ArrowHelper`,
`CameraHelper`, light helpers, `SkeletonHelper`, `TransformGizmoHelper`,
`GizmoGeometry`, `SelectionHelper`, `BoundsGizmoHelper`, `SnapGridHelper`,
`show_inspector` behind `egui`, and the v1.9.0 additions: `PoseAxesHelper`
(always-on), `HandSkeletonHelper` with `HAND_LINKS`/`HAND_JOINT_COUNT`, and
`ColliderDebugHelper` behind the `physics` feature.

## Common Use

```rust
use scenix_helpers::AxesHelper;
let axes = AxesHelper::new(1.0).geometry();
# let _ = axes;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

v1.9.0 adds three debug surfaces: the always-on (no `scenix-xr` dependency)
`PoseAxesHelper` draws an oriented RGB triad plus a white `-Z` forward stick
for controller and head poses, `HandSkeletonHelper` wires the 26-slot hand
profile through `HAND_LINKS` (wrist 0 into five digit chains, carpals
unlinked), and behind the new `physics` feature `ColliderDebugHelper`
wireframes world colliders with solid-cyan and sensor-orange colors via
`world_to_geometry`.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Transform gizmo example](../examples/transform-gizmo.md)
- [Editor inspector example](../examples/editor-inspector.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
