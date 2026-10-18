//! The per-particle simulation state.
//!
//! [`Particle`] is plain data: the CPU simulator owns a compact pool of
//! these, integrates them, and folds them into render batches. No GPU
//! types, no traits — values only.

use scenix_math::Vec3;

/// One alive particle's simulation state.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Particle {
    /// World-space position.
    pub position: Vec3,
    /// World-space velocity.
    pub velocity: Vec3,
    /// Current tint color, linear rgba.
    ///
    /// Seeded from the emitter color; modifier curves may overwrite it.
    pub color: [f32; 4],
    /// Current world-space half-extent of the billboard.
    pub size: f32,
    /// Billboard rotation around the view axis in radians.
    pub rotation: f32,
    /// Seconds since spawn.
    pub age: f32,
    /// Total lifetime in seconds; zero or negative means "never dies".
    pub lifetime: f32,
}

impl Particle {
    /// Returns the normalized lifetime progress in `[0, 1]`.
    ///
    /// Particles without a lifetime report `0.0` forever.
    #[inline]
    pub fn life_fraction(&self) -> f32 {
        if self.lifetime <= 0.0 {
            return 0.0;
        }
        let fraction = self.age / self.lifetime;
        if fraction.is_finite() {
            fraction.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Returns whether the particle has aged past its lifetime.
    #[inline]
    pub fn is_dead(&self) -> bool {
        self.lifetime > 0.0 && self.age >= self.lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn life_fraction_clamps_and_zero_lifetime_never_progresses() {
        let mut particle = Particle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            color: [1.0; 4],
            size: 1.0,
            rotation: 0.0,
            age: 0.75,
            lifetime: 1.0,
        };
        assert_eq!(particle.life_fraction(), 0.75);
        assert!(!particle.is_dead());
        particle.age = 2.0;
        assert_eq!(particle.life_fraction(), 1.0);
        assert!(particle.is_dead());

        particle.lifetime = 0.0;
        particle.age = 10.0;
        assert_eq!(particle.life_fraction(), 0.0);
        assert!(!particle.is_dead(), "eternal particle");
    }
}
