# Sky And Sun

## Purpose

Evaluates procedural sky profiles at the zenith and horizon, bakes a deterministic equirectangular map with ambient and light-probe bakes, and drives a warm sunset light.

## Source

`examples/sky_sun.rs`

## Relevant Feature Flags

facade `sky` feature

## Run Or Check

```sh
cargo run -p scenix --example sky_sun --features sky
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Scenix v1.8.0 release](../release-v1.8.0.md)
