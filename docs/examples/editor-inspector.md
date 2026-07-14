# Editor Inspector

Source: `examples/editor_inspector.rs`

This example snapshots a scene through `Inspectable` and renders the shared
typed model with the optional egui adapter. Scenix does not own the egui context
or event loop.

```sh
cargo run -p scenix --example editor_inspector --features egui
```
