# Renderer Picking

Source: `examples/renderer_picking.rs`

This example creates a headless renderer, uploads one cube, and requests one
editor pixel. The returned value includes the optional node ID, depth, decoded
normal, and reconstructed world position.

```sh
cargo run -p scenix --example renderer_picking --features renderer
```

The first request allocates editor targets; later requests reuse them. The
example exits cleanly when no headless adapter is available.
