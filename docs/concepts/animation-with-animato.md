# Animation With Animato

## Purpose

Drive scene, camera, material, and skeleton values through the optional Animato bridge.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Enable `animato`; it uses `animato = "1.4.0"`.

## Key Rules

- The driver ticks deterministic animator lists.
- Scene animators target transforms and visibility.
- Material and camera animators use store traits.


## Example

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

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
