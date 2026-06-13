# `scenix-animato`

## Role

Optional Animato 1.5.0 bridge for scene, camera, material, and skeleton animation.

## Dependency Weight

Optional `std` path; enable `animato` on facade.

## Install

```toml
[dependencies]
scenix-animato = "1"
```

## Key Public API

AnimVec3, AnimQuat, Vec3Track, QuatTrack, NodeAnimator, CameraAnimator, MaterialAnimator, SkeletonPose, ScenixAnimationDriver

## Common Use

```rust
use scenix::{NodeAnimationTarget, NodeAnimator, ScenixAnimationDriver, Vec3, Vec3Track};

let mut driver = ScenixAnimationDriver::new();
driver.add_node(NodeAnimator::new(
    scenix::NodeId::new(1),
    NodeAnimationTarget::Translation(Vec3Track::tween(
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
        0.5,
    )),
));
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
