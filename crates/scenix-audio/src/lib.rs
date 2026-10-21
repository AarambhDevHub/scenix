#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side audio systems for scenix.
//!
//! This crate is the `v1.9.0` audio bridge: a spatialized listener, 3D
//! sources with playback state machines, deterministic distance
//! attenuation and equal-power panning with directional cones, Doppler
//! frequency shifts, streaming-buffer bookkeeping, and PCM analysis for
//! reactive visuals. It is **GPU- and driver-free** — actual audio
//! backends (WebAudio `AudioContext` on wasm, OpenAL/CoreAudio natively)
//! consume these per-frame voice decisions.
//!
//! # Quick start
//!
//! ```rust
//! use scenix_audio::{AudioListener, AudioSource, PlaybackState, compute_spatial_gain};
//! use scenix_math::Vec3;
//!
//! let listener = AudioListener::at(Vec3::new(0.0, 1.6, 0.0));
//! let mut source = AudioSource::new(7, PlaybackState::Playing)
//!     .at_position(Vec3::new(2.0, 1.6, 0.0));
//! let decision = compute_spatial_gain(&listener, &source);
//! assert!((decision.left + decision.right).is_finite());
//!
//! source.stop();
//! assert_eq!(source.state, PlaybackState::Stopped);
//! ```
//!
//! # House rules
//!
//! * Simulation is deterministic CPU state; every gain decision is a pure
//!   function of `(listener, source)` per frame.
//! * Validation is total: sources never report `Playing` after `stop()`,
//!   and streaming buffers count underruns instead of lying about latency.

extern crate alloc;

mod analyser;
mod distance;
mod doppler;
mod listener;
mod source;
mod spatial;
mod stream;

pub use analyser::{AnalyserBand, AnalyserFrame, AudioAnalyser};
pub use distance::{AttenuationModel, DistanceGain};
pub use doppler::{DopplerModel, DopplerShift, SPEED_OF_SOUND};
pub use listener::{AUDIO_LISTENER_MAX_SOURCES, AudioListener};
pub use source::{AudioChannel, AudioLoop, AudioSource, PlaybackState};
pub use spatial::{
    SpatialDecision, SpatialMix, SpeakerLayout, blend_pan, compute_spatial_decision,
    compute_spatial_gain,
};
pub use stream::{
    AudioStreamPlan, STREAM_MAX_CHUNKS, STREAM_UNDERRUN_CAPACITY, StreamFormat, StreamState,
};

/// Crate version reported for audio tooling.
pub const AUDIO_VERSION: &str = "1.9.0";
