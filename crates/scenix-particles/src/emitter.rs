//! Emitter configuration: where particles appear and with what base state.

use alloc::boxed::Box;
use alloc::vec::Vec;

use scenix_core::Color;
use scenix_math::Vec3;
use scenix_math::float::{cos, sin, sqrt};

use crate::modules::ParticleModifier;
use crate::particle::Particle;
use crate::rng::DeterministicRng;

/// Spawn volume shapes.
///
/// Shapes only affect the spawn *position* selection; velocities come from
/// the emitter's direction plus optional radial spread.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EmissionShape {
    /// All particles appear at the emitter origin.
    Point,
    /// Particles appear uniformly inside a sphere of the given radius.
    Sphere {
        /// Sphere radius in world units.
        radius: f32,
    },
    /// Particles appear uniformly inside an axis-aligned box.
    Box {
        /// Half extents in world units.
        half_extents: Vec3,
    },
    /// Particles appear inside a cone opening along the emitter direction.
    Cone {
        /// Base radius at the emitter origin.
        radius: f32,
        /// Cone height along the emission direction.
        height: f32,
    },
}

impl Default for EmissionShape {
    #[inline]
    fn default() -> Self {
        Self::Point
    }
}

/// Base spawn configuration for one particle effect.
///
/// Not `Clone` — modifier stacks carry boxed trait objects; rebuild from a
/// descriptor or keep the emitter owned inside a [`crate::ParticleSystem`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParticleEmitter {
    /// Emitter origin in the system's space.
    pub origin: Vec3,
    /// Spawn volume.
    pub shape: EmissionShape,
    /// Preferred emission direction (cone axis, velocity bias).
    pub direction: Vec3,
    /// Base spawn color, linear rgba.
    pub color: Color,
    /// Base world-space half-extent at spawn.
    pub size: f32,
    /// Base spin in radians per second at spawn.
    pub spin: f32,
    /// Emission rate in particles per second; `0` spawns only bursts.
    pub rate: f32,
    /// Inclusive lifetime range in seconds sampled per particle.
    pub lifetime_range: (f32, f32),
    /// Initial speed range along the emission direction, sampled per particle.
    pub speed_range: (f32, f32),
    /// Random velocity spread applied around the direction.
    pub spread: f32,
    /// Modifier modules applied to every spawned particle.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub modifiers: Vec<Box<dyn ParticleModifier>>,
}

impl ParticleEmitter {
    /// Creates a point emitter with sane defaults.
    ///
    /// Rate is 64 particles per second at a 0.5–1.5 s lifetime, spawning
    /// upward from the origin.
    #[inline]
    pub fn new() -> Self {
        Self {
            origin: Vec3::ZERO,
            shape: EmissionShape::Point,
            direction: Vec3::UP,
            color: Color::WHITE,
            size: 0.1,
            spin: 0.0,
            rate: 64.0,
            lifetime_range: (0.5, 1.5),
            speed_range: (1.0, 2.0),
            spread: 0.0,
            modifiers: Vec::new(),
        }
    }

    /// Returns the emitter with a different origin.
    #[inline]
    pub const fn at(mut self, origin: Vec3) -> Self {
        self.origin = origin;
        self
    }

    /// Returns the emitter with a different spawn shape.
    #[inline]
    pub const fn with_shape(mut self, shape: EmissionShape) -> Self {
        self.shape = shape;
        self
    }

    /// Returns the emitter with a different direction.
    #[inline]
    pub const fn toward(mut self, direction: Vec3) -> Self {
        self.direction = direction;
        self
    }

    /// Returns the emitter with a different base color.
    #[inline]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns the emitter with a different base size.
    #[inline]
    pub const fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Returns the emitter with a different spin rate.
    #[inline]
    pub const fn with_spin(mut self, spin: f32) -> Self {
        self.spin = spin;
        self
    }

    /// Returns the emitter with a different emission rate.
    #[inline]
    pub const fn with_rate(mut self, rate: f32) -> Self {
        self.rate = rate;
        self
    }

    /// Returns the emitter with a different lifetime range.
    #[inline]
    pub const fn with_lifetime(mut self, min: f32, max: f32) -> Self {
        self.lifetime_range = (min, max);
        self
    }

    /// Returns the emitter with a different speed range.
    #[inline]
    pub const fn with_speed(mut self, min: f32, max: f32) -> Self {
        self.speed_range = (min, max);
        self
    }

