use scenix_core::{Color, TextureId};

use crate::{
    FEATURE_CLEARCOAT, FEATURE_IRIDESCENCE, FEATURE_NORMAL_TEXTURE, FEATURE_SHEEN,
    FEATURE_TRANSMISSION, Material, PbrMaterial, PipelineAlphaMode, PipelineKey, ShaderKind,
};

/// PBR material with advanced physical surface effects.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicalMaterial {
    /// Base metallic-roughness material parameters.
    pub base: PbrMaterial,
    /// Clearcoat layer strength in `0.0..=1.0`.
    pub clearcoat: f32,
    /// Clearcoat layer roughness in `0.0..=1.0`.
    pub clearcoat_roughness: f32,
    /// Optional clearcoat normal texture.
    pub clearcoat_normal_texture: Option<TextureId>,
    /// Fabric-like sheen strength in `0.0..=1.0`.
    pub sheen: f32,
    /// Sheen color.
    pub sheen_color: Color,
    /// Sheen roughness in `0.0..=1.0`.
    pub sheen_roughness: f32,
    /// Glass-like transmission strength in `0.0..=1.0`.
    pub transmission: f32,
    /// Volume thickness used for transmission.
    pub thickness: f32,
    /// Index of refraction.
    pub ior: f32,
    /// Thin-film iridescence strength in `0.0..=1.0`.
    pub iridescence: f32,
    /// Thin-film iridescence IOR.
    pub iridescence_ior: f32,
}

impl PhysicalMaterial {
    /// Creates a default physical material.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this material with base PBR parameters.
    #[inline]
    pub fn base(mut self, base: PbrMaterial) -> Self {
        self.base = base;
        self
    }

    /// Returns this material with clearcoat parameters.
    #[inline]
    pub fn clearcoat(mut self, strength: f32, roughness: f32) -> Self {
        self.clearcoat = strength.clamp(0.0, 1.0);
        self.clearcoat_roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Returns this material with sheen parameters.
    #[inline]
    pub fn sheen(mut self, strength: f32, color: Color, roughness: f32) -> Self {
        self.sheen = strength.clamp(0.0, 1.0);
        self.sheen_color = color;
        self.sheen_roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Returns this material with transmission parameters.
    #[inline]
    pub fn transmission(mut self, strength: f32, thickness: f32) -> Self {
        self.transmission = strength.clamp(0.0, 1.0);
        self.thickness = thickness.max(0.0);
        self
    }

    /// Returns this material with iridescence parameters.
    #[inline]
    pub fn iridescence(mut self, strength: f32, ior: f32) -> Self {
        self.iridescence = strength.clamp(0.0, 1.0);
        self.iridescence_ior = ior.max(1.0);
        self
    }

    fn feature_bits(&self) -> u64 {
        let mut bits = self.base.feature_bits();
        if self.clearcoat > 0.0 {
            bits |= FEATURE_CLEARCOAT;
        }
        if self.clearcoat_normal_texture.is_some() {
            bits |= FEATURE_NORMAL_TEXTURE;
        }
        if self.sheen > 0.0 {
            bits |= FEATURE_SHEEN;
        }
        if self.transmission > 0.0 {
            bits |= FEATURE_TRANSMISSION;
        }
        if self.iridescence > 0.0 {
            bits |= FEATURE_IRIDESCENCE;
        }
        bits
    }
}

impl Default for PhysicalMaterial {
    #[inline]
    fn default() -> Self {
        Self {
            base: PbrMaterial::default(),
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            clearcoat_normal_texture: None,
            sheen: 0.0,
            sheen_color: Color::WHITE,
            sheen_roughness: 1.0,
            transmission: 0.0,
            thickness: 0.0,
            ior: 1.5,
            iridescence: 0.0,
            iridescence_ior: 1.3,
        }
    }
}

impl Material for PhysicalMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        let alpha = if self.transmission > 0.0 {
            PipelineAlphaMode::Blend
        } else {
            self.base.alpha_mode.pipeline_alpha()
        };
        PipelineKey::new(ShaderKind::Physical, alpha, self.feature_bits())
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.base.alpha_mode.is_transparent() || self.transmission > 0.0
    }

    #[inline]
    fn double_sided(&self) -> bool {
        self.base.double_sided
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        if self.transmission > 0.0 {
            None
        } else {
            self.base.alpha_mode.cutoff()
        }
    }
}
