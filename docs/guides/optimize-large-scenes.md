# Optimize Large Scenes

## Goal

Reduce transform, culling, raycasting, upload, and draw overhead for large scene graphs.

## Relevant Feature Flags

Use only needed features; renderer/raycaster for relevant hot paths.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```rust
scene.update_world_transforms();
// Build or refresh BVH only after relevant scene or geometry changes.
```

## Verify

Profile transform propagation, BVH rebuilds, and renderer registration separately.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
