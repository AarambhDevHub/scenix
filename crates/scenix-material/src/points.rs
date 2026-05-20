use scenix_core::Color;

use crate::{
    AlphaMode, FEATURE_SIZE_ATTENUATION, Material, PipelineAlphaMode, PipelineKey, ShaderKind,
};

/// Point material for point-list geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointsMaterial {
    /// Point color in linear RGBA.
    pub color: Color,
    /// Point size in logical pixels.
    pub size: f32,
    /// Whether point size attenuates with depth.
    pub size_attenuation: bool,
    /// Alpha behavior.
    pub alpha_mode: AlphaMode,
}

impl PointsMaterial {
    /// Creates a default white point material.
    #[inline]
    pub const fn new() -> Self {
        Self {
            color: Color::WHITE,
            size: 1.0,
            size_attenuation: true,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}

impl Default for PointsMaterial {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Material for PointsMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        let bits = if self.size_attenuation {
            FEATURE_SIZE_ATTENUATION
        } else {
            0
        };
        let alpha = if self.color.a < 1.0 {
            PipelineAlphaMode::Blend
        } else {
            self.alpha_mode.pipeline_alpha()
        };
        PipelineKey::new(ShaderKind::Points, alpha, bits)
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.alpha_mode.is_transparent() || self.color.a < 1.0
    }

    #[inline]
    fn double_sided(&self) -> bool {
        true
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        self.alpha_mode.cutoff()
    }
}
