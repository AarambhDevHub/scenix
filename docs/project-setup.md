# Project Setup

Use this page when starting a new Scenix application or example. The recommended shape keeps CPU scene data separate from GPU registration and makes optional systems obvious.

## Recommended Layout

```text
my-app/
  Cargo.toml
  src/
    main.rs
    scene.rs       # SceneGraph, transforms, IDs
    assets.rs      # Geometry/material/texture stores
    render.rs      # Renderer setup and GPU registration
    input.rs       # Pointer/keyboard state and camera controls
```

## Cargo Features

```toml
[dependencies]
scenix = { version = "1", features = ["renderer", "post"] }
```

Use `loader` only when you decode assets at runtime. Use `animato` only when the app needs animation tracks.

## Development Commands

```sh
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
```

## Runtime Pattern

Create scene resources on the CPU, call `scene.update_world_transforms()` after transform edits, register changed resources with the renderer, then render using the active camera. This keeps scene authoring deterministic and GPU ownership explicit.
