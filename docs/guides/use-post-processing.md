# Use Post-Processing

## Goal

Attach a post stack to renderer output for bloom, SSAO, tonemap, and anti-aliasing.

## Relevant Feature Flags

`renderer`, `post`

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```rust
use scenix::{FxaaConfig, PostStack, ToneMapper};

let stack = PostStack::new()
    .with_tonemap(ToneMapper::Aces)
    .with_fxaa(FxaaConfig::default());
# let _ = stack;
```

## Verify

Run `cargo run -p scenix --example post_processing --features "renderer post"`.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
