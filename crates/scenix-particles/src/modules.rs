//! Modifier modules: reusable per-particle effects.
//!
//! A module is a stateless, shareable effect — gravity, drag, size ramps —
//! that hooks two points of the simulation: `on_spawn` (tweak the freshly
//! sampled spawn state) and `update` (integrate per tick). Modules are
//! shared behind trait objects, so a system can run several in sequence.

use scenix_math::Vec3;

use crate::curve::{ColorCurve, ScalarCurve};
use crate::particle::Particle;
use crate::rng::DeterministicRng;

/// Shared trait for per-particle effect modules.
///
/// Implementations must be cheap (`Send + Sync`, no interior state) so a
/// single module can drive thousands of particles per tick.
pub trait ParticleModifier: Send + Sync {
    /// Hooks a freshly spawned particle.
    #[inline]
    fn on_spawn(&self, _particle: &mut Particle, _rng: &mut DeterministicRng) {}

    /// Integrates one particle by `dt` seconds.
    ///
    /// `t` is the particle's normalized lifetime progress in `[0, 1]`
    /// (`0.0` for particles without a lifetime).
    #[inline]
    fn update(&self, _particle: &mut Particle, _dt: f32, _t: f32) {}
}

/// Constant world-space acceleration, typically gravity.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gravity {
    /// Acceleration in world units per second squared.
    pub acceleration: Vec3,
}

impl Gravity {
    /// Creates a gravity module.
    #[inline]
    pub const fn new(acceleration: Vec3) -> Self {
        Self { acceleration }
    }
}

impl ParticleModifier for Gravity {
    #[inline]
    fn update(&self, particle: &mut Particle, dt: f32, _t: f32) {
        particle.velocity += self.acceleration * dt;
    }
}

/// Empirical velocity damping (`velocity *= (1 - k * dt)` per tick).
///
/// Semi-implicit and cheap; intended for small `k * dt` per tick.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Drag {
    /// Damping coefficient; larger values bleed velocity faster.
    pub coefficient: f32,
}

impl Drag {
    /// Creates a drag module.
    #[inline]
    pub const fn new(coefficient: f32) -> Self {
        Self { coefficient }
    }
}

impl ParticleModifier for Drag {
    #[inline]
    fn update(&self, particle: &mut Particle, dt: f32, _t: f32) {
        let factor = (1.0 - self.coefficient * dt).max(0.0);
        particle.velocity *= factor;
    }
}

/// Multiplies the particle color against a rgba ramp over its lifetime.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorOverLifetime {
    /// rgba multiplier ramp; componentwise over the current color.
    pub curve: ColorCurve,
}

impl ColorOverLifetime {
    /// Creates a color-over-lifetime module.
    #[inline]
    pub const fn new(curve: ColorCurve) -> Self {
        Self { curve }
    }

    /// Creates a module that fades a color out on both rgb and alpha.
    #[inline]
    pub fn fade_to(to: [f32; 4]) -> Self {
        Self::new(ColorCurve::from_to([1.0, 1.0, 1.0, 1.0], to))
    }
}

impl ParticleModifier for ColorOverLifetime {
    #[inline]
    fn update(&self, particle: &mut Particle, _dt: f32, t: f32) {
        for (channel, multiplier) in particle.color.iter_mut().zip(self.curve.eval(t)) {
            *channel *= multiplier;
        }
    }
}

/// Multiplies the billboard size over the particle lifetime.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SizeOverLifetime {
    /// Size multiplier ramp evaluated at the lifetime fraction.
    pub curve: ScalarCurve,
}

impl SizeOverLifetime {
    /// Creates a size-over-lifetime module.
    #[inline]
    pub const fn new(curve: ScalarCurve) -> Self {
        Self { curve }
    }

    /// Creates a module that evaluates `from` at birth and `to` at death.
    #[inline]
    pub fn from_to(from: f32, to: f32) -> Self {
        Self::new(ScalarCurve::new(alloc::vec![(0.0, from), (1.0, to)]))
    }
}

impl ParticleModifier for SizeOverLifetime {
    #[inline]
    fn update(&self, particle: &mut Particle, _dt: f32, t: f32) {
        particle.size *= self.curve.eval(t);
    }
}

/// Adds view-axis spin proportional to the lifetime ramp.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpinOverLife {
    /// Radians-per-second multiplier ramp.
    pub curve: ScalarCurve,
}

impl SpinOverLife {
    /// Creates a spin module.
    #[inline]
    pub const fn new(curve: ScalarCurve) -> Self {
        Self { curve }
    }

