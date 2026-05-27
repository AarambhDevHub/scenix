# Add Lights And Shadows

## Goal

Register light data and enable shadow-capable renderer examples.

## Relevant Feature Flags

`light` by default; `renderer` for shadows.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```rust
use scenix::{DirectionalLight, Vec3};

let sun = DirectionalLight::new(Vec3::new(-1.0, -2.0, -1.0));
# let _ = sun;
```

## Verify

Run `cargo run -p scenix --example shadow_demo --features renderer`.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
