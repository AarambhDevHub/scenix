# no_std

## Purpose

Use CPU crates without standard-library dependencies.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Disable default features on supported CPU crates; use `libm` for math when needed.

## Key Rules

- Renderer, loader, post, and WASM are not `no_std` targets.
- Keep alloc usage explicit in no-default builds.
- Run no-default checks in CI for CPU crates.


## Example

```toml
scenix-math = { version = "1", default-features = false, features = ["libm"] }
scenix-core = { version = "1", default-features = false }
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
