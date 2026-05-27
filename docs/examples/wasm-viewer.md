# WASM Viewer

## Purpose

Compiles the standalone browser viewer crate.

## Source

`examples/wasm_viewer`

## Relevant Feature Flags

wasm target

## Run Or Check

```sh
cargo check --manifest-path examples/wasm_viewer/Cargo.toml --target wasm32-unknown-unknown
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Feature flags](../concepts/feature-flags.md)
