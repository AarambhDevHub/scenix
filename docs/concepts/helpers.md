# Helpers

## Purpose

Generate debug line geometry for grids, axes, bounds, cameras, lights, arrows, and skeletons.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `helpers`.

## Key Rules

- Helpers return `LineGeometry`.
- They do not require the renderer.
- Use them for editor overlays and diagnostics.


## Example

```rust
use scenix::GridHelper;

let grid = GridHelper::new(10, 1.0).geometry();
# let _ = grid;
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
