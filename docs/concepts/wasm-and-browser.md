# WASM And Browser

## Purpose

Use the browser wrapper, DOM input mapping, and generated scene demo.

## When To Use This

Read this page when the subsystem affects your app architecture or dependency choices. For implementation steps, pair it with the matching guide in `../guides/`.

## Relevant Feature Flags

Enable `wasm`; renderer support requires browser WebGPU.

## Key Rules

- `scenix-wasm` wraps canvas setup and input forwarding.
- The website is Leptos CSR and builds with Trunk.
- Fallback UI should handle unavailable WebGPU cleanly.


## Example

```sh
rustup target add wasm32-unknown-unknown
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
```

## Related Docs

- [Feature flags](feature-flags.md)
- [Architecture overview](architecture-overview.md)
- [API reference](../api/facade-crate.md)
