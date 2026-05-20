use crate::traits::double_sided_bit;
use crate::{
    AlphaMode, FEATURE_FLAT_SHADING, FEATURE_WORLD_SPACE, Material, PipelineAlphaMode, PipelineKey,
    ShaderKind,
};

/// Debug material that renders surface normals as RGB colors.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NormalMaterial {
    /// Output opacity.
    pub opacity: f32,
    /// Whether to use flat face normals.
    pub flat_shading: bool,
    /// Whether normals are displayed in world space instead of view space.
    pub world_space: bool,
    /// Whether the material is rendered double-sided.
    pub double_sided: bool,
}

impl NormalMaterial {
    /// Creates a default normal debug material.
    #[inline]
    pub const fn new() -> Self {
        Self {
            opacity: 1.0,
            flat_shading: false,
            world_space: false,
            double_sided: false,
        }
    }

    /// Returns this material with opacity.
    #[inline]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for NormalMaterial {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Material for NormalMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        let mut bits = double_sided_bit(self.double_sided);
        if self.flat_shading {
            bits |= FEATURE_FLAT_SHADING;
        }
        if self.world_space {
            bits |= FEATURE_WORLD_SPACE;
        }
        let alpha = if self.opacity < 1.0 {
            PipelineAlphaMode::Blend
        } else {
            PipelineAlphaMode::Opaque
        };
        PipelineKey::new(ShaderKind::Normal, alpha, bits)
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.opacity < 1.0
    }

    #[inline]
    fn double_sided(&self) -> bool {
        self.double_sided
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        AlphaMode::Opaque.cutoff()
    }
}
