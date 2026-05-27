# Platform Guide

## Native

Use the default facade for CPU authoring. Add `renderer` for `wgpu` surface or headless rendering. GPU tests are gated with `SCENIX_RUN_GPU_TESTS=1`.

## no_std

The CPU crates below support no-default builds:

- `scenix-math`
- `scenix-core`
- `scenix-input`
- `scenix-scene`
- `scenix-camera`
- `scenix-mesh`
- `scenix-material`
- `scenix-light`
- `scenix-texture`
- `scenix-raycaster`
- `scenix-helpers`
- `scenix-animato`

`scenix-loader`, `scenix-renderer`, `scenix-post`, and `scenix-wasm` are `std`-oriented.

## Browser

Enable the `wasm` facade feature or depend on `scenix-wasm` directly. The wrapper exposes a generated scene, DOM input mapping helpers, canvas resize, and demo state getters.

```sh
rustup target add wasm32-unknown-unknown
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
```

## Website

The website is a standalone Leptos CSR crate:

```sh
cd website
trunk build --release --public-url /scenix/
```
