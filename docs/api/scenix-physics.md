# `scenix-physics`

## Role

Deterministic CPU source of truth for rigid-body simulation: a scenix-native
penalty-based solver (no engine dependency), collider groups and sensors,
spatial queries, sleeping, and the kinematic character controller (v1.9.0).

## Dependency Weight

Lightweight `no_std` on `scenix-core`, `scenix-math`, and `scenix-mesh`.

## Install

```toml
[dependencies]
scenix-physics = "1"
```

## Key Public API

`PhysicsWorld`, `PhysicsBody`, `RigidBodyKind`, `Collider`, `ColliderShape`,
`ColliderGroup`, `Contact`, `ContactEvent`, `WorldEvent`, `SpatialQueryHit`,
`CharacterState`, `CharacterInput`, `CharacterStepResult`, `GroundContact`,
`PhysicsTransformSync`, `SyncQueue`, `SyncEntry`, `debug_wireframe`,
`debug_wireframe_world`, `wireframe_box`, `wireframe_capsule`,
`wireframe_sphere`, `WireframeSeg`, `GRAVITY_DEFAULT`, `MAX_SUBSTEPS`,
`MAX_SUBSTEP_DT`, `CONTACT_SLOP`, `CONTACT_RESOLVE_PERCENT`,
`REST_CONTACT_VELOCITY`, `SLEEP_FRAME_THRESHOLD`,
`SLEEP_MOTION_THRESHOLD`, `PHYSICS_GROUND_REATTACH_SPEED`,
`PHYSICS_DEFAULT_STEP_OFFSET`, `PHYSICS_MAX_CLIMBABLE_SLOPE`

## Common Use

```rust
use scenix_math::{Transform, Vec3};
use scenix_physics::{ColliderShape, PhysicsBody, PhysicsWorld, RigidBodyKind};

let mut world = PhysicsWorld::earth();
let _floor = world.add_solid(
    PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
    ColliderShape::boxy(8.0, 0.5, 8.0),
);
let overlapping = world.overlap_sphere(Vec3::new(0.0, 2.0, 0.0), 0.5);
let _ = overlapping;
```

## Notes

v1.9.0 adds this crate. Boxes stay world-axis-aligned so the analytic narrow
phase stays branch-light; contacts resolve restitution plus Coulomb friction
with a Baumgarte positional push (`CONTACT_RESOLVE_PERCENT` 0.2 gain over a
`CONTACT_SLOP` of 5 mm). Substeps realign long frames up to `MAX_SUBSTEPS`
at `≤ MAX_SUBSTEP_DT`, sleeping is frame-threshold based with wake events,
and `Contact.a` is always the lower body id with the normal pointing a→b.
`scenix-helpers`' `physics` feature consumes `debug_wireframe` for the
collider wireframe pass.

## Related Docs

- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
