use alloc::{string::String, vec::Vec};

use scenix_core::TextureId;

use crate::traits::{double_sided_bit, stable_shader_id};
use crate::{FEATURE_CUSTOM_TEXTURES, Material, PipelineAlphaMode, PipelineKey, ShaderKind};

/// Custom WGSL material.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShaderMaterial {
    /// Human-readable material name.
    pub name: String,
    /// Vertex shader WGSL source.
    pub vertex_wgsl: String,
    /// Fragment shader WGSL source.
    pub fragment_wgsl: String,
    /// Raw uniform buffer bytes owned by the application.
    pub uniforms: Vec<u8>,
    /// Texture IDs referenced by the custom shader.
    pub textures: Vec<TextureId>,
    /// Whether the shader needs transparent sorting and blending.
    pub transparent: bool,
    /// Whether the material is rendered double-sided.
    pub double_sided: bool,
    /// Optional alpha-test cutoff.
    pub alpha_cutoff: Option<f32>,
}

impl ShaderMaterial {
    /// Creates a shader material from WGSL sources.
    #[inline]
    pub fn new(vertex_wgsl: impl Into<String>, fragment_wgsl: impl Into<String>) -> Self {
        Self {
            name: String::new(),
            vertex_wgsl: vertex_wgsl.into(),
            fragment_wgsl: fragment_wgsl.into(),
            uniforms: Vec::new(),
            textures: Vec::new(),
            transparent: false,
            double_sided: false,
            alpha_cutoff: None,
        }
    }

    /// Returns the stable source-derived shader ID used by `PipelineKey`.
    #[inline]
    pub fn shader_id(&self) -> u64 {
        stable_shader_id(&self.vertex_wgsl, &self.fragment_wgsl)
    }

    /// Returns this material with transparent rendering enabled or disabled.
    #[inline]
    pub const fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }
}

impl Default for ShaderMaterial {
    #[inline]
    fn default() -> Self {
        Self::new("", "")
    }
}

impl Material for ShaderMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        let alpha = if self.transparent {
            PipelineAlphaMode::Blend
        } else if self.alpha_cutoff.is_some() {
            PipelineAlphaMode::Mask
        } else {
            PipelineAlphaMode::Opaque
        };
        let mut bits = double_sided_bit(self.double_sided);
        if !self.textures.is_empty() {
            bits |= FEATURE_CUSTOM_TEXTURES;
        }
        PipelineKey::new(ShaderKind::Custom(self.shader_id()), alpha, bits)
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.transparent
    }

    #[inline]
    fn double_sided(&self) -> bool {
        self.double_sided
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        self.alpha_cutoff
    }
}
