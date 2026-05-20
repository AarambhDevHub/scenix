use scenix_core::Color;

/// Constant ambient scene light.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AmbientLight {
    /// Light color in linear RGB.
    pub color: Color,
    /// Scalar intensity.
    pub intensity: f32,
}

impl AmbientLight {
    /// Creates an ambient light.
    #[inline]
    pub const fn new(color: Color, intensity: f32) -> Self {
        Self { color, intensity }
    }
}

impl Default for AmbientLight {
    #[inline]
    fn default() -> Self {
        Self::new(Color::WHITE, 1.0)
    }
}
