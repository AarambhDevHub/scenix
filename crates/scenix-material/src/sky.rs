//! Sky background material descriptor (scenix v1.8.0).
//!
//! A [`SkyMaterial`] packs the gradient plus sun state the renderer's
//! `sky_dome` WGSL pipeline consumes: procedural atmosphere evaluation
//! (`FEATURE_ATMOSPHERE`) or a grounded gradient fallback, with an optional
//! sun glare disk. Sun light data originates in `scenix-light::SunLight` /
//! `scenix_sky::SunPosition`; the material stays light-free and carries
//! only the visual pieces.

use scenix_core::Color;
use scenix_math::Vec3;

use crate::traits::double_sided_bit;
use crate::{FEATURE_ATMOSPHERE, Material, PipelineAlphaMode, PipelineKey, ShaderKind};

/// Sky background material.
///
/// The gradient bookends land at `horizon_color` for `y == 0`; above it the
/// sky lerps to `top_color`, below it to `ground_color` (the grounded
/// skybox convention from `scenix_sky::GroundedSkybox`).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkyMaterial {
    /// Zenith color.
    pub top_color: Color,
    /// Horizon color.
    pub horizon_color: Color,
    /// Below-horizon ground color.
    pub ground_color: Color,
    /// Unit sun direction; drives the glare disk position.
    pub sun_direction: Vec3,
    /// Sun glare tint.
    pub sun_color: Color,
    /// Glare halo strength `0..=1` (`0` disables the sun disk path).
    pub glare_strength: f32,
    /// Normalized `y` where the grounded fade starts (usually negative).
    pub fade_start: f32,
    /// Grounded fade width in normalized units.
    pub fade_range: f32,
    /// Whether the procedural atmosphere path runs (vs. flat gradient).
    pub atmosphere: bool,
    /// Whether the dome renders back faces (sky spheres are usually
    /// viewed from inside).
    pub double_sided: bool,
}

impl SkyMaterial {
    /// A palatable default sky with the atmosphere path on.
    #[inline]
    #[must_use]
    pub fn clear_day() -> Self {
        Self {
            top_color: Color::rgb(0.10, 0.30, 0.78),
            horizon_color: Color::rgb(0.78, 0.70, 0.55),
            ground_color: Color::rgb(0.15, 0.14, 0.13),
            sun_direction: Vec3::new(0.35, 0.62, 0.71),
            sun_color: Color::rgb(1.0, 0.92, 0.72),
            glare_strength: 0.55,
            fade_start: -0.02,
            fade_range: 0.12,
            atmosphere: true,
            double_sided: true,
        }
    }

    /// Returns the sky with gradient bookends `(top, horizon, ground)`.
    #[inline]
    #[must_use]
    pub const fn with_gradient(
        mut self,
        top_color: Color,
        horizon_color: Color,
        ground_color: Color,
    ) -> Self {
        self.top_color = top_color;
        self.horizon_color = horizon_color;
        self.ground_color = ground_color;
        self
    }

    /// Returns the sky with sun glare parameters `(direction, color,
    /// strength)`.
    #[inline]
    #[must_use]
    pub const fn with_sun(
        mut self,
        sun_direction: Vec3,
        sun_color: Color,
        glare_strength: f32,
    ) -> Self {
        self.sun_direction = sun_direction;
        self.sun_color = sun_color;
        self.glare_strength = glare_strength.clamp(0.0, 1.0);
        self
    }

    /// Returns the sky with a custom grounded fade band.
    #[inline]
    #[must_use]
    pub const fn with_fade(mut self, fade_start: f32, fade_range: f32) -> Self {
        self.fade_start = fade_start;
        self.fade_range = fade_range;
        self
    }

    /// Returns the sky toggling the procedural atmosphere path.
    #[inline]
    #[must_use]
    pub const fn with_atmosphere(mut self, atmosphere: bool) -> Self {
        self.atmosphere = atmosphere;
        self
    }

    /// Returns the sky with explicit back-face culling.
    #[inline]
    #[must_use]
    pub const fn with_double_sided(mut self, double_sided: bool) -> Self {
        self.double_sided = double_sided;
        self
    }
}

impl Material for SkyMaterial {
    fn pipeline_key(&self) -> PipelineKey {
        let mut bits = double_sided_bit(self.double_sided);
        if self.atmosphere {
            bits |= FEATURE_ATMOSPHERE;
        }
        PipelineKey::new(ShaderKind::Sky, PipelineAlphaMode::Opaque, bits)
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
        None
    }
}

#[cfg(test)]
mod tests {
    use super::SkyMaterial;
    use crate::{FEATURE_ATMOSPHERE, Material, ShaderKind};
    use scenix_core::Color;
    use scenix_math::Vec3;

    #[test]
    fn sky_pipeline_routes_atmosphere_and_gradient() {
        let sky = SkyMaterial::clear_day();
        assert_eq!(sky.pipeline_key().shader, ShaderKind::Sky);
        assert!(sky.pipeline_key().has_feature(FEATURE_ATMOSPHERE));
        assert!(!sky.is_transparent());
        assert!(sky.double_sided());
        assert_eq!(sky.alpha_cutoff(), None);

        let flat = sky.with_atmosphere(false);
        assert!(!flat.pipeline_key().has_feature(FEATURE_ATMOSPHERE));

        let tuned = sky.with_gradient(
            Color::BLACK,
            Color::rgb(0.6, 0.6, 0.6),
            Color::rgb(0.1, 0.1, 0.1),
        );
        assert_eq!(tuned.top_color, Color::BLACK);
        let clamped = sky.with_sun(Vec3::new(0.0, 1.0, 0.0), Color::WHITE, 9.0);
        assert_eq!(clamped.glare_strength, 1.0);
        let faded = sky.with_fade(-0.3, 0.4);
        assert_eq!((faded.fade_start, faded.fade_range), (-0.3, 0.4));
    }
}
