# Use Orbit Camera

## Goal

Attach orbit camera controls to pointer input for product viewers and editors.

## Relevant Feature Flags

Default `camera` and `input` features.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```rust
use scenix::{OrbitController, PointerState, Vec3};

let mut controller = OrbitController::new(Vec3::ZERO, 4.0);
let pointer = PointerState::default();
controller.update(&pointer, 1.0 / 60.0);
```

## Verify

Run `cargo run -p scenix --example orbit_camera`.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
