//! Underwater view tinting: per-meter absorption, murk fog, and depth
//! falloff for cameras below the water plane.

use scenix_core::Color;
use scenix_math::float::exp;

/// Underwater visual state.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnderwaterView {
    /// Absorption rates (per meter) for red/green/blue light.
    pub absorption: [f32; 3],
    /// Target murk/fog color (already linear light).
    pub murk_color: Color,
    /// Murk fog density (1/m) shaped by view distance.
    pub murk_density: f32,
}

impl UnderwaterView {
    /// Clear swimmable water (blue-green, low murk).
    #[inline]
    #[must_use]
    pub const fn clear_pool() -> Self {
        Self {
            absorption: [0.55, 0.12, 0.08],
            murk_color: Color::rgb(0.02, 0.18, 0.22),
            murk_density: 0.08,
        }
    }

    /// Murky lake or river water.
    #[inline]
    #[must_use]
    pub const fn murky_river() -> Self {
        Self {
            absorption: [0.80, 0.35, 0.22],
            murk_color: Color::rgb(0.10, 0.16, 0.10),
            murk_density: 0.35,
        }
    }

    /// Returns the view with custom murk density `0..=1.5`.
    #[inline]
    #[must_use]
    pub fn with_murk(mut self, murk_density: f32) -> Self {
        self.murk_density = murk_density.clamp(0.0, 1.5);
        self
    }

    /// Transmittance over a straight light path `distance` meters.
    #[must_use]
    pub fn transmittance(&self, distance: f32) -> [f32; 3] {
        let span = distance.max(0.0);
        [
            exp(-self.absorption[0].max(0.0) * span),
            exp(-self.absorption[1].max(0.0) * span),
            exp(-self.absorption[2].max(0.0) * span),
        ]
    }

    /// Tints a surface color for a camera `distance` meters away below the
    /// water line, blending toward murk as `distance` grows.
    #[must_use]
    pub fn view_tint(&self, surface_color: Color, distance: f32) -> [f32; 3] {
        let transmittance = self.transmittance(distance);
        // Murk halls in with distance fraction.
        let murk = 1.0 - exp(-self.murk_density * distance.max(0.0));
        let color = surface_color.to_array();
        let mut result = [0.0_f32; 3];
        for (index, channel) in result.iter_mut().enumerate() {
            *channel =
                color[index] * transmittance[index] + self.murk_color.to_array()[index] * murk;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::UnderwaterView;
    use scenix_core::Color;

    #[test]
    fn transmittance_fades_red_first() {
        let view = UnderwaterView::clear_pool();
        let faded = view.transmittance(2.0);
        assert!(faded[0] < faded[1] && faded[1] < faded[2]);
        assert!(faded[2] < 1.0);
        assert_eq!(view.transmittance(0.0), [1.0, 1.0, 1.0]);
        assert!(view.transmittance(50.0).iter().all(|item| *item < 0.1));
    }

    #[test]
    fn view_tint_pulls_toward_murk_with_distance() {
        let view = UnderwaterView::murky_river();
        let close = view.view_tint(Color::WHITE, 0.4);
        let distant = view.view_tint(Color::WHITE, 9.0);
        let murk = view.murk_color.to_array();

        // Total drift toward the murk color shrinks with distance.
        let drift_near: f32 = close.iter().zip(murk).map(|(v, m)| (v - m).abs()).sum();
        let drift_far: f32 = distant.iter().zip(murk).map(|(v, m)| (v - m).abs()).sum();
        assert!(
            drift_far < drift_near,
            "distance pulls the view toward murk"
        );
        assert!(
            distant[0] < close[0],
            "distance mutes the red channel first"
        );

        // The additive murk tint always contributes non-negative channel
        // energy, so denser murk reads no darker than lighter murk.
        let murkier = view.with_murk(1.0).view_tint(Color::WHITE, 3.0);
        let clear = view.with_murk(0.05).view_tint(Color::WHITE, 3.0);
        assert!(
            murkier.iter().zip(clear).all(|(&a, b)| a >= b - 1.0e-6),
            "extra murk never darkens a channel: {murkier:?} vs {clear:?}"
        );
    }
}
