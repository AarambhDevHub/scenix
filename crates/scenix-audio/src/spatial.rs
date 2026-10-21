//! Combines listener orientation, attenuation, panning, cones, and
//! Doppler into one voice decision per frame.

use scenix_math::Vec3;

use crate::distance::{AttenuationModel, DistanceGain};
use crate::doppler::DopplerModel;
use crate::listener::AudioListener;
use crate::source::{AudioChannel, AudioSource, PlaybackState};

/// Cone parameters for direction-emitting sources.
///
/// Angles use the WebAudio convention, half-angles measured from the
/// emission axis.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirectionalCone {
    /// Half-angle inside which full gain applies, radians.
    pub inner_angle: f32,
    /// Half-angle outside which `outer_gain` applies, radians.
    pub outer_angle: f32,
    /// Gain applied outside the outer cone, `0..=1`.
    pub outer_gain: f32,
}

impl DirectionalCone {
    /// Cone scale from the cosine between the emission axis and the ray to
    /// the listener; full gain inside the inner cone, linear blend to
    /// `outer_gain` between cones, and `outer_gain` past the outer cone.
    #[must_use]
    pub fn gain_at(&self, cos_angle: f32) -> f32 {
        if self.outer_angle >= self.inner_angle {
            // WebAudio treats inner >= outer as an omnidirectional cone.
            if self.outer_gain >= 1.0 {
                return 1.0;
            }
        }
        let cos_inner = scenix_math::float::cos(self.inner_angle);
        let cos_outer = scenix_math::float::cos(self.outer_angle);
        if cos_angle >= cos_inner {
            1.0
        } else if cos_angle <= cos_outer {
            self.outer_gain.clamp(0.0, 1.0)
        } else {
            let span = cos_inner - cos_outer;
            if span.abs() <= 1.0e-6 {
                self.outer_gain.clamp(0.0, 1.0)
            } else {
                let t = (cos_angle - cos_inner) / span;
                (1.0 + (self.outer_gain.clamp(0.0, 1.0) - 1.0) * t).clamp(0.0, 1.0)
            }
        }
    }
}

/// Human-ish ear model layout composition styles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum SpeakerLayout {
    /// Headphones or stereo speakers: strict left/right panning.
    #[default]
    Stereo,
    /// 2.1 systems routing the center to a dedicated speaker.
    StereoCenter,
    /// Tagged 5.1-style layouts whose back channels stay unpanned.
    Surround51,
}

impl SpeakerLayout {
    /// Number of active speakers for bookkeeping.
    #[inline]
    #[must_use]
    pub const fn speaker_count(self) -> u8 {
        match self {
            Self::Stereo => 2,
            Self::StereoCenter => 3,
            Self::Surround51 => 6,
        }
    }
}

/// Final routed decision for one source this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpatialDecision {
    /// Whether the voice should produce sound right now.
    pub audible: bool,
    /// Positional panning weights, left before right.
    pub pan: [f32; 2],
    /// Per-ear distance gain.
    pub distance: DistanceGain,
    /// Doppler multiplier decided for this frame.
    pub frequency_multiplier: f32,
    /// Signed closing velocity, m/s.
    pub radial_velocity: f32,
    /// Cone scale factor applied to both ears.
    pub cone_gain: f32,
    /// Distance to the listener, meters.
    pub distance_to_listener: f32,
}

impl SpatialDecision {
    /// Route gain for the left channel after all filters `0..=1`.
    #[inline]
    #[must_use]
    pub fn left_gain(&self) -> f32 {
        (self.pan[0].max(0.0) * self.distance.far * self.cone_gain).min(1.0)
    }

    /// Route gain for the right channel after all filters `0..=1`.
    #[inline]
    #[must_use]
    pub fn right_gain(&self) -> f32 {
        (self.pan[1].max(0.0) * self.distance.near * self.cone_gain).min(1.0)
    }

    /// Flat center-channel gain for non-positional routes.
    #[inline]
    #[must_use]
    pub fn center_gain(&self) -> f32 {
        self.distance.center() * self.cone_gain
    }
}

