/// Texture filtering mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FilterMode {
    /// Nearest-neighbor sampling.
    Nearest,
    /// Linear interpolation.
    #[default]
    Linear,
}

/// Texture coordinate addressing mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AddressMode {
    /// Repeat outside `[0, 1]`.
    Repeat,
    /// Mirror every repeated interval.
    MirrorRepeat,
    /// Clamp coordinates to the edge texels.
    #[default]
    ClampToEdge,
}

/// Optional depth-compare function for shadow/depth sampling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CompareFunction {
    /// Passes when the sampled value is less than the reference.
    Less,
    /// Passes when the sampled value is less than or equal to the reference.
    LessEqual,
    /// Passes when the sampled value is greater than the reference.
    Greater,
    /// Passes when the sampled value is greater than or equal to the reference.
    GreaterEqual,
    /// Passes when the sampled value equals the reference.
    Equal,
    /// Passes when the sampled value differs from the reference.
    NotEqual,
    /// Always passes.
    Always,
    /// Never passes.
    Never,
}

/// CPU-side sampler configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sampler {
    /// Magnification filter.
    pub mag_filter: FilterMode,
    /// Minification filter.
    pub min_filter: FilterMode,
    /// Mipmap filter.
    pub mip_filter: FilterMode,
    /// Addressing for U coordinates.
    pub address_u: AddressMode,
    /// Addressing for V coordinates.
    pub address_v: AddressMode,
    /// Addressing for W coordinates.
    pub address_w: AddressMode,
    /// Anisotropy level clamped to `1..=16`.
    pub anisotropy: u8,
    /// Optional depth compare function.
    pub compare: Option<CompareFunction>,
}

impl Sampler {
    /// Creates a default linear clamp sampler.
    #[inline]
    pub const fn new() -> Self {
        Self {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mip_filter: FilterMode::Linear,
            address_u: AddressMode::ClampToEdge,
            address_v: AddressMode::ClampToEdge,
            address_w: AddressMode::ClampToEdge,
            anisotropy: 1,
            compare: None,
        }
    }

    /// Returns this sampler with filter modes set.
    #[inline]
    pub const fn filters(
        mut self,
        mag_filter: FilterMode,
        min_filter: FilterMode,
        mip_filter: FilterMode,
    ) -> Self {
        self.mag_filter = mag_filter;
        self.min_filter = min_filter;
        self.mip_filter = mip_filter;
        self
    }

    /// Returns this sampler with address modes set.
    #[inline]
    pub const fn address_modes(
        mut self,
        address_u: AddressMode,
        address_v: AddressMode,
        address_w: AddressMode,
    ) -> Self {
        self.address_u = address_u;
        self.address_v = address_v;
        self.address_w = address_w;
        self
    }

    /// Returns this sampler with anisotropy clamped to `1..=16`.
    #[inline]
    pub const fn anisotropy(mut self, anisotropy: u8) -> Self {
        self.anisotropy = if anisotropy < 1 {
            1
        } else if anisotropy > 16 {
            16
        } else {
            anisotropy
        };
        self
    }

    /// Returns this sampler with a compare function.
    #[inline]
    pub const fn compare(mut self, compare: Option<CompareFunction>) -> Self {
        self.compare = compare;
        self
    }
}

impl Default for Sampler {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
