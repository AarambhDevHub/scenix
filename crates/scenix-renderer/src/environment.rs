use scenix_core::{LightId, TextureId};

/// Image-based-lighting environment descriptor.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnvironmentMap {
    /// Registered cube texture used as the environment source.
    pub texture_id: TextureId,
    /// Scalar intensity applied by renderer lighting.
    pub intensity: f32,
    /// Optional registered light probe used for diffuse irradiance.
    pub light_probe: Option<LightId>,
}

impl EnvironmentMap {
    /// Creates an environment map descriptor with intensity `1.0`.
    #[inline]
    pub const fn new(texture_id: TextureId) -> Self {
        Self {
            texture_id,
            intensity: 1.0,
            light_probe: None,
        }
    }

    /// Returns this descriptor with an intensity.
    #[inline]
    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.max(0.0);
        self
    }

    /// Returns this descriptor with a light probe.
    #[inline]
    pub const fn light_probe(mut self, light_probe: LightId) -> Self {
        self.light_probe = Some(light_probe);
        self
    }
}
