# Transform Gizmo

Source: `examples/transform_gizmo.rs`

This example writes a transform gizmo into reusable line and handle storage,
then performs analytic ray hit testing without building render meshes for the
handles.

```sh
cargo run -p scenix --example transform_gizmo
```

Change `TransformMode` to generate translation, rotation, or scale handles.
