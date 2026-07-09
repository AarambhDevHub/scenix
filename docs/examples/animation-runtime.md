# Animation Runtime

## Purpose

Plays, loops, and crossfades clip-based animation through the `AnimationMixer`,
the clip-based counterpart to the procedural `ScenixAnimationDriver`.

## Source

`examples/animation_runtime.rs`

## Relevant Feature Flags

`animato`, `scene`

## Run Or Check

```sh
cargo run -p scenix --example animation_runtime --features animato,scene
```

## What To Look For

- Clip playback advances node translation along keyframed positions.
- Loop mode repeats the clip seamlessly each iteration.
- A second clip can be crossfaded in by adding another action and calling
  `fade_to` on both actions.

## Related Docs

- [Examples index](README.md)
- [Release v1.4.0](../release-v1.4.0.md)
- [Feature flags](../concepts/feature-flags.md)
