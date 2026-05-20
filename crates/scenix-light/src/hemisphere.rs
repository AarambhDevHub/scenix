use scenix_core::Color;

/// Sky/ground gradient ambient light.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HemisphereLight {
    /// Sky-facing color in linear RGB.
    pub sky_color: Color,
    /// Ground-facing color in linear RGB.
    pub ground_color: Color,
    /// Scalar intensity.
    pub intensity: f32,
}

impl HemisphereLight {
    /// Creates a hemisphere light.
    #[inline]
    pub const fn new(sky_color: Color, ground_color: Color, intensity: f32) -> Self {
        Self {
            sky_color,
            ground_color,
            intensity,
        }
    }
}

impl Default for HemisphereLight {
    #[inline]
    fn default() -> Self {
        Self::new(Color::WHITE, Color::BLACK, 1.0)
    }
}
