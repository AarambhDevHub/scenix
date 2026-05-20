use alloc::string::String;

use scenix_core::{Color, TextureId};
use scenix_math::Vec3;

use crate::traits::{
    AlphaMode, FEATURE_ALBEDO_TEXTURE, FEATURE_EMISSIVE_TEXTURE,
    FEATURE_METALLIC_ROUGHNESS_TEXTURE, FEATURE_NORMAL_TEXTURE, FEATURE_OCCLUSION_TEXTURE,
    Material, PipelineKey, ShaderKind, double_sided_bit, option_texture_bit,
};

/// Metallic-roughness physically based material.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PbrMaterial {
    /// Human-readable material name.
    pub name: String,
    /// Base color in linear RGBA.
    pub albedo: Color,
    /// Optional base color texture.
    pub albedo_texture: Option<TextureId>,
    /// Metallic factor, where 0 is dielectric and 1 is metal.
    pub metallic: f32,
    /// Roughness factor, where 0 is mirror-like and 1 is matte.
    pub roughness: f32,
    /// Optional packed metallic-roughness texture.
    pub metallic_roughness_texture: Option<TextureId>,
    /// Optional tangent-space normal map.
    pub normal_texture: Option<TextureId>,
    /// Optional ambient occlusion texture.
    pub occlusion_texture: Option<TextureId>,
    /// Emissive RGB color in linear space.
    pub emissive: Vec3,
    /// Optional emissive texture.
    pub emissive_texture: Option<TextureId>,
    /// Alpha behavior.
    pub alpha_mode: AlphaMode,
    /// Whether the material is rendered double-sided.
    pub double_sided: bool,
}

impl PbrMaterial {
    /// Creates a default opaque white dielectric material.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this material with a name.
    #[inline]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Returns this material with a base color.
    #[inline]
    pub const fn albedo(mut self, albedo: Color) -> Self {
        self.albedo = albedo;
        self
    }

    /// Returns this material with metallic and roughness factors.
    #[inline]
    pub fn metallic_roughness(mut self, metallic: f32, roughness: f32) -> Self {
        self.metallic = metallic.clamp(0.0, 1.0);
        self.roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Returns this material with alpha behavior.
    #[inline]
    pub const fn alpha_mode(mut self, alpha_mode: AlphaMode) -> Self {
        self.alpha_mode = alpha_mode;
        self
    }

    /// Returns this material with double-sided rendering enabled or disabled.
    #[inline]
    pub const fn double_sided(mut self, double_sided: bool) -> Self {
        self.double_sided = double_sided;
        self
    }

    pub(crate) fn feature_bits(&self) -> u64 {
        double_sided_bit(self.double_sided)
            | option_texture_bit(&self.albedo_texture, FEATURE_ALBEDO_TEXTURE)
            | option_texture_bit(
                &self.metallic_roughness_texture,
                FEATURE_METALLIC_ROUGHNESS_TEXTURE,
            )
            | option_texture_bit(&self.normal_texture, FEATURE_NORMAL_TEXTURE)
            | option_texture_bit(&self.occlusion_texture, FEATURE_OCCLUSION_TEXTURE)
            | option_texture_bit(&self.emissive_texture, FEATURE_EMISSIVE_TEXTURE)
    }
}

impl Default for PbrMaterial {
    #[inline]
    fn default() -> Self {
        Self {
            name: String::new(),
            albedo: Color::WHITE,
            albedo_texture: None,
            metallic: 0.0,
            roughness: 1.0,
            metallic_roughness_texture: None,
            normal_texture: None,
            occlusion_texture: None,
            emissive: Vec3::ZERO,
            emissive_texture: None,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        }
    }
}

impl Material for PbrMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        PipelineKey::new(
            ShaderKind::Pbr,
            self.alpha_mode.pipeline_alpha(),
            self.feature_bits(),
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
