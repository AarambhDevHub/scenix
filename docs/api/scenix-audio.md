# `scenix-audio`

## Role

Deterministic CPU source of truth for spatial audio mixing decisions:
listener/source state, attenuation and pan, Doppler, streamed buffering
health, and spectrum metering (v1.9.0).

## Dependency Weight

Lightweight `no_std` on `scenix-core` and `scenix-math`.

## Install

```toml
[dependencies]
scenix-audio = "1"
```

## Key Public API

`AudioListener`, `AudioSource`, `AudioChannel`, `AudioLoop`,
`PlaybackState`, `SpeakerLayout`, `DirectionalCone`,
`AttenuationModel`, `DistanceGain`, `DopplerModel`, `DopplerShift`,
`compute_spatial_decision`, `compute_spatial_gain`, `blend_pan`,
`SpatialDecision`, `SpatialMix`, `AudioStreamPlan`, `StreamState`,
`AudioAnalyser`, `AnalyserBand`, `AnalyserFrame`, `SPEED_OF_SOUND`,
`STREAM_MAX_CHUNKS`, `AUDIO_LISTENER_MAX_SOURCES`, `ANALYSER_MIN_FFT_SIZE`,
`ANALYSER_MAX_FFT_SIZE`

## Common Use

```rust
use scenix_audio::AudioListener;
use scenix_math::Vec3;

let listener = AudioListener::axes(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y);
let _ = listener;
```

## Notes

v1.9.0 adds this crate. The crate decides mixes and buffers; the host
applies them to whatever DSP backend it owns. One `SpatialDecision` covers
pan, distance gain, cone filtering, and the Doppler multiplier for one
source per frame, so tests replay identical mixes. The Doppler `direction`
is the wave-travel direction (source → listener) and multipliers clamp to
`0.25..=4` so resampling stays finite.

## Related Docs

- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
