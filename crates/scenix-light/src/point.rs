use scenix_core::Color;

use crate::{ShadowConfig, positive};

/// Omnidirectional punctual light.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointLight {
    /// Light color in linear RGB.
    pub color: Color,
    /// Scalar intensity.
    pub intensity: f32,
    /// Maximum influence distance. `0.0` means unbounded.
    pub range: f32,
    /// Distance falloff exponent. Physically based inverse-square falloff is `2.0`.
    pub decay: f32,
    /// Optional shadow configuration.
    pub shadow: Option<ShadowConfig>,
}

impl PointLight {
    /// Creates a point light.
    #[inline]
    pub fn new(color: Color, intensity: f32, range: f32) -> Self {
        Self {
            color,
            intensity,
            range: range.max(0.0),
            decay: 2.0,
            shadow: None,
        }
    }

    /// Returns this light with a distance falloff exponent.
    #[inline]
    pub fn decay(mut self, decay: f32) -> Self {
        self.decay = positive(decay, 2.0);
        self
    }

    /// Returns this light with shadow configuration.
    #[inline]
    pub const fn shadow(mut self, shadow: ShadowConfig) -> Self {
        self.shadow = Some(shadow);
        self
    }
}

impl Default for PointLight {
    #[inline]
    fn default() -> Self {
        Self::new(Color::WHITE, 1.0, 0.0)
    }
}
