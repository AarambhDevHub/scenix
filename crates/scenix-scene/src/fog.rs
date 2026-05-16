use scenix_core::Color;

/// Scene-wide fog configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Fog {
    /// Linear fog between `near` and `far`.
    Linear {
        /// Distance where fog starts.
        near: f32,
        /// Distance where fog reaches full strength.
        far: f32,
        /// Fog color.
        color: Color,
    },
    /// Exponential fog controlled by density.
    Exponential {
        /// Fog density.
        density: f32,
        /// Fog color.
        color: Color,
    },
}

impl Fog {
    /// Creates linear fog.
    #[inline]
    pub const fn linear(near: f32, far: f32, color: Color) -> Self {
        Self::Linear { near, far, color }
    }

    /// Creates exponential fog.
    #[inline]
    pub const fn exponential(density: f32, color: Color) -> Self {
        Self::Exponential { density, color }
    }
}
