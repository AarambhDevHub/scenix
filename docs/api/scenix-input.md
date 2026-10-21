# `scenix-input`

## Role

Platform-neutral keyboard, pointer, touch, gesture, gamepad, pointer-lock, XR
controller vocabulary, and viewport state.

## Dependency Weight

Lightweight `no_std`; useful with camera controllers, WASM input mapping, and
the XR bridge.

## Install

```toml
[dependencies]
scenix-input = "1"
```

## Key Public API

`InputState`, `KeyboardState`, `PointerState`, `TouchState`, `GestureState`,
`GamepadStates`, `PointerLockState`, `ViewportMetrics`, `KeyCode`,
`PointerButton`, and the v1.9.0 XR vocabulary: `XrHand`,
`XrControllerInput`, `XrTrackingStatus`, `XR_PRESS_THRESHOLD`.

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

v1.9.0 adds the XR vocabulary every XR-facing crate shares: `XrHand`
(left/right), `XrControllerInput` trigger/squeeze/button state with the
`XR_PRESS_THRESHOLD` press gate, and `XrTrackingStatus` pose quality
(`Untracked`/`Approximate`/`Tracked`). `scenix-xr` and the facade re-use
these types rather than defining their own, so controller state stays
comparable across the input and XR crates.

## Related Docs

- [Feature flags](../concepts/feature-flags.md)
- [Interaction and editor primitives](../concepts/interaction-and-editor.md)
- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
