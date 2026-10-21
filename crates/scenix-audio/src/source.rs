//! Positional audio sources and their playback state machines.

use scenix_math::Vec3;

/// Playback lifecycle of a positional audio source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum PlaybackState {
    /// Not sounding and no buffered play request.
    #[default]
    Stopped,
    /// Audible, consuming stream chunks or PCM buffers.
    Playing,
    /// Frozen midway; resumes on `play()`.
    Paused,
}

impl PlaybackState {
    /// Returns whether an audible signal exists right now.
    #[inline]
    #[must_use]
    pub const fn is_audible(self) -> bool {
        matches!(self, Self::Playing)
    }

    /// Returns a short human-readable label used by inspector tooling.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Playing => "playing",
            Self::Paused => "paused",
        }
    }
}

/// Which speaker output a stream renders into.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum AudioChannel {
    /// Left/right stereo pair (the default scene layout).
    #[default]
    Stereo,
    /// Center speaker, ignoring positional pan.
    Center,
    /// Low-frequency effects channel; subwoofer driven.
    Lfe,
}

impl AudioChannel {
    /// Returns whether positional pan applies.
    #[inline]
    #[must_use]
    pub const fn is_positional(self) -> bool {
        matches!(self, Self::Stereo)
    }
}

/// Looping flag set with explicit counts for deterministic schedulers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum AudioLoop {
    /// Play exactly once.
    Once,
    /// Play forever.
    Infinite,
    /// Play `count` full passes then stop.
    Times(u32),
}

impl AudioLoop {
    /// Returns whether another pass exists after the one that just ended.
    ///
    /// `passes_done` counts completed passes including the one ending now.
    #[inline]
    #[must_use]
    pub const fn has_next_pass(self, passes_done: u32) -> bool {
        match self {
            Self::Once => false,
            Self::Infinite => true,
            Self::Times(count) => passes_done < count,
        }
    }

    /// Total scheduled passes; `u32::MAX` means unbounded.
    #[inline]
    #[must_use]
    pub const fn total_passes(self) -> u32 {
        match self {
            Self::Once => 1,
            Self::Infinite => u32::MAX,
            Self::Times(count) => {
                if count > 1 {
                    count
                } else {
                    1
                }
            }
        }
    }
}

/// A positional audio source with a deterministic playback state machine.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AudioSource {
    /// Opaque source identifier.
    pub id: scenix_core::AudioSourceId,
    /// Listener this source is heard by.
    pub listener: scenix_core::AudioListenerId,
    /// World position in tracking space.
    pub position: Vec3,
    /// Direction the source points, for cone filtering.
    pub direction: Vec3,
    /// Velocity for Doppler, m/s.
    pub velocity: Vec3,
    /// Voice gain multiplier `0..=1`.
    pub volume: f32,
    /// Playback rate multiplier `>= 0`.
    pub pitch: f32,
    /// Looping schedule.
    pub looping: AudioLoop,
    /// Pass currently sounding, `0` for the first pass.
    pub pass: u32,
    /// Output route.
    pub channel: AudioChannel,
    /// Current playback lifecycle.
    pub state: PlaybackState,
    /// Distance model at which volume barely attenuates, meters.
    pub min_distance: f32,
    /// Cutoff distance where the gain floor applies, meters (0 = none).
    pub max_distance: f32,
}

impl AudioSource {
    /// Creates a source with the default id/route/stereo target.
    #[must_use]
    pub fn new(id: u64, state: PlaybackState) -> Self {
        Self {
            id: scenix_core::AudioSourceId::new(id),
            listener: scenix_core::AudioListenerId::new(1),
            position: Vec3::ZERO,
            direction: Vec3::NEG_Z,
            velocity: Vec3::ZERO,
            volume: 1.0,
            pitch: 1.0,
            looping: AudioLoop::Once,
            pass: 0,
            channel: AudioChannel::Stereo,
            state,
            min_distance: 1.0,
            max_distance: 0.0,
        }
    }

    /// Returns the source with a new world position.
    #[inline]
    #[must_use]
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Returns the source pointing along `direction` (normalized to zero
    /// when the input is degenerate).
    #[must_use]
    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.direction = if direction.length_squared() <= 1.0e-12 {
            Vec3::NEG_Z
        } else {
            direction.normalize()
        };
        self
    }

