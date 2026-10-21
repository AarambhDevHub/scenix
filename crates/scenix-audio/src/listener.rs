//! The spatial hearing point of the system.

use scenix_math::Vec3;

/// Maximum number of positional sources a listener processes per frame.
///
/// Keeping the budget fixed keeps the spatial mix deterministic and makes
/// the WGSL uniform layout trivially packable.
pub const AUDIO_LISTENER_MAX_SOURCES: usize = 32;

/// Head and ear state used by every spatialization model.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AudioListener {
    /// Opaque listener identifier.
    pub id: scenix_core::AudioListenerId,
    /// World position of the listener's head.
    pub position: Vec3,
    /// Forward axis (typically the head's gaze).
    pub forward: Vec3,
    /// Up axis; mutually orthogonal with `forward` after building.
    pub up: Vec3,
    /// Head velocity for Doppler, in m/s.
    pub velocity: Vec3,
    /// Global master gain applied after per-source mixing `0..=1`.
    pub master_gain: f32,
}

impl AudioListener {
    /// Creates a listener with a stable default id and scene orientation.
    #[must_use]
    pub fn at(position: Vec3) -> Self {
        Self {
            id: scenix_core::AudioListenerId::new(1),
            position,
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
            velocity: Vec3::ZERO,
            master_gain: 1.0,
        }
    }

    /// Builds the listener from head axes, orthonormalizing `forward`
    /// against `up` for deterministic matte-looking ears.
    #[must_use]
    pub fn axes(position: Vec3, forward: Vec3, up: Vec3) -> Self {
        let forward = if forward.length_squared() <= 1.0e-12 {
            Vec3::NEG_Z
        } else {
            forward.normalize()
        };
        let right = forward.cross(up);
        let up = if right.length_squared() <= 1.0e-12 {
            Vec3::Y
        } else {
            right.cross(forward).normalize()
        };
        Self {
            id: scenix_core::AudioListenerId::new(1),
            position,
            forward,
            up,
            velocity: Vec3::ZERO,
            master_gain: 1.0,
        }
    }

    /// Returns the listener with a new world position.
    #[inline]
    #[must_use]
    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Returns the listener with a new head velocity.
    #[inline]
    #[must_use]
    pub const fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    /// Returns the listener with a new master gain in `0..=1`.
    #[inline]
    #[must_use]
    pub const fn with_master_gain(mut self, gain: f32) -> Self {
        self.master_gain = gain.clamp(0.0, 1.0);
        self
    }

    /// Returns the listener with a new id.
    #[inline]
    #[must_use]
    pub const fn with_id(mut self, id: scenix_core::AudioListenerId) -> Self {
        self.id = id;
        self
    }

    /// Right ear axis after orthonormalization.
    #[inline]
    #[must_use]
    pub fn right(&self) -> Vec3 {
        let right = self.forward.cross(self.up);
        if right.length_squared() <= 1.0e-12 {
            Vec3::X
        } else {
            right.normalize()
        }
    }

    /// Center-channel share used while distributing power around the head.
    #[inline]
    #[must_use]
    pub const fn scale_hint(&self) -> f32 {
        CENTER_CHANNEL_WEIGHT
    }

    /// True when the head orientation is degenerate and needs rebuilding.
    #[inline]
    #[must_use]
    pub fn orientation_is_degenerate(&self) -> bool {
        self.forward.length_squared() <= 1.0e-12
            || self.up.length_squared() <= 1.0e-12
            || self.forward.dot(self.up).abs() > 0.999
    }
}

impl Default for AudioListener {
    #[inline]
    fn default() -> Self {
        Self::at(Vec3::ZERO)
    }
}

/// One-third constant used when distributing center-channel power.
pub const CENTER_CHANNEL_WEIGHT: f32 = 1.0 / 3.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_listener_faces_into_scene() {
        let listener = AudioListener::default();
        assert_eq!(listener.forward, Vec3::NEG_Z);
        assert_eq!(listener.up, Vec3::Y);
        assert_eq!(listener.master_gain, 1.0);
        assert_eq!(listener.id, scenix_core::AudioListenerId::new(1));
        assert!((CENTER_CHANNEL_WEIGHT - listener.scale_hint()).abs() < 1.0e-7);
    }

    #[test]
    fn axes_orthonormalize() {
        let listener = AudioListener::axes(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y);
        assert!(!listener.orientation_is_degenerate());
        assert!((listener.right().dot(listener.forward)).abs() < 1.0e-5);
        assert!((listener.right().dot(listener.up)).abs() < 1.0e-5);
        assert!((listener.forward.dot(listener.up)).abs() < 1.0e-5);
    }

    #[test]
    fn degenerate_up_falls_back() {
        // Parallel forward/up is a degenerate basis.
        let listener = AudioListener::axes(Vec3::ZERO, Vec3::Y, Vec3::Y);
        assert!(listener.orientation_is_degenerate());
        let up = listener.up;
        assert!(!(up.x.is_nan() || up.y.is_nan() || up.z.is_nan()));
    }
}
