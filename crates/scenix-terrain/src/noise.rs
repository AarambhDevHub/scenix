//! Deterministic 2D value noise and fBm for heightmap generation.
//!
//! The generator is stateless: every sample is a pure function of
//! `(seed, x, z, config)` so terrain builds identically on every host and in
//! every thread order — matching the deterministic-simulation policy used by
//! `scenix-particles` in v1.7.0.

use alloc::vec::Vec;

/// Seamless 2D value noise configuration.
///
/// Frequencies are in cycles per meter; `amplitude` is applied by callers
/// after fBm evaluation (fBm itself returns roughly `[-1, 1]`).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoiseConfig {
    /// Integer seed mixed into every hashed coordinate.
    pub seed: u64,
    /// Base frequency in cycles per meter.
    pub frequency: f32,
    /// Octave count (clamped to at least one when sampled).
    pub octaves: u32,
    /// Frequency multiplier per octave.
    pub lacunarity: f32,
    /// Amplitude multiplier per octave, usually `0.35..=0.75`.
    pub gain: f32,
}

impl NoiseConfig {
    /// Creates a single-octave configuration at `frequency`.
    #[inline]
    pub const fn new(seed: u64, frequency: f32) -> Self {
        Self {
            seed,
            frequency,
            octaves: 1,
            lacunarity: 2.0,
            gain: 0.5,
        }
    }

    /// Returns this configuration with `octaves` fractional octaves.
    #[inline]
    pub const fn with_octaves(mut self, octaves: u32) -> Self {
        self.octaves = octaves;
        self
    }

    /// Returns this configuration with the octave ratio `(lacunarity, gain)`.
    #[inline]
    pub const fn with_lacunarity(mut self, lacunarity: f32) -> Self {
        self.lacunarity = lacunarity;
        self
    }

    /// Returns this configuration with the octave gain.
    #[inline]
    pub const fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain;
        self
    }

    /// Evaluates the noise field at `(x, z)` meters in roughly `[-1, 1]`.
    ///
    /// Each octave samples value noise at a scaled position and accumulates
    /// with `gain`; higher octaves contribute less detail. The sum stays
    /// deterministic for a fixed `seed`.
    #[must_use]
    pub fn sample(&self, x: f32, z: f32) -> f32 {
        // Octave count clamps to at least one so even `new()` sums a base
        // layer instead of returning a zero field.
        let octaves = self.octaves.max(1);
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.frequency.max(1.0e-6);
        let mut total = 0.0;
        let mut octave = 0;
        while octave < octaves {
            value += amplitude * Self::value_noise(self.seed, x * frequency, z * frequency);
            total += amplitude;
            amplitude *= self.gain;
            frequency *= self.lacunarity;
            octave += 1;
        }
        if total == 0.0 { 0.0 } else { value / total }
    }

    /// Single-octave lattice value noise with smooth interpolation.
    #[inline]
    fn value_noise(seed: u64, x: f32, z: f32) -> f32 {
        let floor_x = Self::floor(x);
        let floor_z = Self::floor(z);
        let base_u = Self::hash(seed, floor_x, floor_z);
        let base_v = Self::hash(seed, floor_x + 1.0, floor_z);
        let base_w = Self::hash(seed, floor_x, floor_z + 1.0);
        let base_t = Self::hash(seed, floor_x + 1.0, floor_z + 1.0);
        Self::bilinear(
            base_u,
            base_v,
            base_w,
            base_t,
            Self::smoothstep(x - floor_x),
            Self::smoothstep(z - floor_z),
        )
    }

    /// std-free floor: truncate, then step down when truncation rounded up.
    #[inline]
    const fn floor(value: f32) -> f32 {
        let truncated = value as i64 as f32;
        if value < truncated {
            truncated - 1.0
        } else {
            truncated
        }
    }

    /// 52-bit safe mix of an integer seed and two lattice coordinates.
    #[inline]
    fn hash(seed: u64, x: f32, z: f32) -> f32 {
        // Bit patterns of finite f32 coordinates hold ~1000 distinct
        // sign+mantissa hashes per unit step; both coordinates arrive as
        // lattice integers already.
        let mut mixed = seed ^ (x.to_bits() as u64).rotate_left(23);
        mixed ^= (z.to_bits() as u64).rotate_left(41);
        mixed = mixed.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        mixed ^= mixed >> 31;
        mixed = mixed.wrapping_mul(0xd6e8_feb8_6659_7b93);
        mixed ^= mixed >> 29;
        ((mixed & 0x00ff_ffff) as f32 / 16_777_216.0) * 2.0 - 1.0
    }

    /// Quintic crumple applied before lattice interpolation.
    #[inline]
    fn smoothstep(t: f32) -> f32 {
        let clamped = t.clamp(0.0, 1.0);
        clamped * clamped * clamped * (clamped * (clamped * 6.0 - 15.0) + 10.0)
    }

    /// Smooth lattice corner blend.
    #[inline]
    fn bilinear(a: f32, b: f32, c: f32, d: f32, su: f32, sv: f32) -> f32 {
        let top = a + (b - a) * su;
        let bottom = c + (d - c) * su;
        top + (bottom - top) * sv
    }
}

/// Fills `heights` (row-major, `width × depth`) from a noise configuration.
#[must_use]
pub fn noise_heights(width: u32, depth: u32, cell_size: f32, config: &NoiseConfig) -> Vec<f32> {
    let cell = if cell_size <= 0.0 { 1.0 } else { cell_size };
    let mut heights = Vec::with_capacity((width * depth) as usize);
    for iz in 0..depth {
        for ix in 0..width {
            heights.push(config.sample(ix as f32 * cell, iz as f32 * cell));
        }
    }
    heights
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{NoiseConfig, noise_heights};

    #[test]
    fn equal_inputs_give_equal_samples_across_seeds() {
        let a = NoiseConfig::new(7, 0.02).with_octaves(4);
        let b = NoiseConfig::new(9, 0.02).with_octaves(4);
        assert_eq!(a.sample(1.25, 3.5), a.sample(1.25, 3.5));
        assert!((a.sample(1.25, 3.5) - b.sample(1.25, 3.5)).abs() > 1.0e-9);
    }

    #[test]
    fn samples_stay_within_the_promised_range() {
        let config = NoiseConfig::new(1, 0.05).with_octaves(5).with_gain(0.42);
        let heights = noise_heights(32, 32, 1.0, &config);
        let extremes = heights.iter().fold(0.0_f32, |worst, h| worst.max(h.abs()));
        assert!(extremes <= 1.0 + 1.0e-6, "abs overflow {extremes}");
        assert!(extremes > 0.01, "field is dead-flat {extremes}");
    }

    #[test]
    fn zero_octaves_still_places_a_field() {
        let config = NoiseConfig::new(3, 0.1);
        let heights: Vec<f32> = noise_heights(8, 8, 1.0, &config);
        assert_eq!(heights.len(), 64);
    }
}
