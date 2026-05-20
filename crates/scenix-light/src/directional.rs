use scenix_core::Color;
use scenix_math::Vec3;

use crate::ShadowConfig;

/// Directional light with optional shadow configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirectionalLight {
    /// Direction the light shines toward.
    pub direction: Vec3,
    /// Light color in linear RGB.
    pub color: Color,
    /// Scalar intensity.
    pub intensity: f32,
    /// Optional shadow configuration.
    pub shadow: Option<ShadowConfig>,
}

impl DirectionalLight {
    /// Creates a directional light. Zero directions fall back to negative Z.
    #[inline]
    pub fn new(direction: Vec3, color: Color, intensity: f32) -> Self {
        let direction = direction.normalize();
        Self {
            direction: if direction == Vec3::ZERO {
                Vec3::NEG_Z
            } else {
                direction
            },
            color,
            intensity,
            shadow: None,
        }
    }

    /// Returns this light with shadow configuration.
    #[inline]
    pub const fn shadow(mut self, shadow: ShadowConfig) -> Self {
        self.shadow = Some(shadow);
        self
    }
}

impl Default for DirectionalLight {
    #[inline]
    fn default() -> Self {
        Self::new(Vec3::NEG_Z, Color::WHITE, 1.0)
    }
}
