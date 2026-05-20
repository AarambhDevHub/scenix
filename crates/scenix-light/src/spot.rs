use scenix_core::Color;

use crate::{ShadowConfig, clamp01};

/// Cone-shaped punctual light.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpotLight {
    /// Light color in linear RGB.
    pub color: Color,
    /// Scalar intensity.
    pub intensity: f32,
    /// Maximum influence distance. `0.0` means unbounded.
    pub range: f32,
    /// Outer cone half-angle in radians.
    pub angle: f32,
    /// Soft-edge fraction in `0.0..=1.0`.
    pub penumbra: f32,
    /// Optional shadow configuration.
    pub shadow: Option<ShadowConfig>,
}

impl SpotLight {
    /// Creates a spot light.
    #[inline]
    pub fn new(color: Color, intensity: f32, range: f32, angle: f32) -> Self {
        Self {
            color,
            intensity,
            range: range.max(0.0),
            angle: angle.max(0.0),
            penumbra: 0.0,
            shadow: None,
        }
    }

    /// Returns this light with a soft-edge fraction.
    #[inline]
    pub fn penumbra(mut self, penumbra: f32) -> Self {
        self.penumbra = clamp01(penumbra);
        self
    }

    /// Returns this light with shadow configuration.
    #[inline]
    pub const fn shadow(mut self, shadow: ShadowConfig) -> Self {
        self.shadow = Some(shadow);
        self
    }
}

impl Default for SpotLight {
    #[inline]
    fn default() -> Self {
        Self::new(Color::WHITE, 1.0, 0.0, core::f32::consts::FRAC_PI_4)
    }
}
