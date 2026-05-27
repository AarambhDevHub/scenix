# Benchmarking

## Goal

Compile benches in CI and run expensive GPU benches only when explicitly enabled.

## Measure First

Use focused commands and compare one change at a time. Avoid enabling heavy features globally when only one binary or example needs them.

## Command Or Pattern

```sh
cargo bench --workspace --no-run
SCENIX_RUN_GPU_BENCHES=1 cargo bench -p scenix-post
```

## Practical Checks

- Keep CPU-only crates lightweight.
- Avoid rebuilding data structures every frame unless inputs changed.
- Separate load, registration, update, and render costs when profiling.