    /// Returns the emitter with a different velocity spread (radians).
    #[inline]
    pub const fn with_spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    /// Attaches one modifier module.
    #[inline]
    pub fn with_modifier(mut self, modifier: impl ParticleModifier + 'static) -> Self {
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Unused range loans for config formatters.
    #[inline]
    pub fn lifetime_range(&self) -> (f32, f32) {
        self.lifetime_range
    }

    /// Samples one spawn position according to the emission shape.
    pub fn sample_position(&self, rng: &mut DeterministicRng) -> Vec3 {
        let offset = match self.shape {
            EmissionShape::Point => Vec3::ZERO,
            EmissionShape::Sphere { radius } => rng.inside_unit_sphere() * radius.max(0.0),
            EmissionShape::Box { half_extents } => {
                rng.inside_unit_sphere().mul_elements(half_extents)
            }
            EmissionShape::Cone { radius, height } => {
                // Area-proportional radial sampling on a disc that widens
                // along the direction; height is biased by squared radius
                // to keep density uniform.
                let t = sqrt(rng.next_f32());
                let angle = core::f32::consts::TAU * rng.next_f32();
                Vec3::new(
                    radius * t * cos(angle),
                    height * (1.0 - t * t),
                    radius * t * sin(angle),
                )
            }
        };
        self.origin + offset
    }

    /// Builds a particle from the emitter state.
    #[must_use]
    pub fn spawn(&self, rng: &mut DeterministicRng) -> Particle {
        let position = self.sample_position(rng);
        let speed = rng.range(self.speed_range.0, self.speed_range.1);
        let direction = if self.spread > 0.0 {
            let unit = rng.unit_vector();
            let axis = self.direction.normalize();
            let blended = axis + unit * self.spread;
            blended.normalize()
        } else {
            self.direction.normalize()
        };
        let lifetime_min = self.lifetime_range.0.min(self.lifetime_range.1);
        let lifetime_max = self.lifetime_range.1.max(self.lifetime_range.0);
        Particle {
            position,
            velocity: direction * speed,
            color: self.color.to_array(),
            size: self.size,
            rotation: 0.0,
            age: 0.0,
            lifetime: rng.range(lifetime_min, lifetime_max).max(0.0),
        }
    }
}

impl Default for ParticleEmitter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for ParticleEmitter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ParticleEmitter")
            .field("origin", &self.origin)
            .field("shape", &self.shape)
            .field("direction", &self.direction)
            .field("color", &self.color)
            .field("size", &self.size)
            .field("rate", &self.rate)
            .field("modifiers", &self.modifiers.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::{ColorCurve, ScalarCurve};
    use crate::modules::{ColorOverLifetime, Drag, Gravity, SizeOverLifetime};

    #[test]
    fn point_emitter_spawns_at_origin() {
        let mut rng = DeterministicRng::new(3);
        let emitter = ParticleEmitter::new();
        for _ in 0..32 {
            let particle = emitter.spawn(&mut rng);
            assert_eq!(particle.position, Vec3::ZERO);
            assert!(particle.velocity.y.is_finite());
            assert!(particle.lifetime >= 0.5 && particle.lifetime <= 1.5);
        }
    }

    #[test]
    fn sphere_shape_respects_radius() {
        let mut rng = DeterministicRng::new(11);
        let emitter = ParticleEmitter::new().with_shape(EmissionShape::Sphere { radius: 2.0 });
        for _ in 0..128 {
            let particle = emitter.spawn(&mut rng);
            let offset = particle.position.distance(Vec3::ZERO);
            assert!(offset <= 2.0 + 1e-4);
        }
    }

    #[test]
    fn box_shape_respects_half_extents() {
        let mut rng = DeterministicRng::new(5);
        let emitter = ParticleEmitter::new().with_shape(EmissionShape::Box {
            half_extents: Vec3::new(1.0, 5.0, 1.0),
        });
        for _ in 0..64 {
            let particle = emitter.spawn(&mut rng);
            assert!(particle.position.x.abs() <= 1.0 + 1e-4);
            assert!(particle.position.y.abs() <= 5.0 + 1e-4);
        }
    }

    #[test]
    fn modifiers_chain_in_builder_order() {
        let emitter = ParticleEmitter::new()
            .with_modifier(Gravity::new(Vec3::new(0.0, -9.0, 0.0)))
            .with_modifier(Drag::new(2.0))
            .with_modifier(ColorOverLifetime::new(ColorCurve::default()))
            .with_modifier(SizeOverLifetime::new(ScalarCurve::default()));
        assert_eq!(emitter.modifiers.len(), 4);
    }

    #[test]
    fn spawned_particles_inherit_base_color_and_size() {
        let mut rng = DeterministicRng::new(2);
        let emitter = ParticleEmitter::new()
            .with_color(Color::from_rgba(0.1, 0.2, 0.3, 0.4))
            .with_size(0.5);
        let particle = emitter.spawn(&mut rng);
        assert_eq!(particle.color, [0.1, 0.2, 0.3, 0.4]);
        assert_eq!(particle.size, 0.5);
    }
}
