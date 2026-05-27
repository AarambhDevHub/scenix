# Transforms

## Purpose

Work with local transforms, world transforms, and parent-child propagation.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Default facade features include `scene` and `math`.

## Key Rules

- Local transforms describe a node relative to its parent.
- World transforms are cached by the scene graph.
- Dirty roots are deduplicated before propagation.


## Example

```rust
use scenix::{SceneGraph, Transform, Vec3};

let mut scene = SceneGraph::new();
let transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
# let _ = (&mut scene, transform);
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
