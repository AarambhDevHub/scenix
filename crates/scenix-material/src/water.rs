//! Water surface material descriptor (scenix v1.8.0).
//!
//! A [`WaterMaterial`] carries the visual end of `scenix-water`'s
//! `WaterSurface`: deep/shallow tints, Schlick base
//! reflectance, foam mask parameters, and the optional
//! reflection/refraction render-target textures the renderer samples
//! through its `water_surface` WGSL pipeline.
//!
//! Water is always alpha-blended, so the pipeline key selects the
//! `Blend` alpha mode.

use scenix_core::{Color, TextureId};

use crate::traits::{FEATURE_ALBEDO_TEXTURE, option_texture_bit};
use crate::{
    FEATURE_FOAM, FEATURE_FRESNEL, FEATURE_REFRACTION, Material, PipelineAlphaMode, PipelineKey,
    ShaderKind,
};

/// Water surface material.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaterMaterial {
    /// Deep-water absorption tint.
    pub deep_color: Color,
    /// Shallow-water tint mixed near grazing angles.
    pub shallow_color: Color,
    /// Mirror target texture (registered render target id).
    pub reflection_texture: Option<TextureId>,
    /// Refraction target texture (`None` falls back to pure tint math).
    pub refraction_texture: Option<TextureId>,
    /// Schlick base reflectance `F0` (water sits near `0.02`).
    pub base_reflectance: f32,
    /// Reflection texture mix strength `0..=1`.
    pub reflection_strength: f32,
    /// Refraction distortion strength `0..=1`.
    pub refraction_strength: f32,
    /// Foam coverage gate `0..=1`; `None` disables the foam path.
    pub foam_threshold: Option<f32>,
    /// Foam edge sharpness multiplier.
    pub foam_sharpness: f32,
}

impl WaterMaterial {
    /// The clear-pool default: half mirror, one-third distortion, foam off.
    #[inline]
    #[must_use]
    pub fn clear() -> Self {
        Self {
            deep_color: Color::rgb(0.02, 0.13, 0.18),
            shallow_color: Color::rgb(0.12, 0.42, 0.42),
            reflection_texture: None,
            refraction_texture: None,
            base_reflectance: 0.020_371,
            reflection_strength: 0.55,
            refraction_strength: 0.35,
            foam_threshold: None,
            foam_sharpness: 6.0,
        }
    }

    /// Returns the water with a tint pair `(deep, shallow)`.
    #[inline]
    #[must_use]
    pub const fn with_tint(mut self, deep_color: Color, shallow_color: Color) -> Self {
        self.deep_color = deep_color;
        self.shallow_color = shallow_color;
        self
    }

    /// Returns the water with a reflection target bound.
    #[inline]
    #[must_use]
    pub const fn with_reflection(mut self, texture_id: TextureId) -> Self {
        self.reflection_texture = Some(texture_id);
        self
    }

    /// Returns the water with a refraction target bound.
    #[inline]
    #[must_use]
    pub const fn with_refraction(mut self, texture_id: TextureId) -> Self {
        self.refraction_texture = Some(texture_id);
        self
    }

    /// Returns the water with fresnel and strength knobs.
    #[inline]
    #[must_use]
    pub const fn with_fresnel(
        mut self,
        base_reflectance: f32,
        reflection_strength: f32,
        refraction_strength: f32,
    ) -> Self {
        self.base_reflectance = base_reflectance.clamp(0.0, 1.0);
        self.reflection_strength = reflection_strength.clamp(0.0, 1.0);
        self.refraction_strength = refraction_strength.clamp(0.0, 1.0);
        self
    }

    /// Returns the water with the foam path enabled `(threshold, sharpness)`.
    #[inline]
    #[must_use]
    pub const fn with_foam(mut self, threshold: f32, sharpness: f32) -> Self {
        self.foam_threshold = Some(threshold.clamp(0.0, 1.0));
        self.foam_sharpness = sharpness.max(0.0);
        self
    }
}

impl Material for WaterMaterial {
    fn pipeline_key(&self) -> PipelineKey {
        let mut bits = option_texture_bit(&self.reflection_texture, FEATURE_ALBEDO_TEXTURE);
        if self.refraction_texture.is_some() {
            bits |= FEATURE_REFRACTION;
        }
        if self.foam_threshold.is_some() {
            bits |= FEATURE_FOAM;
        }
        bits |= FEATURE_FRESNEL;
        PipelineKey::new(ShaderKind::Water, PipelineAlphaMode::Blend, bits)
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        true
    }

    #[inline]
    fn double_sided(&self) -> bool {
        false
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::WaterMaterial;
    use crate::{
        FEATURE_ALBEDO_TEXTURE, FEATURE_FOAM, FEATURE_FRESNEL, FEATURE_REFRACTION, Material,
        PipelineAlphaMode, ShaderKind,
    };
    use scenix_core::{Color, TextureId};

    #[test]
    fn water_pipeline_routes_targets_and_foam() {
        let bare = WaterMaterial::clear();
        assert_eq!(bare.pipeline_key().shader, ShaderKind::Water);
        assert_eq!(bare.pipeline_key().alpha, PipelineAlphaMode::Blend);
        assert!(bare.is_transparent());
        assert!(bare.pipeline_key().has_feature(FEATURE_FRESNEL));
        assert!(!bare.pipeline_key().has_feature(FEATURE_ALBEDO_TEXTURE));
        assert!(!bare.pipeline_key().has_feature(FEATURE_FOAM));
        assert!(!bare.pipeline_key().has_feature(FEATURE_REFRACTION));

        let full = bare
            .with_reflection(TextureId::new(4))
            .with_refraction(TextureId::new(5))
            .with_foam(0.5, 6.0);
        assert!(full.pipeline_key().has_feature(FEATURE_ALBEDO_TEXTURE));
        assert!(full.pipeline_key().has_feature(FEATURE_REFRACTION));
        assert!(full.pipeline_key().has_feature(FEATURE_FOAM));
        assert_eq!(full.foam_threshold, Some(0.5));

        let clamped = bare.with_fresnel(2.0, 3.0, -1.0);
        assert_eq!(clamped.base_reflectance, 1.0);
        assert_eq!(
            (clamped.reflection_strength, clamped.refraction_strength),
            (1.0, 0.0)
        );

        let tinted = bare.with_tint(Color::rgb(0.0, 0.2, 0.4), Color::rgb(0.3, 0.6, 0.5));
        assert_ne!(tinted.deep_color, bare.deep_color);
    }
}