    /// Returns the source with new velocity for Doppler.
    #[inline]
    #[must_use]
    pub const fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    /// Returns the source with a new volume in `0..=1`.
    #[inline]
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Returns the source with a new pitch in `0.1..=4.0`.
    #[inline]
    #[must_use]
    pub const fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.1, 4.0);
        self
    }

    /// Returns the source with a new loop schedule.
    #[inline]
    #[must_use]
    pub const fn with_looping(mut self, looping: AudioLoop) -> Self {
        self.looping = looping;
        self
    }

    /// Returns the source with a new listener binding.
    #[inline]
    #[must_use]
    pub const fn with_listener(mut self, listener: scenix_core::AudioListenerId) -> Self {
        self.listener = listener;
        self
    }

    /// Returns the source routed to `channel`.
    #[inline]
    #[must_use]
    pub const fn with_channel(mut self, channel: AudioChannel) -> Self {
        self.channel = channel;
        self
    }

    /// Returns the source with new attenuation distances.
    #[inline]
    #[must_use]
    pub const fn with_distances(mut self, min: f32, max: f32) -> Self {
        self.min_distance = min.max(0.001);
        self.max_distance = max.max(0.0);
        self
    }

    /// Moves the source into audible state from `Stopped` or `Paused`.
    #[inline]
    pub fn play(&mut self) {
        if self.state != PlaybackState::Playing {
            self.state = PlaybackState::Playing;
        }
    }

    /// Freezes playback, preserving position and pass count.
    #[inline]
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }

    /// Stops sound and rewinds scheduling; looping counters reset.
    #[inline]
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.pass = 0;
    }

    /// Whether this voice may request the next scheduling pass after
    /// finishing pass `pass`. Returns `false` when unbounded.
    ///
    /// `completion_notifications` is a pure function of loop state, so
    /// schedulers never mutate shared counters from mixed paths.
    #[inline]
    #[must_use]
    pub fn scheduler_wants_next_pass(&self) -> bool {
        if self.state != PlaybackState::Playing {
            return false;
        }
        match self.looping {
            AudioLoop::Once => false,
            AudioLoop::Infinite => true,
            AudioLoop::Times(count) => self.pass < count,
        }
    }

    /// Audio-thread offset in burst units after `dt` seconds, used by the
    /// streaming layer to decide when buffers flip.
    #[inline]
    #[must_use]
    pub fn burst_offset(&self, dt: f32) -> f32 {
        if dt <= 0.0 {
            0.0
        } else {
            self.pitch * dt.max(0.0)
        }
    }
}

impl Default for AudioSource {
    #[inline]
    fn default() -> Self {
        Self::new(0, PlaybackState::Stopped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_machine_transitions() {
        let mut source = AudioSource::new(3, PlaybackState::Paused);
        assert!(!source.state.is_audible());
        source.pause(); // no-op when paused
        assert_eq!(source.state, PlaybackState::Paused);
        source.play();
        assert_eq!(source.state, PlaybackState::Playing);
        source.stop();
        assert_eq!(source.state, PlaybackState::Stopped);
        assert_eq!(source.pass, 0);
        assert!(!source.state.is_audible());
    }

    #[test]
    fn gain_and_pitch_clamp() {
        let source = AudioSource::new(3, PlaybackState::Playing)
            .with_volume(4.0)
            .with_pitch(0.0);
        assert_eq!(source.volume, 1.0);
        assert_eq!(source.pitch, 0.1);
        assert!((source.burst_offset(0.5) - 0.05).abs() < 1.0e-6);
    }

    #[test]
    fn loop_schedules() {
        assert!(!AudioLoop::Once.has_next_pass(0));
        assert!(AudioLoop::Infinite.has_next_pass(1_000_000));
        assert!(AudioLoop::Times(3).has_next_pass(2));
        assert!(!AudioLoop::Times(3).has_next_pass(3));
        assert_eq!(AudioLoop::Times(0).total_passes(), 1);
    }

    #[test]
    fn scheduler_respects_state() {
        let mut source =
            AudioSource::new(3, PlaybackState::Playing).with_looping(AudioLoop::Times(3));
        source.pass = 1;
        assert!(source.scheduler_wants_next_pass(), "two passes remain");
        source.pass = 3;
        assert!(!source.scheduler_wants_next_pass(), "all passes completed");
        source.pause();
        assert!(!source.scheduler_wants_next_pass());
    }

    #[test]
    fn direction_normalization() {
        let source = AudioSource::new(1, PlaybackState::Stopped).with_direction(Vec3::ZERO);
        assert_eq!(source.direction, Vec3::NEG_Z);
        let source = AudioSource::new(1, PlaybackState::Stopped).with_direction(Vec3::Y);
        assert!((source.direction.length() - 1.0).abs() < 1.0e-5);
        assert!(
            source.channel.is_positional(),
            "a set direction makes the source positional"
        );
    }
}