/// Full stereo mix proposal, as the host audio engine consumes it.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpatialMix {
    /// Left gain `0..=1` after volume + pan + distance + cone.
    pub left: f32,
    /// Right gain `0..=1` after volume + pan + distance + cone.
    pub right: f32,
    /// Center/LFE gain after volume + distance + cone.
    pub center: f32,
}

impl SpatialMix {
    /// Silenced mix.
    #[inline]
    #[must_use]
    pub const fn muted() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            center: 0.0,
        }
    }
}

/// Computes the per-frame spatial decision for one source.
///
/// The function is pure: identical inputs always yield identical weights,
/// which keeps the audio graph replayable and deterministic.
#[must_use]
pub fn compute_spatial_decision(
    listener: &AudioListener,
    source: &AudioSource,
    attenuation: AttenuationModel,
    rolloff: f32,
    doppler: &DopplerModel,
) -> SpatialDecision {
    let to_source = source.position - listener.position;
    let distance = to_source.length();
    let right = listener.right();
    let direction = if distance > 1.0e-6 {
        to_source * (1.0 / distance)
    } else {
        Vec3::ZERO
    };

    // Rational-projected panning: cos between travel axis and head axes.
    let dot_right = right.dot(direction);
    let [pan_left, pan_right] = match source.channel {
        AudioChannel::Stereo => [
            (1.0 - (dot_right * 0.5 + 0.5)).max(0.15),
            (dot_right * 0.5 + 0.5).max(0.15),
        ],
        AudioChannel::Center | AudioChannel::Lfe => [0.5, 0.5],
    };

    let distance_gain = match source.channel {
        AudioChannel::Lfe => DistanceGain::flat(1.0),
        _ => DistanceGain::flat(attenuation.gain(
            distance,
            source.min_distance,
            source.max_distance,
            rolloff,
        )),
    };

    // Cone weight: sources emit along `direction`; the builtin cone is a
    // WebAudio-style 30° inner / 150° outer beam with 0.25 rear gain.
    let cone_gain = cone_weight(source.direction, to_source);

    // Doppler treats its `direction` as the wave-travel direction: the
    // vector from the source toward the listener.
    let to_listener = to_source * -1.0;
    let shift = doppler.shift(source.velocity, listener.velocity, to_listener);

    SpatialDecision {
        audible: source.state == PlaybackState::Playing && source.volume > 0.0,
        pan: [pan_left, pan_right],
        distance: distance_gain,
        frequency_multiplier: shift.frequency_multiplier,
        radial_velocity: shift.radial_velocity,
        cone_gain,
        distance_to_listener: distance,
    }
}

/// Computes the final stereo mix, folding in source volume and master gain.
#[must_use]
pub fn compute_spatial_gain(listener: &AudioListener, source: &AudioSource) -> SpatialMix {
    let decision = compute_spatial_decision(
        listener,
        source,
        AttenuationModel::InverseSquare,
        1.0,
        &DopplerModel::new(),
    );
    if !decision.audible {
        return SpatialMix::muted();
    }
    let volume = source.volume * listener.master_gain;
    match source.channel {
        AudioChannel::Lfe => SpatialMix {
            left: 0.0,
            right: 0.0,
            center: (decision.center_gain() * volume).min(1.0),
        },
        AudioChannel::Center => SpatialMix {
            left: 0.0,
            right: 0.0,
            center: (decision.center_gain() * volume).min(1.0),
        },
        AudioChannel::Stereo => SpatialMix {
            left: (decision.left_gain() * volume).min(1.0),
            right: (decision.right_gain() * volume).min(1.0),
            center: (decision.center_gain() * volume).min(1.0),
        },
    }
}

/// Cone weight for directional voices: full inside 30°, blending to 0.25
/// at the 150° rear half-angle.
#[must_use]
fn default_cone() -> DirectionalCone {
    DirectionalCone {
        inner_angle: core::f32::consts::FRAC_PI_6,
        outer_angle: core::f32::consts::FRAC_PI_6 * 5.0,
        outer_gain: 0.25,
    }
}

/// Building-block weight: cosine between the emission axis and listener ray.
#[must_use]
fn cone_weight(emission_axis: Vec3, to_listener: Vec3) -> f32 {
    if emission_axis.length_squared() <= 1.0e-12 || to_listener.length_squared() <= 1.0e-12 {
        return 1.0;
    }
    default_cone().gain_at(emission_axis.normalize().dot(to_listener.normalize()))
}

