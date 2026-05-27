# Compile Time

## Goal

Keep default features CPU-only, avoid enabling loader/renderer/post/wasm in crates that do not need them, and prefer focused crates for libraries.

## Measure First

Use focused commands and compare one change at a time. Avoid enabling heavy features globally when only one binary or example needs them.

## Command Or Pattern

```sh
cargo check -p scenix --no-default-features
```

## Practical Checks

- Keep CPU-only crates lightweight.
- Avoid rebuilding data structures every frame unless inputs changed.
- Separate load, registration, update, and render costs when profiling.
