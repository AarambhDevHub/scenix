# Create A PBR Material

## Goal

Configure metallic-roughness material data for renderer registration.

## Relevant Feature Flags

Default `material`; `renderer` for GPU preview.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```rust
use scenix::{Color, PbrMaterial};

let material = PbrMaterial::new()
    .albedo(Color::rgb(0.8, 0.6, 0.3))
    .metallic(0.1)
    .roughness(0.45);
# let _ = material;
```

## Verify

Run `cargo run -p scenix --example pbr_sphere --features renderer`.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