/// Lerps the two entries of a pan pair on `t`, pan pair maths only.
#[inline]
#[must_use]
pub fn blend_pan(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::AudioLoop;
    use scenix_core::{AudioListenerId, AudioSourceId};

    fn listener_at_zero() -> AudioListener {
        AudioListener::at(Vec3::ZERO)
    }

    fn playing_at(position: Vec3) -> AudioSource {
        AudioSource {
            id: AudioSourceId::new(1),
            listener: AudioListenerId::new(1),
            position,
            direction: Vec3::NEG_Z,
            velocity: Vec3::ZERO,
            volume: 1.0,
            pitch: 1.0,
            looping: AudioLoop::Once,
            pass: 0,
            channel: AudioChannel::Stereo,
            state: PlaybackState::Playing,
            min_distance: 1.0,
            max_distance: 0.0,
        }
    }

    #[test]
    fn centered_source_pans_equal() {
        let listener = listener_at_zero();
        let source = playing_at(Vec3::new(0.0, 0.0, -5.0));
        let decision = compute_spatial_decision(
            &listener,
            &source,
            AttenuationModel::InverseSquare,
            1.0,
            &DopplerModel::new(),
        );
        assert!((decision.pan[0] - decision.pan[1]).abs() < 1.0e-5);
        assert!(decision.audible);
        assert!((source.volume * 1.0 - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn right_of_listener_pans_right() {
        let listener = listener_at_zero();
        let source = playing_at(Vec3::new(5.0, 0.0, 0.0));
        let mix = compute_spatial_gain(&listener, &source);
        assert!(mix.right > mix.left, "hard right should favor right ear");
        assert!(mix.right > 0.0);
    }

    #[test]
    fn stopped_source_is_muted() {
        let listener = listener_at_zero();
        let mut source = playing_at(Vec3::new(0.0, 0.0, -4.0));
        source.stop();
        let mix = compute_spatial_gain(&listener, &source);
        assert_eq!(mix, SpatialMix::muted());
    }

    #[test]
    fn far_scaling_reduces_all_channels() {
        let listener = listener_at_zero();
        let near = compute_spatial_gain(&listener, &playing_at(Vec3::new(0.0, 0.0, -2.0)));
        let far = compute_spatial_gain(&listener, &playing_at(Vec3::new(0.0, 0.0, -400.0)));
        assert!(near.right > far.right);
        assert!(far.right > 0.0);
        assert!(far.right < 1.0);
    }

    #[test]
    fn lfe_route_bypasses_positional_pan() {
        let listener = listener_at_zero();
        let mut source = playing_at(Vec3::new(6.0, 0.0, 0.0));
        source.channel = AudioChannel::Lfe;
        let mix = compute_spatial_gain(&listener, &source);
        assert_eq!(mix.left, 0.0);
        assert_eq!(mix.right, 0.0);
        assert!(mix.center > 0.0);
        assert_eq!(SpeakerLayout::Surround51.speaker_count(), 6);
    }

    #[test]
    fn cone_softens_behind_the_source() {
        let listener = listener_at_zero();
        let mut source = playing_at(Vec3::new(0.0, 0.0, -4.0));
        source.direction = Vec3::Z; // points away from the listener
        let decision = compute_spatial_decision(
            &listener,
            &source,
            AttenuationModel::InverseSquare,
            1.0,
            &DopplerModel::new(),
        );
        assert!(decision.cone_gain < 1.0);
        assert!((decision.cone_gain - blend_pan([0.2, 0.2], [0.4, 0.4], 0.0)[0]).abs() < 0.35);
    }

    #[test]
    fn master_gain_scales_everything() {
        let listener = listener_at_zero().with_master_gain(0.5);
        let source = playing_at(Vec3::new(0.0, 0.0, -5.0));
        let mix = compute_spatial_gain(&listener, &source);
        let full = compute_spatial_gain(&listener_at_zero(), &source);
        assert!((mix.left - full.left * 0.5).abs() < 1.0e-5);
    }
}
