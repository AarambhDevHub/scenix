use alloc::string::String;

use scenix_core::{Color, TextureId};

use crate::traits::{double_sided_bit, option_texture_bit};
use crate::{AlphaMode, FEATURE_ALBEDO_TEXTURE, Material, PipelineKey, ShaderKind};

/// Constant-color material that ignores scene lighting.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnlitMaterial {
    /// Human-readable material name.
    pub name: String,
    /// Color in linear RGBA.
    pub color: Color,
    /// Optional color texture.
    pub color_texture: Option<TextureId>,
    /// Alpha behavior.
    pub alpha_mode: AlphaMode,
    /// Whether the material is rendered double-sided.
    pub double_sided: bool,
}

impl UnlitMaterial {
    /// Creates a default opaque white unlit material.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this material with a color.
    #[inline]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns this material with alpha behavior.
    #[inline]
    pub const fn alpha_mode(mut self, alpha_mode: AlphaMode) -> Self {
        self.alpha_mode = alpha_mode;
        self
    }
}

impl Default for UnlitMaterial {
    #[inline]
    fn default() -> Self {
        Self {
            name: String::new(),
            color: Color::WHITE,
            color_texture: None,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        }
    }
}

impl Material for UnlitMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        PipelineKey::new(
            ShaderKind::Unlit,
            self.alpha_mode.pipeline_alpha(),
            double_sided_bit(self.double_sided)
                | option_texture_bit(&self.color_texture, FEATURE_ALBEDO_TEXTURE),
        )
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
