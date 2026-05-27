# `scenix-input`

## Role

Platform-neutral keyboard and pointer state.

## Dependency Weight

Lightweight `no_std`; useful with camera controllers and WASM input mapping.

## Install

```toml
[dependencies]
scenix-input = "1"
```

## Key Public API

KeyboardState, PointerState, KeyCode, PointerButton

## Common Use

```rust
use scenix_input::PointerState;
let pointer = PointerState::default();
# let _ = pointer;
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
