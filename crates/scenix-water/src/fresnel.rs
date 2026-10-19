//! Schlick fresnel reflectance for water and wet surfaces.

/// Fresnel reflectance parameters (Schlick approximation).
///
/// `base_reflectance` (`F0`) is the reflectance at a grazing-normal angle;
/// water sits near `0.02`, glass near `0.04`, and metals far higher.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FresnelParams {
    /// Reflectance at zero incidence, `0..=1`.
    pub base_reflectance: f32,
    /// Optional roughness tint applied to the reflected lobe; `1.0` keeps
    /// the reflection glossy per the Schlick curve itself.
    pub sheen: f32,
}

impl FresnelParams {
    /// Creates the classic Schlick setup with a sheen factor.
    #[inline]
    #[must_use]
    pub const fn new(base_reflectance: f32) -> Self {
        Self {
            base_reflectance: base_reflectance.clamp(0.0, 1.0),
            sheen: 1.0,
        }
    }

    /// Water default: the refraction index -> `~0.02` reflectance.
    #[inline]
    #[must_use]
    pub const fn water() -> Self {
        Self::new(0.020_371)
    }

    /// Returns these parameters locked to a fixed sheen loss `0..=1`.
    #[inline]
    #[must_use]
    pub const fn with_sheen(mut self, sheen: f32) -> Self {
        self.sheen = sheen.clamp(0.0, 1.0);
        self
    }

    /// Schlick's `R = F0 + (1 - F0)(1 - cosθ)^5`.
    ///
    /// `cos_theta` clamps into `[0, 1]` — pass `dot(normal, view)` clamped
    /// already for grazing surfaces.
    #[must_use]
    #[inline]
    pub fn reflectance(&self, cos_theta: f32) -> f32 {
        let f0 = self.base_reflectance.clamp(0.0, 1.0);
        let cosine = cos_theta.clamp(0.0, 1.0);
        let reciprocal = 1.0 - cosine;
        let power = reciprocal * reciprocal * reciprocal * reciprocal * reciprocal;
        (f0 + (1.0 - f0) * power) * self.sheen
    }
}

#[cfg(test)]
mod tests {
    use super::FresnelParams;

    #[test]
    fn schlick_curve_rises_to_grazing_angles() {
        let water = FresnelParams::water();
        let straight = water.reflectance(1.0);
        assert!((straight - water.base_reflectance).abs() < 1.0e-6);
        let grazing = water.reflectance(0.02);
        assert!(grazing > 0.5, "grazing angles approach mirror reflections");
        assert!(grazing <= 1.0);
        // Sheen fades the reflection linearly.
        assert!((water.with_sheen(0.0).reflectance(0.9)).abs() < 1.0e-6);
    }
}
