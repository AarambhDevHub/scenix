# Supported Platforms

## Native

CPU crates support stable Rust. Renderer paths use `wgpu` and depend on the host graphics backend.

## Browser

WASM builds target `wasm32-unknown-unknown`. WebGPU support depends on the browser; UI should provide a fallback when unavailable.

## no_std

Supported by CPU crates listed in `no_std.md`: math, core, input, scene, camera, mesh, material, light, texture, raycaster, helpers, and animato no-default checks.

## CI Commands

```sh
cargo test --workspace --all-features
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
```
