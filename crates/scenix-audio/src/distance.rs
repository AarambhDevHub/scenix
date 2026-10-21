//! Deterministic distance-attenuation models.

/// Gain models available to positional sources.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum AttenuationModel {
    /// Simple linear falloff between `min..max` distance.
    Linear,
    /// `1 / (1 + rolloff * (d - min))`, physically motivated.
    Inverse,
    /// `1 / d^2`, the most aggressive realistic falloff.
    #[default]
    InverseSquare,
}

impl AttenuationModel {
    /// Computes gain for distance `d` with `min`/`max` bounds, clamped to
    /// `0..=1`.
    ///
    /// `max <= 0` disables the far clamp entirely (pure model gain).
    #[must_use]
    pub fn gain(self, d: f32, min: f32, max: f32, rolloff: f32) -> f32 {
        let distance = if d.is_finite() { d.max(0.0) } else { 0.0 };
        let min = min.max(0.0);
        let effective = distance.max(min);
        let high_floor = if max > min && max > 0.0 {
            min / max
        } else {
            0.0
        };
        let gain = match self {
            Self::Linear => {
                if max <= min {
                    1.0
                } else {
                    (1.0 - (effective - min) / (max - min)).max(0.0)
                }
            }
            Self::Inverse => 1.0 / (1.0 + rolloff.max(0.0) * (effective - min)),
            Self::InverseSquare => {
                if effective <= 1.0e-6 {
                    1.0
                } else {
                    1.0 / (effective * effective)
                }
            }
        };
        if high_floor > 0.0 {
            gain.max(high_floor).min(1.0)
        } else {
            gain.min(1.0)
        }
    }

    /// Raw per-model gain without the `max` floor, for research profiles.
    #[must_use]
    pub fn raw_gain(self, d: f32, min: f32, rolloff: f32) -> f32 {
        let distance = if d.is_finite() { d.max(min) } else { min };
        match self {
            Self::Linear => 0.0,
            Self::Inverse => 1.0 / (1.0 + rolloff.max(0.0) * (distance - min)),
            Self::InverseSquare => {
                if distance <= 1.0e-6 {
                    1.0
                } else {
                    1.0 / (distance * distance)
                }
            }
        }
    }
}

/// Per-ear distance gain for one source.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DistanceGain {
    /// Farther ear multiplier, clamped to `[0, 1]`.
    pub far: f32,
    /// Nearer ear multiplier, clamped to `[0, 1]`.
    pub near: f32,
}

impl DistanceGain {
    /// Flat, unfiltered pair.
    #[inline]
    #[must_use]
    pub const fn flat(gain: f32) -> Self {
        Self {
            far: gain,
            near: gain,
        }
    }

    /// Averaged gain for `Lfe`/`Center` routes.
    #[inline]
    #[must_use]
    pub fn center(&self) -> f32 {
        (self.far + self.near) * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_square_attenuates_monotonically() {
        let near = AttenuationModel::InverseSquare.raw_gain(1.0, 0.5, 1.0);
        let far = AttenuationModel::InverseSquare.raw_gain(4.0, 0.5, 1.0);
        assert!(near > far);
        assert!((near - 1.0).abs() < 1.0e-4);
        assert!(far > 0.0 && far < 1.0);
    }

    #[test]
    fn linear_model_falls_to_zero_at_max() {
        let model = AttenuationModel::Linear;
        assert!((model.gain(0.0, 0.0, 10.0, 1.0) - 1.0).abs() < 1.0e-6);
        assert!(
            model.gain(10.0, 0.0, 10.0, 1.0) <= 1.0e-6,
            "linear rolls to zero at max distance"
        );
        // Unbounded linear treats reach as full volume.
        assert!((model.gain(4.0, 0.0, -1.0, 1.0) - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn inverse_model_uses_rolloff() {
        assert!((AttenuationModel::Inverse.raw_gain(1.0, 1.0, 0.0) - 1.0).abs() < 1.0e-6);
        let rolled = AttenuationModel::Inverse.raw_gain(3.0, 1.0, 1.0);
        assert!((rolled - 1.0 / 3.0).abs() < 1.0e-6, "1/(1 + r*(d-min))");
        // Far clamps never lift gain above the base curve.
        let clamped = AttenuationModel::Inverse.gain(3.0, 1.0, 6.0, 1.0);
        assert!((1.0 / 6.0..=0.5).contains(&clamped));
    }

    #[test]
    fn degenerate_distances() {
        assert!((AttenuationModel::InverseSquare.raw_gain(0.0, 0.0, 1.0) - 1.0).abs() < 1.0e-6);
        assert!(
            !AttenuationModel::InverseSquare
                .raw_gain(f32::NAN, 0.5, 1.0)
                .is_nan()
        );
        assert!(
            (AttenuationModel::InverseSquare.gain(f32::NAN, 0.5, 0.0, 1.0) - 1.0).abs() < 1.0e-3,
            "NaN distance clamps to min-distance gain"
        );
        assert_eq!(DistanceGain::flat(0.5).center(), 0.5);
    }
}
