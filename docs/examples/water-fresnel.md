# Water Fresnel

## Purpose

Displaces an ocean-shore wave surface, samples heights against wave crests, prints the fresnel reflectance ramp, foam gate, underwater murk drift, and mirror-camera math.

## Source

`examples/water_fresnel.rs`

## Relevant Feature Flags

facade `water` (and default `camera`) features

## Run Or Check

```sh
cargo run -p scenix --example water_fresnel --features water,camera
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- Renderer examples may need a working native graphics backend or headless support.

## Related Docs

- [Examples index](README.md)
- [Scenix v1.8.0 release](../release-v1.8.0.md)
