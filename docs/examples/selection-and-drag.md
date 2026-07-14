# Selection And Drag

Source: `examples/selection_and_drag.rs`

This example builds a scene BVH, selects nodes through a perspective marquee
frustum, updates the scene selection model, and performs a snapped drag on a
camera-facing plane.

```sh
cargo run -p scenix --example selection_and_drag
```

Drag sessions can be committed with `end` or reverted with `cancel`.
