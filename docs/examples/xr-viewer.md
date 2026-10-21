# XR Viewer

## Purpose

Validates an immersive VR session plan, walks the runtime lifecycle to `Active`, advances two frame ticks, and builds the stereo camera rig the renderer would stream.

## Source

`examples/xr_viewer.rs`

## Relevant Feature Flags

facade `xr` (and `camera`) features

## Run Or Check

```sh
cargo run -p scenix --example xr_viewer --features xr,camera
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- No WebXR runtime is needed: the plan is the deterministic CPU agreement.

## Related Docs

- [Examples index](README.md)
- [scenix-xr crate reference](../api/scenix-xr.md)
- [Camera stereo surface](../api/scenix-camera.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
