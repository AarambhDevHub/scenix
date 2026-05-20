use scenix_core::Color;

use crate::positive;

/// Rectangular area light description.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AreaLight {
    /// Emitter width in world units.
    pub width: f32,
    /// Emitter height in world units.
    pub height: f32,
    /// Light color in linear RGB.
    pub color: Color,
    /// Scalar intensity.
    pub intensity: f32,
}

impl AreaLight {
    /// Creates an area light.
    #[inline]
    pub fn new(width: f32, height: f32, color: Color, intensity: f32) -> Self {
        Self {
            width: positive(width, 1.0),
            height: positive(height, 1.0),
            color,
            intensity,
        }
    }
}

impl Default for AreaLight {
    #[inline]
    fn default() -> Self {
        Self::new(1.0, 1.0, Color::WHITE, 1.0)
    }
}
