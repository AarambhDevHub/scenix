use alloc::string::String;

use scenix_core::{Color, TextureId};
use scenix_math::Vec3;

use crate::traits::{double_sided_bit, option_texture_bit};
use crate::{AlphaMode, FEATURE_ALBEDO_TEXTURE, Material, PipelineKey, ShaderKind};

/// Diffuse-only material for fast lit surfaces.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LambertMaterial {
    /// Human-readable material name.
    pub name: String,
    /// Diffuse color in linear RGBA.
    pub color: Color,
    /// Optional diffuse color texture.
    pub color_texture: Option<TextureId>,
    /// Emissive RGB color in linear space.
    pub emissive: Vec3,
    /// Alpha behavior.
    pub alpha_mode: AlphaMode,
    /// Whether the material is rendered double-sided.
    pub double_sided: bool,
}

impl LambertMaterial {
    /// Creates a default opaque white Lambert material.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this material with a diffuse color.
    #[inline]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Default for LambertMaterial {
    #[inline]
    fn default() -> Self {
        Self {
            name: String::new(),
            color: Color::WHITE,
            color_texture: None,
            emissive: Vec3::ZERO,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        }
    }
}

impl Material for LambertMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        PipelineKey::new(
            ShaderKind::Lambert,
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
