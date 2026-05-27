# Scene Graph Optimization

## Goal

Batch transform edits, update world transforms once, and avoid unnecessary hierarchy churn.

## Measure First

Use focused commands and compare one change at a time. Avoid enabling heavy features globally when only one binary or example needs them.

## Command Or Pattern

```rust
scene.update_world_transforms();
```

## Practical Checks

- Keep CPU-only crates lightweight.
- Avoid rebuilding data structures every frame unless inputs changed.
- Separate load, registration, update, and render costs when profiling.
