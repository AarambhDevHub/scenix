# Post-Processing

## Purpose

Apply full-screen bloom, SSAO, tonemap, anti-aliasing, fog, outline, depth of field, and motion blur effects.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Enable `renderer` and `post`.

## Key Rules

- Post effects are GPU passes.
- Renderer integration keeps the public render signature stable.
- Use only effects that fit the target platform budget.


## Example

```rust
use scenix::{PostStack, ToneMapper};

let stack = PostStack::new().with_tonemap(ToneMapper::Aces);
# let _ = stack;
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
