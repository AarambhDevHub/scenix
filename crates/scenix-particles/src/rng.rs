//! Deterministic particle RNG.
//!
//! Particle simulation must be reproducible: the same emitter configuration,
//! seed, and tick sequence must produce the same particles. scenix therefore
//! ships its own `xorshift64*` generator instead of depending on `rand`.
//! The generator is `no_std`-safe and costs one multiply and two shifts.

/// Deterministic `xorshift64*` generator for particle spawning.
///
/// A zero seed is folded to one so every seed produces a valid stream.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeterministicRng(u64);

impl DeterministicRng {
    /// Creates a generator from a non-zero-folded seed.
    #[inline]
    pub const fn new(seed: u64) -> Self {
        Self(if seed == 0 { 1 } else { seed })
    }

    /// Returns the next raw 64-bit value.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// Returns the next value in `[0, 1)`.
    ///
    /// Takes the high 24 bits, which keeps the float conversion exact.
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        let bits = self.next_u64() >> 40;
        (bits as f64 / 16_777_216.0) as f32
    }

    /// Returns a scalar in `[min, max)`.
    #[inline]
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    /// Returns a uniformly distributed unit-length direction.
    #[inline]
    pub fn unit_vector(&mut self) -> scenix_math::Vec3 {
        use scenix_math::Vec3;
        use scenix_math::float::{cos, sin, sqrt};

        let azimuth = core::f32::consts::TAU * self.next_f32();
        let z = 2.0 * self.next_f32() - 1.0;
        let radial = sqrt((1.0 - z * z).max(0.0));
        Vec3::new(radial * cos(azimuth), z, radial * sin(azimuth))
    }

    /// Returns a vector inside the unit sphere, uniform by volume.
    #[inline]
    pub fn inside_unit_sphere(&mut self) -> scenix_math::Vec3 {
        use scenix_math::float::cbrt;

        let r = cbrt(self.next_f32());
        self.unit_vector() * r
    }
}

impl Default for DeterministicRng {
    #[inline]
    fn default() -> Self {
        Self::new(0x5dee_ce66_d194_7723)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_math::Vec3;

    #[test]
    fn streams_are_deterministic_per_seed() {
        let mut a = DeterministicRng::new(7);
        let mut b = DeterministicRng::new(7);
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
            assert_eq!(a.next_f32(), b.next_f32());
        }
        let mut c = DeterministicRng::new(8);
        assert_ne!(a.next_u64(), c.next_u64());
    }

    #[test]
    fn zero_seed_is_folded_to_a_valid_stream() {
        let mut zero = DeterministicRng::new(0);
        assert_ne!(zero.next_u64(), 0);
    }

    #[test]
    fn ranges_and_directions_are_valid() {
        let mut rng = DeterministicRng::new(99);
        for _ in 0..64 {
            let value = rng.range(2.0, 5.0);
            assert!((2.0..5.0).contains(&value));
            let unit = rng.unit_vector();
            assert!((unit.length() - 1.0).abs() < 1e-5);
            let inside = rng.inside_unit_sphere();
            assert!(inside.length() <= 1.0005);
        }
    }

    #[test]
    fn direction_sampling_covers_every_axis() {
        let mut rng = DeterministicRng::new(1);
        let mut min_xyz = [f32::MAX; 3];
        let mut max_xyz = [f32::MIN; 3];
        for _ in 0..512 {
            let v = rng.unit_vector().to_array();
            for axis in 0..3 {
                min_xyz[axis] = min_xyz[axis].min(v[axis]);
                max_xyz[axis] = max_xyz[axis].max(v[axis]);
            }
        }
        for axis in 0..3 {
            assert!(max_xyz[axis] > 0.8);
            assert!(min_xyz[axis] < -0.8);
        }
    }

    #[test]
    fn default_stream_is_the_default_seed() {
        let mut from_default = DeterministicRng::default();
        let mut from_seeded = DeterministicRng::new(0x5dee_ce66_d194_7723);
        assert_eq!(from_default.next_u64(), from_seeded.next_u64());

        // Silence the unused binding warning from the `Vec3` import helper.
        let _ = Vec3::ZERO;
    }
}
