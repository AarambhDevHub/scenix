/// Material alpha behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlphaMode {
    /// Fully opaque material.
    Opaque,
    /// Alpha-tested material using the stored cutoff threshold.
    Mask(f32),
    /// Alpha-blended material.
    Blend,
}

impl AlphaMode {
    /// Returns the pipeline-level alpha mode.
    #[inline]
    pub const fn pipeline_alpha(self) -> PipelineAlphaMode {
        match self {
            Self::Opaque => PipelineAlphaMode::Opaque,
            Self::Mask(_) => PipelineAlphaMode::Mask,
            Self::Blend => PipelineAlphaMode::Blend,
        }
    }

    /// Returns whether this mode needs transparent sorting and blending.
    #[inline]
    pub const fn is_transparent(self) -> bool {
        matches!(self, Self::Blend)
    }

    /// Returns the alpha-test cutoff for masked materials.
    #[inline]
    pub const fn cutoff(self) -> Option<f32> {
        match self {
            Self::Mask(cutoff) => Some(cutoff),
            Self::Opaque | Self::Blend => None,
        }
    }
}

impl Default for AlphaMode {
    #[inline]
    fn default() -> Self {
        Self::Opaque
    }
}

/// Pipeline-level alpha mode. This is hashable because the cutoff value is not
/// part of render-pipeline selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PipelineAlphaMode {
    /// Fully opaque pipeline.
    #[default]
    Opaque,
    /// Alpha-tested pipeline.
    Mask,
    /// Alpha-blended pipeline.
    Blend,
}

/// Built-in shader family used by a material.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ShaderKind {
    /// Physically based metallic-roughness shader.
    #[default]
    Pbr,
    /// Advanced physical surface shader.
    Physical,
    /// Constant-color unlit shader.
    Unlit,
    /// Diffuse Lambert shader.
    Lambert,
    /// Cel/toon shader.
    Toon,
    /// Surface-normal debug shader.
    Normal,
    /// Wireframe shader.
    Wireframe,
    /// Depth-only shader.
    Depth,
    /// Line shader.
    Line,
    /// Point-sprite shader.
    Points,
    /// User-provided WGSL shader sources, identified by a stable hash.
    Custom(u64),
}

/// Compact renderer pipeline selector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PipelineKey {
    /// Shader family.
    pub shader: ShaderKind,
    /// Pipeline-level alpha mode.
    pub alpha: PipelineAlphaMode,
    /// Material feature flags.
    pub feature_bits: u64,
}

impl PipelineKey {
    /// Creates a pipeline key.
    #[inline]
    pub const fn new(shader: ShaderKind, alpha: PipelineAlphaMode, feature_bits: u64) -> Self {
        Self {
            shader,
            alpha,
            feature_bits,
        }
    }

    /// Returns a key with one feature flag enabled.
    #[inline]
    pub const fn with_feature(mut self, feature: u64) -> Self {
        self.feature_bits |= feature;
        self
    }

    /// Returns whether the feature flag is enabled.
    #[inline]
    pub const fn has_feature(self, feature: u64) -> bool {
        self.feature_bits & feature != 0
    }
}

/// Material is rendered double-sided.
pub const FEATURE_DOUBLE_SIDED: u64 = 1 << 0;
/// Base color texture is bound.
pub const FEATURE_ALBEDO_TEXTURE: u64 = 1 << 1;
/// Metallic-roughness texture is bound.
pub const FEATURE_METALLIC_ROUGHNESS_TEXTURE: u64 = 1 << 2;
/// Normal map is bound.
pub const FEATURE_NORMAL_TEXTURE: u64 = 1 << 3;
/// Occlusion texture is bound.
pub const FEATURE_OCCLUSION_TEXTURE: u64 = 1 << 4;
/// Emissive texture is bound.
pub const FEATURE_EMISSIVE_TEXTURE: u64 = 1 << 5;
/// Gradient/ramp texture is bound.
pub const FEATURE_GRADIENT_TEXTURE: u64 = 1 << 6;
/// Clearcoat lobe is active.
pub const FEATURE_CLEARCOAT: u64 = 1 << 7;
/// Sheen lobe is active.
pub const FEATURE_SHEEN: u64 = 1 << 8;
/// Transmission path is active.
pub const FEATURE_TRANSMISSION: u64 = 1 << 9;
/// Iridescence path is active.
pub const FEATURE_IRIDESCENCE: u64 = 1 << 10;
/// Flat normal shading is active.
pub const FEATURE_FLAT_SHADING: u64 = 1 << 11;
/// Normals are evaluated in world space.
pub const FEATURE_WORLD_SPACE: u64 = 1 << 12;
/// Wireframe rendering path is active.
pub const FEATURE_WIREFRAME: u64 = 1 << 13;
/// Dashed line path is active.
pub const FEATURE_DASHED: u64 = 1 << 14;
/// Point size attenuation is active.
pub const FEATURE_SIZE_ATTENUATION: u64 = 1 << 15;
/// Custom shader has texture bindings.
pub const FEATURE_CUSTOM_TEXTURES: u64 = 1 << 16;
/// Material expects vertex colors.
pub const FEATURE_VERTEX_COLORS: u64 = 1 << 17;
/// Toon outline path is active.
pub const FEATURE_OUTLINE: u64 = 1 << 18;

/// CPU-side material description with no GPU dependency.
pub trait Material: Send + Sync + 'static {
    /// Returns the renderer pipeline selector for this material state.
    fn pipeline_key(&self) -> PipelineKey;

    /// Returns whether the material should be rendered in a transparent path.
    fn is_transparent(&self) -> bool;

    /// Returns whether back-face culling should be disabled.
    fn double_sided(&self) -> bool;

    /// Returns the alpha-test cutoff for masked materials.
    fn alpha_cutoff(&self) -> Option<f32>;
}

#[inline]
pub(crate) const fn double_sided_bit(double_sided: bool) -> u64 {
    if double_sided {
        FEATURE_DOUBLE_SIDED
    } else {
        0
    }
}

#[inline]
pub(crate) const fn option_texture_bit<T>(texture: &Option<T>, feature: u64) -> u64 {
    if texture.is_some() { feature } else { 0 }
}

/// Stable FNV-1a hash for shader source identity.
#[inline]
pub(crate) fn stable_shader_id(vertex_wgsl: &str, fragment_wgsl: &str) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET;
    for byte in vertex_wgsl.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash ^= 0xff;
    hash = hash.wrapping_mul(PRIME);
    for byte in fragment_wgsl.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}
