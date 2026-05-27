# Error Handling

## Purpose

Handle validation, loader, renderer, and GPU failures consistently.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

No special feature is needed for core errors; subsystem errors appear with their crates.

## Key Rules

- Constructors validate dimensions, byte sizes, and IDs where possible.
- Loader errors distinguish unsupported assets from IO/decode failures.
- Renderer errors should be surfaced to UI or test output.


## Example

```rust
use scenix::ScenixError;

fn report(error: ScenixError) {
    eprintln!("scenix error: {error}");
}
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