    /// Creates a module that spins at a constant rate.
    #[inline]
    pub fn constant_rate(radians_per_second: f32) -> Self {
        Self::new(ScalarCurve::constant(radians_per_second))
    }
}

impl ParticleModifier for SpinOverLife {
    #[inline]
    fn update(&self, particle: &mut Particle, dt: f32, t: f32) {
        particle.rotation += self.curve.eval(t) * dt;
    }
}

/// Reflects confined particles off an axis-aligned volume.
///
/// Useful for contained effects — airflow, sparks in a box — where
/// particles bounce instead of escaping.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfineBox {
    /// Half extents of the axis-aligned volume.
    pub half_extents: Vec3,
    /// Fraction of velocity kept after a bounce (`0..=1`).
    pub restitution: f32,
}

impl ConfineBox {
    /// Creates a confining module with elastic bounces.
    #[inline]
    pub const fn elastic(half_extents: Vec3) -> Self {
        Self {
            half_extents,
            restitution: 0.9,
        }
    }
}

impl ParticleModifier for ConfineBox {
    fn update(&self, particle: &mut Particle, _dt: f32, _t: f32) {
        let restitution = self.restitution.clamp(0.0, 1.0);
        for axis in 0..3 {
            let half = self.half_extents.to_array()[axis].abs();
            if half <= 0.0 {
                continue;
            }
            let position = particle.position.to_array()[axis];
            let velocity = particle.velocity.to_array()[axis];
            if position > half && velocity > 0.0 {
                particle.velocity = negate_axis(particle.velocity, axis, -velocity * restitution);
                particle.position = clip_axis(particle.position, axis, half - position + half);
            } else if position < -half && velocity < 0.0 {
                particle.velocity = negate_axis(particle.velocity, axis, -velocity * restitution);
                particle.position = clip_axis(particle.position, axis, -half - position - half);
            }
        }
    }
}

#[inline]
fn negate_axis(v: Vec3, axis: usize, new_value: f32) -> Vec3 {
    let mut out = v.to_array();
    out[axis] = new_value;
    Vec3::new(out[0], out[1], out[2])
}

#[inline]
fn clip_axis(v: Vec3, axis: usize, new_value: f32) -> Vec3 {
    let mut out = v.to_array();
    out[axis] = new_value;
    Vec3::new(out[0], out[1], out[2])
}

// `negate_axis` and `clip_axis` implement axis tweaks mirrored from the
// velocity sign flip; both are pure element-wise writers.

#[cfg(test)]
mod tests {
    use super::*;

    fn particle() -> Particle {
        Particle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            color: [1.0; 4],
            size: 1.0,
            rotation: 0.0,
            age: 0.0,
            lifetime: 1.0,
        }
    }

    #[test]
    fn gravity_integrates_velocity() {
        let mut p = particle();
        Gravity::new(Vec3::new(0.0, -9.81, 0.0)).update(&mut p, 0.1, 0.0);
        assert!((p.velocity.y + 0.981).abs() < 1e-5);
    }

    #[test]
    fn drag_never_inverts() {
        let mut p = particle();
        p.velocity = Vec3::new(10.0, 0.0, 0.0);
        let drag = Drag::new(12.0);
        for _ in 0..8 {
            drag.update(&mut p, 0.1, 0.0);
        }
        assert!(p.velocity.x >= 0.0 && p.velocity.x < 10.0);
    }

    #[test]
    fn color_and_size_curves_track_the_lifetime() {
        let mut p = particle();
        p.color = [1.0, 1.0, 1.0, 1.0];
        ColorOverLifetime::fade_to([0.0, 0.0, 0.0, 0.0]).update(&mut p, 0.016, 1.0);
        assert_eq!(p.color[3], 0.0, "alpha fully faded at end of life");
        let mut p2 = particle();
        SizeOverLifetime::from_to(1.0, 0.0).update(&mut p2, 0.016, 0.5);
        assert!((p2.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn spin_integrates_rotations() {
        let mut p = particle();
        SpinOverLife::constant_rate(2.0).update(&mut p, 0.5, 0.0);
        assert!((p.rotation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn confine_box_bounces_escaping_particles() {
        let mut p = particle();
        p.position = Vec3::new(1.5, 0.0, 0.0);
        p.velocity = Vec3::new(1.0, 0.0, 0.0);
        ConfineBox::elastic(Vec3::ONE).update(&mut p, 0.016, 0.0);
        assert!(p.position.x <= 1.0 + 1e-5);
        assert!(p.velocity.x < 0.0, "bounced back");
    }
}
