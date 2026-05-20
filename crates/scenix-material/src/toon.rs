use alloc::string::String;

use scenix_core::{Color, TextureId};

use crate::traits::{double_sided_bit, option_texture_bit};
use crate::{
    AlphaMode, FEATURE_ALBEDO_TEXTURE, FEATURE_GRADIENT_TEXTURE, FEATURE_OUTLINE, Material,
    PipelineKey, ShaderKind,
};

/// Cel-shading material with discrete lighting bands.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ToonMaterial {
    /// Human-readable material name.
    pub name: String,
    /// Base color in linear RGBA.
    pub color: Color,
    /// Optional base color texture.
    pub color_texture: Option<TextureId>,
    /// Optional one-dimensional gradient/ramp texture.
    pub gradient_map: Option<TextureId>,
    /// Number of fallback discrete bands when no gradient map is present.
    pub steps: u32,
    /// Outline width. `0.0` disables the outline path.
    pub outline_width: f32,
    /// Outline color in linear RGBA.
    pub outline_color: Color,
    /// Alpha behavior.
    pub alpha_mode: AlphaMode,
    /// Whether the material is rendered double-sided.
    pub double_sided: bool,
}

impl ToonMaterial {
    /// Creates a default toon material.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this material with a fallback band count.
    #[inline]
    pub const fn steps(mut self, steps: u32) -> Self {
        self.steps = steps;
        self
    }

    /// Returns this material with outline settings.
    #[inline]
    pub fn outline(mut self, width: f32, color: Color) -> Self {
        self.outline_width = width.max(0.0);
        self.outline_color = color;
        self
    }
}

impl Default for ToonMaterial {
    #[inline]
    fn default() -> Self {
        Self {
            name: String::new(),
            color: Color::WHITE,
            color_texture: None,
            gradient_map: None,
            steps: 4,
            outline_width: 0.0,
            outline_color: Color::BLACK,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        }
    }
}

impl Material for ToonMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        let mut bits = double_sided_bit(self.double_sided)
            | option_texture_bit(&self.color_texture, FEATURE_ALBEDO_TEXTURE)
            | option_texture_bit(&self.gradient_map, FEATURE_GRADIENT_TEXTURE);
        if self.outline_width > 0.0 {
            bits |= FEATURE_OUTLINE;
        }
        PipelineKey::new(ShaderKind::Toon, self.alpha_mode.pipeline_alpha(), bits)
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.alpha_mode.is_transparent()
    }

    #[inline]
    fn double_sided(&self) -> bool {
        self.double_sided
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        self.alpha_mode.cutoff()
    }
}
