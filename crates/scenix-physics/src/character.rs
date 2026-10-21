//! Kinematic character controller with ground detection and slopes.

use scenix_math::{Transform, Vec3};

use crate::colliders::ColliderShape;
use crate::world::PhysicsWorld;

/// Default step offset, meters.
pub const PHYSICS_DEFAULT_STEP_OFFSET: f32 = 0.35;

/// Steepest climbable slope, radians (≈50°).
pub const PHYSICS_MAX_CLIMBABLE_SLOPE: f32 = 0.8727;

/// Vertical speed below which a ground-probe hit counts as standing.
///
/// A probe hit while the body still climbs faster than this is a launch
/// frame, not grounded support: re-attaching there would re-arm the jump
/// branch every frame and let the character fly at launch speed.
pub const PHYSICS_GROUND_REATTACH_SPEED: f32 = 0.1;

/// Player input snapshot consumed by the controller each frame.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharacterInput {
    /// Horizontal wish direction in world space, unit length or zero.
    pub wish_direction: Vec3,
    /// Desired ground speed, m/s.
    pub speed: f32,
    /// Jump trigger for this frame.
    pub jump: bool,
    /// World delta time, seconds.
    pub dt: f32,
}

impl CharacterInput {
    /// No-op frame.
    #[inline]
    #[must_use]
    pub const fn idle(dt: f32) -> Self {
        Self {
            wish_direction: Vec3::ZERO,
            speed: 0.0,
            jump: false,
            dt,
        }
    }

    /// Sanitized move command.
    #[must_use]
    pub fn move_towards(direction: Vec3, speed: f32, dt: f32) -> Self {
        Self {
            wish_direction: if direction.length_squared() <= 1.0e-12 {
                Vec3::ZERO
            } else {
                direction.normalize()
            },
            speed: speed.max(0.0),
            jump: false,
            dt,
        }
    }
}

/// Ground contact state for the previous frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroundContact {
    /// Standing on geometry this frame.
    pub landed: bool,
    /// Ground normal if landed, else `+Y`.
    pub normal: Vec3,
    /// Was airborne last frame; used for landing events.
    pub just_landed: bool,
}

impl GroundContact {
    /// Airborne state.
    #[inline]
    #[must_use]
    pub const fn airborne() -> Self {
        Self {
            landed: false,
            normal: Vec3::Y,
            just_landed: false,
        }
    }

    /// Slope angle extracted from the ground normal, radians.
    #[inline]
    #[must_use]
    pub fn slope(&self) -> f32 {
        let cos = self.normal.dot(Vec3::Y);
        scenix_authored_acos(cos)
    }
}

/// Header-safe acos, clamped; deterministic CPU path.
#[inline]
#[must_use]
fn scenix_authored_acos(value: f32) -> f32 {
    scenix_math::float::acos(value.clamp(-1.0, 1.0))
}

/// Kinematic capsule controller operating on one `PhysicsWorld`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharacterState {
    /// Body id the controller drives; created externally with a capsule.
    pub body: scenix_core::PhysicsBodyId,
    /// Capsule geometry expected from the body's collider.
    pub radius: f32,
    /// Capsule half height above the inner segment.
    pub half_height: f32,
    /// Movement settings.
    pub step_offset: f32,
    /// Max climbable slope angle, radians.
    pub max_slope: f32,
    /// Gravity magnitude while airborne, m/s².
    pub gravity: f32,
    /// Jump impulse speed, m/s.
    pub jump_speed: f32,
    /// Ground bookkeeping.
    pub ground: GroundContact,
}

impl CharacterState {
    /// Builds a controller plan over a pre-existing capsule body.
    #[must_use]
    pub fn new(body: scenix_core::PhysicsBodyId, radius: f32, half_height: f32) -> Self {
        Self {
            body,
            radius,
            half_height,
            step_offset: PHYSICS_DEFAULT_STEP_OFFSET,
            max_slope: PHYSICS_MAX_CLIMBABLE_SLOPE,
            gravity: 9.81,
            jump_speed: 5.0,
            ground: GroundContact::airborne(),
        }
    }

    /// Returns the controller with new step offset.
    #[inline]
    #[must_use]
    pub const fn with_step_offset(mut self, offset: f32) -> Self {
        self.step_offset = offset.max(0.0);
        self
    }

    /// Returns the controller with new gravity magnitude.
    #[inline]
    #[must_use]
    pub const fn with_gravity(mut self, gravity: f32) -> Self {
        self.gravity = gravity.max(0.0);
        self
    }

    /// Returns the controller with new jump speed.
    #[inline]
    #[must_use]
    pub const fn with_jump_speed(mut self, speed: f32) -> Self {
        self.jump_speed = speed.max(0.0);
        self
    }

