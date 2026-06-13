# Build For WASM

## Goal

Compile the browser wrapper and WASM viewer example.

## Relevant Feature Flags

`wasm`; `BrowserRenderer` uses WebGPU when safe, WebGL2 as the full browser fallback when WebGPU is unavailable, and WebGL1 only as a reduced last-resort fallback.

## Steps

1. Add the required Cargo features.
2. Keep CPU scene data in caller-owned stores.
3. Call `update_world_transforms()` after transform or hierarchy edits.
4. Register resources with optional systems only when those systems are enabled.

## Example

```sh
rustup target add wasm32-unknown-unknown
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
cargo check --manifest-path examples/wasm_viewer/Cargo.toml --target wasm32-unknown-unknown
```

## Verify

Use the website fallback path for browsers without WebGPU or WebGL. In normal browsers without WebGPU, `scenix-wasm` should still render through WebGL2; WebGL1 is retained for older browsers with reduced parity.

## Related Docs

- [Quick start](../quick-start.md)
- [Feature flags](../concepts/feature-flags.md)
