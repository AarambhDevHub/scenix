# WASM Performance

## Goal

Keep browser assets small, compile only needed features, and provide clean fallback UI when WebGPU is unavailable.

## Measure First

Use focused commands and compare one change at a time. Avoid enabling heavy features globally when only one binary or example needs them.

## Command Or Pattern

```sh
trunk build --release --public-url /scenix/
```

## Practical Checks

- Keep CPU-only crates lightweight.
- Avoid rebuilding data structures every frame unless inputs changed.
- Separate load, registration, update, and render costs when profiling.
