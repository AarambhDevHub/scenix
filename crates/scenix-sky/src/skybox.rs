//! Grounded skyboxes: a gradient hemisphere whose lower half blends into a
//! ground color, so a camera floating above terrain never sees through the
//! geometry seams below the horizon.
//!
//! Grounded skyboxes pair well with [`crate::SkyModel`]: the procedural sky
//! covers the upper dome while the grounded fade fills the gap between the
//! terrain skirt and the horizon line.

use alloc::vec::Vec;

use scenix_core::Color;
use scenix_math::Vec3;
use scenix_math::float::{cos, sin};

/// Gradient sky sphere with a grounded fade.
///
/// The upper hemisphere lerp runs from `horizon_color` (at `y == fade_start`)
/// to `top_color` (at `y == 1.0`); below `fade_start` the color cross-fades
/// into `ground_color` across `fade_range` normalized units.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroundedSkybox {
    /// Zenith color.
    pub top_color: Color,
    /// Color right at the horizon line.
    pub horizon_color: Color,
    /// Color used below the horizon (ground bounce approximation).
    pub ground_color: Color,
    /// Normalized `y` where the grounded fade starts (usually negative).
    pub fade_start: f32,
    /// Normalized fade distance before the horizon when approaching from
    /// above (usually `0.0..=0.3`).
    pub fade_range: f32,
}

impl GroundedSkybox {
    /// Creates a grounded skybox with defaults clamped into sane ranges.
    #[must_use]
    pub fn new(top_color: Color, horizon_color: Color, ground_color: Color) -> Self {
        Self {
            top_color,
            horizon_color,
            ground_color,
            fade_start: -0.02,
            fade_range: 0.12,
        }
    }

    /// Returns this skybox with a custom grounded fade band.
    #[inline]
    #[must_use]
    pub const fn with_fade(mut self, fade_start: f32, fade_range: f32) -> Self {
        self.fade_start = fade_start;
        self.fade_range = fade_range;
        self
    }

    /// Evaluates the gradient at a unit direction.
    #[must_use]
    pub fn rgb_at(&self, direction: Vec3) -> [f32; 3] {
        let y = direction.y.clamp(-1.0, 1.0);
        let fade_range = self.fade_range.clamp(0.001, 1.0);
        if y >= self.fade_start {
            let above = ((y - self.fade_start) / fade_range).clamp(0.0, 1.0);
            let above = above * above * (3.0 - 2.0 * above);
            let blend = self.horizon_color.lerp(self.top_color, above);
            return [blend.r, blend.g, blend.b];
        }
        let below = ((self.fade_start - y) / fade_range).clamp(0.0, 1.0);
        let blend = self.horizon_color.lerp(self.ground_color, below);
        [blend.r, blend.g, blend.b]
    }

    /// Samples a small dome of directions — handy for quick editor previews
    /// or renderer background validation (row-major, zenith first).
    #[must_use]
    pub fn sample_dome(&self, width: u32, height: u32) -> Vec<[f32; 3]> {
        let width = width.max(2);
        let height = height.max(2);
        let mut samples = Vec::with_capacity((width * height) as usize);
        for row in 0..height {
            let pitch = (row as f32 + 0.5) / height as f32;
            let latitude = core::f32::consts::FRAC_PI_2 - pitch * core::f32::consts::PI;
            for column in 0..width {
                let longitude = column as f32 / width as f32 * core::f32::consts::TAU;
                let horizontal = cos(latitude).max(0.0);
                samples.push(self.rgb_at(Vec3::new(
                    horizontal * cos(longitude),
                    sin(latitude),
                    horizontal * sin(longitude),
                )));
            }
        }
        samples
    }
}

#[cfg(test)]
mod tests {
    use scenix_core::Color;
    use scenix_math::Vec3;

    use super::GroundedSkybox;

    #[test]
    fn gradient_blends_top_horizon_and_ground() {
        let skybox = GroundedSkybox::new(
            Color::rgb(0.10, 0.30, 0.80),
            Color::rgb(0.90, 0.70, 0.45),
            Color::rgb(0.16, 0.14, 0.12),
        );
        let top = skybox.rgb_at(Vec3::Y);
        let horizon = skybox.rgb_at(Vec3::new(1.0, 0.0, 0.0));
        let ground = skybox.rgb_at(Vec3::new(0.0, -1.0, 0.0));

        assert!(
            (top[2] - 0.80).abs() < 1.0e-5,
            "zenith stays near the top color"
        );
        assert!(horizon[0] > 0.7, "horizon leans warm");
        assert!(ground[2] < 0.2, "ground fade closes into the ground color");
    }

    #[test]
    fn dome_sampling_is_order_preserving() {
        let skybox = GroundedSkybox::new(
            Color::rgb(0.1, 0.2, 0.7),
            Color::rgb(0.8, 0.6, 0.4),
            Color::rgb(0.1, 0.1, 0.1),
        )
        .with_fade(-0.05, 0.1);
        let samples = skybox.sample_dome(4, 2);
        assert_eq!(samples.len(), 8);
        // Row 0 (zenith band) stays blue; row 1 low band drifts warm.
        assert!(samples[0][2] > samples[samples.len() - 1][2]);
    }
}
