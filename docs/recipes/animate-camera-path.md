# Animate Camera Path

## Use When

You need this behavior inside an app and want the smallest Scenix subsystem set that supports it.

## Approach

Use `CameraAnimator` or camera store traits from the Animato bridge for deterministic camera moves.

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

## Verify

Add a focused test around the state change or command shown above. For browser or GPU paths, keep tests gated so normal CPU CI remains fast.
