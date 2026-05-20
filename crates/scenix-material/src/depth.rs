use crate::traits::double_sided_bit;
use crate::{AlphaMode, Material, PipelineKey, ShaderKind};

/// Depth-only material used by shadow and prepass renderers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepthMaterial {
    /// Alpha behavior for alpha-tested depth passes.
    pub alpha_mode: AlphaMode,
    /// Whether the material is rendered double-sided.
    pub double_sided: bool,
}

impl DepthMaterial {
    /// Creates a default opaque depth material.
    #[inline]
    pub const fn new() -> Self {
        Self {
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        }
    }
}

impl Default for DepthMaterial {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Material for DepthMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        PipelineKey::new(
            ShaderKind::Depth,
            self.alpha_mode.pipeline_alpha(),
            double_sided_bit(self.double_sided),
        )
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        false
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
