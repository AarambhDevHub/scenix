# `scenix-post`

## Role

Optional GPU post-processing stack and effect configs.

## Dependency Weight

Heavy `std`/`wgpu` path; use with renderer.

## Install

```toml
[dependencies]
scenix-post = "1"
```

## Key Public API

PostStack, BloomConfig, SsaoConfig, ToneMapper, FxaaConfig, TaaConfig, SmaaConfig, DofConfig, FogPostConfig, OutlineConfig, MotionBlurConfig

## Common Use

```rust
use scenix_post::{PostStack, ToneMapper};
let stack = PostStack::new().with_tonemap(ToneMapper::Aces);
# let _ = stack;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
