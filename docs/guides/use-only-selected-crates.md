# Use Only Selected Crates

## Goal

Depend on focused crates instead of the facade for libraries and small tools.

## Relevant Feature Flags

Depends on selected crate set.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```toml
[dependencies]
scenix-math = "1"
scenix-core = "1"
scenix-scene = "1"
scenix-camera = "1"
```

## Verify

Keep renderer, loader, post, Animato, and WASM out of libraries unless they are part of the public contract.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
