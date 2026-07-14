# `scenix-input`

## Role

Platform-neutral keyboard, pointer, touch, gesture, gamepad, pointer-lock, and
viewport state.

## Dependency Weight

Lightweight `no_std`; useful with camera controllers and WASM input mapping.

## Install

```toml
[dependencies]
scenix-input = "1"
```

## Key Public API

`InputState`, `KeyboardState`, `PointerState`, `TouchState`, `GestureState`,
`GamepadStates`, `PointerLockState`, `ViewportMetrics`, `KeyCode`, and
`PointerButton`.

## Common Use

```rust
use scenix_input::{InputState, PointerButton};
use scenix_math::Vec2;

let mut input = InputState::default();
input.on_pointer_down(PointerButton::Left);
input.on_pointer_move(Vec2::new(20.0, 8.0));
assert!(input.was_pointer_pressed(PointerButton::Left));
input.end_frame();
```

## Notes

Use this crate directly when you need its boundary in your own public API. Use the `scenix` facade when building an application and you want one stable import surface.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Interaction and editor primitives](../concepts/interaction-and-editor.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
