# Audio Positions

## Purpose

Faces a listener down-range from a room corner, decides the spatial mix for one playing source, checks stream buffering health, and meters a 1 kHz sine through the analyser.

## Source

`examples/audio_positions.rs`

## Relevant Feature Flags

facade `audio` features

## Run Or Check

```sh
cargo run -p scenix --example audio_positions --features audio
```

## What To Look For

- The example should compile with the listed features.
- CPU examples should not require GPU setup.
- The mix is one deterministic `SpatialDecision` per source per frame.

## Related Docs

- [Examples index](README.md)
- [scenix-audio crate reference](../api/scenix-audio.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
