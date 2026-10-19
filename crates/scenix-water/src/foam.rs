//! Foam masks for wave crests and shorelines.

use alloc::vec::Vec;

/// Foam band parameters shared between the CPU crest evaluation and the
/// water WGSL.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FoamParams {
    /// Crest factor above which foam starts (`0..=1`).
    pub crest_threshold: f32,
    /// How sharply foam reaches full coverage near the threshold.
    pub edge_sharpness: f32,
    /// Scalar texture noise/scale tint multiplied into the final mask; `0`
    /// selects a uniform mask.
    pub pattern_scale: f32,
}

impl FoamParams {
    /// A moderate shoreline foam preset.
    #[inline]
    #[must_use]
    pub const fn shoreline() -> Self {
        Self {
            crest_threshold: 0.55,
            edge_sharpness: 6.0,
            pattern_scale: 1.0,
        }
    }

    /// Returns parameters with a custom crest threshold.
    #[inline]
    #[must_use]
    pub fn with_threshold(mut self, crest_threshold: f32) -> Self {
        self.crest_threshold = crest_threshold.clamp(0.0, 1.0);
        self
    }

    /// Maps a raw crest factor `0..=1` onto foam coverage `0..=1`.
    ///
    /// Uses a deterministic smoothstep across `edge_sharpness` around the
    /// crest threshold so coverage growth matches the WGSL mask exactly.
    #[must_use]
    pub fn mask(&self, crest: f32) -> f32 {
        let crest = crest.clamp(0.0, 1.0);
        let t = (crest - self.crest_threshold).max(0.0);
        let normalized = (t * self.edge_sharpness.max(0.001)).clamp(0.0, 1.0);
        normalized * normalized * (3.0 - 2.0 * normalized)
    }

    /// Sample-masked values for a wave; captures foam tests over many world
    /// positions without a geometry build.
    #[must_use]
    pub fn masks_over(&self, crest_values: &[f32]) -> Vec<f32> {
        crest_values.iter().map(|crest| self.mask(*crest)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::FoamParams;

    #[test]
    fn threshold_maps_monotonic_coverage() {
        let foam = FoamParams::shoreline();
        let none = foam.mask(0.0);
        let start = foam.mask(foam.crest_threshold);
        let full = foam.mask(1.0);
        assert_eq!(none, 0.0);
        assert_eq!(start, 0.0, "gate sits exactly at the threshold");
        assert!((full - 1.0).abs() < 1.0e-6);
        assert!(foam.mask(0.7) > foam.mask(0.6));
    }

    #[test]
    fn masks_over_preserves_order() {
        let foam = FoamParams::shoreline();
        let masks = foam.masks_over(&[0.0, 0.60, 0.65, 0.90, 1.0]);
        assert_eq!(masks.len(), 5);
        assert!((masks[0] - 0.0).abs() < 1.0e-6, "calm water stays clear");
        assert!(masks[0] < masks[1]);
        assert!(masks[1] < masks[2]);
        assert!(masks[2] < masks[3]);
        assert!((masks[3] - 1.0).abs() < 1.0e-5, "strong crests saturate");
        assert!((masks[4] - 1.0).abs() < 1.0e-5);
    }
}
