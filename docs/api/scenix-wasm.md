# `scenix-wasm`

## Role

Optional browser canvas wrapper, DOM input mapping, generated scene setup, and demo state getters.

## Dependency Weight

Browser-oriented optional path; enable `wasm` on facade.

## Install

```toml
[dependencies]
scenix-wasm = "1"
```

## Key Public API

WebRenderer, set_panic_hook, key_code_from_dom, pointer_button_from_dom, canvas_size

## Common Use

```sh
cargo check -p scenix-wasm --target wasm32-unknown-unknown --all-features
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
