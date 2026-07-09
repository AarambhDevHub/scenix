//! Clip data model: a named set of keyframe channels + markers.

use alloc::string::String;
use alloc::vec::Vec;

use crate::binding::PropertyBinding;
use crate::keyframe::{KeyframeBool, KeyframeColor, KeyframeQuat, KeyframeScalar, KeyframeVec3};

/// One keyframe track variant carried by a clip channel.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClipTrack {
    /// Scalar keyframe track (morph weights, intensities, opacity, ...).
    Scalar(KeyframeScalar),
    /// 3D vector keyframe track (translations, scales, emissive, ...).
    Vec3(KeyframeVec3),
    /// Quaternion keyframe track (rotations).
    Quat(KeyframeQuat),
    /// Color keyframe track (albedo, light color, ...).
    Color(KeyframeColor),
    /// Boolean keyframe track (visibility).
    Bool(KeyframeBool),
}

impl ClipTrack {
    /// Returns the track duration (last keyframe time).
    #[inline]
    pub fn duration(&self) -> f32 {
        match self {
            Self::Scalar(t) => t.duration(),
            Self::Vec3(t) => t.duration(),
            Self::Quat(t) => t.duration(),
            Self::Color(t) => t.duration(),
            Self::Bool(t) => t.duration(),
        }
    }
}

/// A single animated channel: a binding + a keyframe track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClipChannel {
    /// Where the sampled value is written.
    pub binding: PropertyBinding,
    /// Keyframe track sampled at clip-local time.
    pub track: ClipTrack,
}

/// A named time marker inside a clip (Three.js `AnimationClip`-style).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimationMarker {
    /// Marker label.
    pub name: String,
    /// Clip-local time in seconds.
    pub time: f32,
}

impl AnimationMarker {
    /// Creates a marker.
    #[inline]
    pub fn new(name: impl Into<String>, time: f32) -> Self {
        Self {
            name: name.into(),
            time: time.max(0.0),
        }
    }
}

/// A playable animation clip.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimationClip {
    /// Human-readable clip name.
    pub name: String,
    /// Clip duration in seconds (`max(channel.track.duration())`).
    pub duration: f32,
    /// Channels in deterministic order.
    pub channels: Vec<ClipChannel>,
    /// Named time markers.
    pub markers: Vec<AnimationMarker>,
}

impl AnimationClip {
    /// Creates an empty named clip.
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            duration: 0.0,
            channels: Vec::new(),
            markers: Vec::new(),
        }
    }

    /// Builder: adds a channel and extends the clip duration if needed.
    pub fn with_channel(mut self, channel: ClipChannel) -> Self {
        self.duration = self.duration.max(channel.track.duration());
        self.channels.push(channel);
        self
    }

    /// Builder: adds a marker.
    pub fn with_marker(mut self, marker: AnimationMarker) -> Self {
        self.duration = self.duration.max(marker.time);
        self.markers.push(marker);
        self
    }

    /// Recomputes duration from channels + markers.
    pub fn recompute_duration(&mut self) {
        let mut d = 0.0_f32;
        for ch in &self.channels {
            d = d.max(ch.track.duration());
        }
        for m in &self.markers {
            d = d.max(m.time);
        }
        self.duration = d;
    }
}
