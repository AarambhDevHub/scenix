//! Procedural sky evaluation (Rayleigh / Mie analytic approximation).
//!
//! The CPU model is a compact analytic approximation tuned for scenix
//! examples and background passes: it is deterministic, cheap (a handful of
//! trig/exp calls per sample), and matches the packed WGSL sky shader state
//! used by the renderer `environment` feature.

use scenix_math::Vec3;
use scenix_math::float::sqrt;

/// Procedural sky parameters shared by CPU evaluation and the GPU shader.
///
/// `turbidity` scales haze thickness; `rayleigh` scales air-scattering
/// brightness; `mie` scales the halo amplitude; `mie_g`
/// (usually `0.6..=0.99`) tightens the forward luster.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkyModel {
    /// Haze thickness factor, usually `2..=12`.
    pub turbidity: f32,
    /// Air-molecule luminance factor `0..=3`.
    pub rayleigh: f32,
    /// Mie scattering amplitude `0..=1`.
    pub mie: f32,
    /// Mie forward-scatter asymmetry `0..=1`.
    pub mie_g: f32,
}

impl SkyModel {
    /// A pleasant daytime default (clear sky, light haze).
    #[inline]
    pub const fn clear_day() -> Self {
        Self {
            turbidity: 4.0,
            rayleigh: 1.6,
            mie: 0.35,
            mie_g: 0.88,
        }
    }

    /// A dusty, low-contrast haze preset.
    #[inline]
    pub const fn overcast_haze() -> Self {
        Self {
            turbidity: 11.0,
            rayleigh: 0.7,
            mie: 0.75,
            mie_g: 0.72,
        }
    }

    /// Returns this model with a turbidity factor.
    #[inline]
    pub const fn with_turbidity(mut self, turbidity: f32) -> Self {
        self.turbidity = turbidity;
        self
    }

    /// Returns this model with a Rayleigh factor.
    #[inline]
    pub const fn with_rayleigh(mut self, rayleigh: f32) -> Self {
        self.rayleigh = rayleigh;
        self
    }

    /// Returns this model with the Mie factor pair `(mie, mie_g)`.
    #[inline]
    pub const fn with_mie(mut self, mie: f32, mie_g: f32) -> Self {
        self.mie = mie;
        self.mie_g = mie_g;
        self
    }

    /// Evaluates sky radiance (linear RGB scale) for a unit view direction
    /// with the sun at `sun_direction` (also unit-length).
    ///
    /// The model composes three parts: a Rayleigh zenith/horizon gradient
    /// scaled by view pitch, a turbidity-driven haze term that warms near
    /// the horizon, and a Mie halo around the sun. Below-horizon view
    /// directions clamp, so the sky itself never flips upside down — ground
    /// handling belongs to [`crate::skybox::GroundedSkybox`].
    #[must_use]
    pub fn rgb_at(&self, view: Vec3, sun_direction: Vec3) -> [f32; 3] {
        let zenith_view = view.y.clamp(0.0, 1.0);
        let sun_elevation = sun_direction.y.clamp(-1.0, 1.0);
        let day_factor = (1.0 + sun_elevation) * 0.5;

        // Rayleigh: air-scattering reads brighter toward the horizon.
        let falloff = sqrt(1.0 - zenith_view * zenith_view);
        let rayleigh = self.rayleigh.max(0.0);
        let air_brightness = (0.55 + 0.35 * rayleigh.min(3.0) / 3.0).max(0.2);
        let mut color = [
            (0.24 + 0.50 * falloff) * air_brightness,
            (0.32 + 0.58 * falloff) * air_brightness,
            (0.55 + 0.44 * falloff) * air_brightness,
        ];

        // Turbidity: haze warms and desaturates near the horizon.
        let haze = (self.turbidity.max(0.0) * 0.05) * (1.0 - zenith_view) * (1.0 - zenith_view);
        color[0] *= 1.0 + haze * 1.2;
        color[2] *= 1.0 - haze * 0.55;

        // Mie halo: bright glow tightening around the sun direction.
        let cos_view_sun = view.dot(sun_direction).clamp(0.0, 1.0);
        let g = self.mie_g.clamp(0.0, 0.999);
        let g_squared = g * g;
        let cos_view_sun_term = 1.0 + g_squared - 2.0 * g * cos_view_sun;
        let htype = (1.0 + 2.0 * g_squared) * (1.0 - g_squared)
            / (4.0 * core::f32::consts::PI * sqrt(cos_view_sun_term));
        let halo = (2.0
            * core::f32::consts::PI
            * self.mie.max(0.0)
            * htype.clamp(0.0, 16.0)
            * (1.0 - g * g)
            * (1.0 + g * g - g * cos_view_sun))
            * day_factor;
        color[0] += halo * 0.55;
        color[1] += halo * 0.85;
        color[2] += halo;

        // Luminance scaling: bright days, dim dawns, dark overcast.
        for item in &mut color {
            *item *= 0.35 + 0.85 * day_factor;
        }
        color
    }
}

#[cfg(test)]
mod tests {
    use scenix_math::Vec3;

    use super::SkyModel;

    #[test]
    fn zenith_reads_blue_horisont_reads_hazy() {
        let model = SkyModel::clear_day();
        let sun = Vec3::new(0.0, 0.94, 0.34).normalize();
        let zenith = model.rgb_at(Vec3::Y, sun);
        let horizon = model.rgb_at(Vec3::new(1.0, 0.0, 0.0), sun);
        assert!(zenith[2] > zenith[0] * 1.2, "zenith stays blue");
        assert!(horizon[0] > zenith[0], "horizon warms the reds");
        for value in zenith.iter().chain(horizon.iter()) {
            assert!(value.is_finite() && *value >= 0.0);
        }
    }

    #[test]
    fn viewing_the_sun_is_brighter_than_away() {
        let model = SkyModel::clear_day();
        let sun = Vec3::new(0.0, 0.55, 0.835).normalize();
        let toward_sun = model.rgb_at(sun, sun);
        let away = model.rgb_at(Vec3::new(0.0, 0.5, -1.0).normalize(), sun);
        let toward: f32 = toward_sun.iter().sum();
        let distance: f32 = away.iter().sum();
        assert!(toward > distance, "Mie halo must lift the glow");
    }

    #[test]
    fn presets_differ_in_turbidity_and_mie() {
        let clear = SkyModel::clear_day();
        let haste = SkyModel::overcast_haze();
        assert!(haste.turbidity > clear.turbidity);
        assert!(haste.mie > clear.mie);
        let tuned = clear
            .with_turbidity(7.0)
            .with_rayleigh(2.0)
            .with_mie(0.5, 0.9);
        assert_eq!(tuned.turbidity, 7.0);
    }
}
