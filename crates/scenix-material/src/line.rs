use scenix_core::Color;

use crate::{
    AlphaMode, FEATURE_DASHED, FEATURE_WORLD_SPACE, Material, PipelineAlphaMode, PipelineKey,
    ShaderKind,
};

/// Line material with optional dash pattern.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineMaterial {
    /// Line color in linear RGBA.
    pub color: Color,
    /// Line width in logical pixels unless `world_units` is true.
    pub width: f32,
    /// Dash length. `0.0` disables dashed rendering.
    pub dash_size: f32,
    /// Gap length between dashes.
    pub gap_size: f32,
    /// Interpret width in world units instead of logical pixels.
    pub world_units: bool,
    /// Alpha behavior.
    pub alpha_mode: AlphaMode,
}

impl LineMaterial {
    /// Creates a default one-pixel white line material.
    #[inline]
    pub const fn new() -> Self {
        Self {
            color: Color::WHITE,
            width: 1.0,
            dash_size: 0.0,
            gap_size: 0.0,
            world_units: false,
            alpha_mode: AlphaMode::Opaque,
        }
    }

    /// Returns this material with a dash pattern.
    #[inline]
    pub fn dashed(mut self, dash_size: f32, gap_size: f32) -> Self {
        self.dash_size = dash_size.max(0.0);
        self.gap_size = gap_size.max(0.0);
        self
    }
}

impl Default for LineMaterial {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Material for LineMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        let mut bits = 0;
        if self.dash_size > 0.0 && self.gap_size > 0.0 {
            bits |= FEATURE_DASHED;
        }
        if self.world_units {
            bits |= FEATURE_WORLD_SPACE;
        }
        let alpha = if self.color.a < 1.0 {
            PipelineAlphaMode::Blend
        } else {
            self.alpha_mode.pipeline_alpha()
        };
        PipelineKey::new(ShaderKind::Line, alpha, bits)
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
