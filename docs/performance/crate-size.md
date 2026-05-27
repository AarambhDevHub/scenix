# Crate Size

## Goal

Select focused crates and disable default features for `no_std` or embedded-adjacent CPU tools.

## Measure First

Use focused commands and compare one change at a time. Avoid enabling heavy features globally when only one binary or example needs them.

## Command Or Pattern

```toml
scenix-scene = { version = "1", default-features = false }
```

## Practical Checks

- Keep CPU-only crates lightweight.
- Avoid rebuilding data structures every frame unless inputs changed.
- Separate load, registration, update, and render costs when profiling.
