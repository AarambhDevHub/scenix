# Controls Showcase

Source: `examples/controls_showcase.rs`

This example builds a DPR-aware `InputState`, drives an arcball with pointer and
scroll input, advances the frame boundary, then drives first-person movement
with keyboard and gamepad state.

```sh
cargo run -p scenix --example controls_showcase
```

The same input snapshot can drive Orbit, Fly, Arcball, Trackball, Map,
FirstPerson, and PointerLock controllers.