    /// Runs one `move_and_slide` frame deterministically.
    ///
    /// Movement is horizontal-first (wish velocity), then vertical
    /// (gravity or jump). Ground positions are restored from ray probes
    /// so characters stay glued to slopes without tunneling.
    pub fn move_and_slide(
        &mut self,
        world: &mut PhysicsWorld,
        input: &CharacterInput,
    ) -> CharacterStepResult {
        let dt = input.dt.clamp(0.0, 0.25);
        if dt <= 0.0 {
            return CharacterStepResult::idle(self.ground.landed);
        }
        let body = match world.body(self.body) {
            Some(body) => *body,
            None => return CharacterStepResult::idle(false),
        };

        // Horizontal displacement from the wish direction.
        let horizontal = input.wish_direction * input.speed * dt;
        let was_landed = self.ground.landed;

        // Probe the ground below the feet through world colliders only
        // (a ray that starts inside our own capsule always strikes its
        // own exit face and would pin the character midair).
        let origin = body.transform.translation;
        let foot_probe = origin + Vec3::new(0.0, -(self.half_height + self.radius * 0.5), 0.0);
        let probe = scenix_math::Ray3::new(foot_probe, Vec3::new(0.0, -1.0, 0.0));
        let ground_hit = world
            .cast_ray_excluding(probe, self.radius + self.step_offset, self.body)
            .map(|hit| (hit.distance, hit.normal));

        // Re-attach only when not still climbing: a probe hit while the
        // body rises faster than [`PHYSICS_GROUND_REATTACH_SPEED`] is a
        // launch frame, not standing support.
        let mut landed =
            ground_hit.is_some() && body.linear_velocity.y <= PHYSICS_GROUND_REATTACH_SPEED;
        let ground_normal = ground_hit.map(|(_, normal)| normal).unwrap_or(Vec3::Y);

        // Slope limit blocks wish motion on steep walls.
        let slope = angle_between(ground_normal, Vec3::Y);
        let climbable = !landed || slope <= self.max_slope;

        // Vertical: jumping off the ground, else gravity pull.
        let mut vertical_velocity = body.linear_velocity.y;
        if input.jump && landed {
            vertical_velocity = self.jump_speed;
            landed = false;
        } else if !landed {
            vertical_velocity -= self.gravity * dt;
        } else if vertical_velocity < 0.0 {
            vertical_velocity = 0.0;
        }

        // Compose and commit velocities; the world integrates on step().
        let wish_velocity = if climbable {
            horizontal * (1.0 / dt)
        } else {
            Vec3::ZERO
        };
        if let Some(body) = world.body_mut(self.body) {
            // Restore the vertical position from the probe so stair edges
            // and slopes glue the feet onto the surface instead of
            // hovering: the probe starts `radius * 0.5` above the capsule
            // bottom, so `pad - hit.distance` is the snap onto the floor.
            if let Some((distance, _)) = ground_hit
                && landed
            {
                let pad = self.radius * 0.5;
                let snap = (pad - distance).clamp(-self.step_offset, self.step_offset);
                body.transform.translation.y = clamp_finite(body.transform.translation.y + snap);
            }
            body.linear_velocity = Vec3::new(
                clamp_finite(wish_velocity.x),
                clamp_finite(vertical_velocity),
                clamp_finite(wish_velocity.z),
            );
            // A controller frame re-drives its capsule, so physics does
            // not sleep the body out from under gameplay.
            body.sleep.wake();
        }

        self.ground = GroundContact {
            landed,
            normal: ground_normal,
            just_landed: landed && !was_landed,
        };

        CharacterStepResult {
            moved: input.wish_direction * input.speed * dt,
            landed: self.ground.landed,
            just_landed: self.ground.just_landed,
            grounded_slope: slope,
        }
    }

    /// Capsule shape of the controller's body; forwards to world lookups.
    #[must_use]
    pub fn capsule_shape(&self) -> ColliderShape {
        ColliderShape::capsule(self.radius, self.half_height)
    }

    /// Resets controller bookkeeping after teleports.
    #[inline]
    pub fn respawn(&mut self, transform: Transform, world: &mut PhysicsWorld) {
        self.ground = GroundContact::airborne();
        if let Some(body) = world.body_mut(self.body) {
            body.set_transform(transform);
            body.linear_velocity = Vec3::ZERO;
            body.angular_velocity = Vec3::ZERO;
        }
    }
}

/// Result bundle of one `move_and_slide` frame.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharacterStepResult {
    /// Horizontal displacement this frame.
    pub moved: Vec3,
    /// Grounded status after the frame.
    pub landed: bool,
    /// Landing edge this frame.
    pub just_landed: bool,
    /// Slope of the current ground, radians.
    pub grounded_slope: f32,
}

impl CharacterStepResult {
    #[inline]
    #[must_use]
    pub const fn idle(landed: bool) -> Self {
        Self {
            moved: Vec3::ZERO,
            landed,
            just_landed: false,
            grounded_slope: 0.0,
        }
    }
}

/// Quaternion-free angle between two unit-ish vectors.
#[inline]
#[must_use]
fn angle_between(from: Vec3, to: Vec3) -> f32 {
    let from_len = from.length_squared();
    let to_len = to.length_squared();
    if from_len <= 1.0e-12 || to_len <= 1.0e-12 {
        return 0.0;
    }
    let cos = from.dot(to) / scenix_math::float::sqrt(from_len) / scenix_math::float::sqrt(to_len);
    scenix_math::float::acos(cos.clamp(-1.0, 1.0))
}

/// Component clamp that rejects non-finite inputs outright.
#[inline]
#[must_use]
fn clamp_finite(component: f32) -> f32 {
    if component.is_finite() {
        component.clamp(-1.0e4, 1.0e4)
    } else {
        0.0
    }
}
