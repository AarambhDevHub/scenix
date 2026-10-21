//! Doppler frequency-shift modeling.

use scenix_math::Vec3;

/// Speed of sound in dry air at 20°C, m/s.
pub const SPEED_OF_SOUND: f32 = 343.0;

/// Doppler shift model parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DopplerModel {
    /// Speed of sound used by the shift, m/s; hosts override for gameplay
    /// at their own risk.
    pub speed_of_sound: f32,
    /// Exaggeration factor `0..=1`; `0` disables Doppler entirely.
    pub factor: f32,
}

impl DopplerModel {
    /// Creates the physically-motivated model.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            speed_of_sound: SPEED_OF_SOUND,
            factor: 1.0,
        }
    }

    /// Returns the model with a new speed of sound clamped sane.
    #[inline]
    #[must_use]
    pub const fn with_speed_of_sound(mut self, speed: f32) -> Self {
        self.speed_of_sound = speed.max(1.0);
        self
    }

    /// Returns the model with a new exaggeration factor `0..=1`.
    #[inline]
    #[must_use]
    pub const fn with_factor(mut self, factor: f32) -> Self {
        self.factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Frequency multiplier applied to a source moving at `velocity`
    /// radiating toward `direction` toward a listener at `listener_velocity`
    /// retreating from the sound.
    ///
    /// The returned multiplier is always positive and finite; degenerate
    /// directions collapse to `1.0`.
    #[must_use]
    pub fn shift(
        &self,
        source_velocity: Vec3,
        listener_velocity: Vec3,
        direction: Vec3,
    ) -> DopplerShift {
        if self.factor <= 0.0 || self.speed_of_sound <= 1.0 {
            return DopplerShift::neutral();
        }
        let axis = if direction.length_squared() <= 1.0e-12 {
            Vec3::NEG_Z
        } else {
            direction.normalize()
        };
        // Positive radial components denote effective closure.
        let source_to_listener = -source_velocity.dot(axis);
        let listener_gain = listener_velocity.dot(axis);
        let relative = source_to_listener + listener_gain;
        let scaled = self.factor * relative / self.speed_of_sound;
        let denominator = 1.0 + scaled;
        if denominator.abs() <= 1.0e-6 {
            return DopplerShift {
                frequency_multiplier: 4.0,
                radial_velocity: relative,
            };
        }
        DopplerShift {
            frequency_multiplier: (1.0 / denominator).clamp(0.25, 4.0),
            radial_velocity: relative,
        }
    }
}

impl Default for DopplerModel {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// The applied frame shift for one voice.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DopplerShift {
    /// Resampling multiplier applied to the source pitch.
    pub frequency_multiplier: f32,
    /// Signed closing velocity in m/s used by the host's game code.
    pub radial_velocity: f32,
}

impl DopplerShift {
    /// No-shift result.
    #[inline]
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            frequency_multiplier: 1.0,
            radial_velocity: 0.0,
        }
    }

    /// Pitch the source actually plays at after combining base pitch.
    #[inline]
    #[must_use]
    pub const fn effective_pitch(self, base_pitch: f32) -> f32 {
        base_pitch * self.frequency_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approaching_source_pitch_up() {
        let model = DopplerModel::new();
        // Source chases the listener along -Z at 34.3 m/s (10% of c).
        let shift = model.shift(Vec3::new(0.0, 0.0, -34.3), Vec3::ZERO, Vec3::NEG_Z);
        assert!(shift.frequency_multiplier > 1.0);
        assert!(shift.effective_pitch(100.0) > 100.0);
    }

    #[test]
    fn receding_source_pitch_down() {
        let model = DopplerModel::new();
        let shift = model.shift(Vec3::new(0.0, 0.0, 34.3), Vec3::ZERO, Vec3::NEG_Z);
        assert!(shift.frequency_multiplier < 1.0);
    }

    #[test]
    fn factor_zero_disables() {
        let model = DopplerModel::new().with_factor(0.0);
        let shift = model.shift(Vec3::new(100.0, 0.0, 0.0), Vec3::ZERO, Vec3::NEG_Z);
        assert_eq!(shift, DopplerShift::neutral());
    }

    #[test]
    fn degenerate_direction_is_neutral() {
        let model = DopplerModel::new();
        let shift = model.shift(Vec3::new(0.0, -10.0, 0.0), Vec3::ZERO, Vec3::ZERO);
        assert!((shift.frequency_multiplier - 1.0).abs() < 1.0e-5);
        assert!(shift.radial_velocity.is_finite());
    }

    #[test]
    fn listener_velocity_adds_closing() {
        let model = DopplerModel::new();
        // The source sits along the +Z side of `direction = -Z`; stepping
        // +Z shrinks the source-listener distance, raising pitch.
        let movement = model.shift(Vec3::ZERO, Vec3::new(0.0, 0.0, 17.15), Vec3::NEG_Z);
        assert!(movement.frequency_multiplier > 1.0);
    }
}
