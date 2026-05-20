use scenix_core::Color;

use crate::traits::double_sided_bit;
use crate::{FEATURE_WIREFRAME, Material, PipelineAlphaMode, PipelineKey, ShaderKind};

/// Wireframe overlay material.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WireframeMaterial {
    /// Wire color in linear RGBA.
    pub color: Color,
    /// Wire opacity in `0.0..=1.0`.
    pub opacity: f32,
    /// Wire width in logical pixels.
    pub line_width: f32,
    /// Whether the material is rendered double-sided.
    pub double_sided: bool,
}

impl WireframeMaterial {
    /// Creates a default black wireframe material.
    #[inline]
    pub const fn new() -> Self {
        Self {
            color: Color::BLACK,
            opacity: 1.0,
            line_width: 1.0,
            double_sided: true,
        }
    }
}

impl Default for WireframeMaterial {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Material for WireframeMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        let alpha = if self.opacity < 1.0 || self.color.a < 1.0 {
            PipelineAlphaMode::Blend
        } else {
            PipelineAlphaMode::Opaque
        };
        PipelineKey::new(
            ShaderKind::Wireframe,
            alpha,
            double_sided_bit(self.double_sided) | FEATURE_WIREFRAME,
        )
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.opacity < 1.0 || self.color.a < 1.0
    }

    #[inline]
    fn double_sided(&self) -> bool {
        self.double_sided
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        None
    }
}
