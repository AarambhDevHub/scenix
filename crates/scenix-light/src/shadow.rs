use scenix_core::ValidationError;

/// Shadow-map configuration shared by shadow-casting light types.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShadowConfig {
    /// Shadow map width and height in texels.
    pub map_size: u32,
    /// Near clip distance for the shadow camera.
    pub near: f32,
    /// Far clip distance for the shadow camera.
    pub far: f32,
    /// Depth bias used to reduce shadow acne.
    pub bias: f32,
    /// PCF kernel radius in texels. `0` means hard shadows.
    pub pcf_radius: u32,
    /// Cascade count for directional lights. Valid range is `1..=4`.
    pub cascades: u8,
}

impl ShadowConfig {
    /// Creates a shadow config with conservative defaults.
    #[inline]
    pub const fn new(map_size: u32, near: f32, far: f32) -> Self {
        Self {
            map_size,
            near,
            far,
            bias: 0.0005,
            pcf_radius: 1,
            cascades: 1,
        }
    }

    /// Returns this config with a depth bias.
    #[inline]
    pub const fn bias(mut self, bias: f32) -> Self {
        self.bias = bias;
        self
    }

    /// Returns this config with a PCF radius.
    #[inline]
    pub const fn pcf_radius(mut self, pcf_radius: u32) -> Self {
        self.pcf_radius = pcf_radius;
        self
    }

    /// Returns this config with a cascade count.
    #[inline]
    pub const fn cascades(mut self, cascades: u8) -> Self {
        self.cascades = cascades;
        self
    }

    /// Validates map size, clip planes, and cascade count.
    pub const fn validate(self) -> Result<(), ValidationError> {
        if self.map_size == 0 || !self.map_size.is_power_of_two() {
            return Err(ValidationError::OutOfRange);
        }
        if self.near <= 0.0 || self.far <= self.near {
            return Err(ValidationError::OutOfRange);
        }
        if self.cascades == 0 || self.cascades > 4 {
            return Err(ValidationError::OutOfRange);
        }
        Ok(())
    }
}

impl Default for ShadowConfig {
    #[inline]
    fn default() -> Self {
        Self::new(1024, 0.1, 100.0)
    }
}
