//! Rigid body state with sleeping and deterministic integration inputs.

use scenix_math::{Quat, Transform, Vec3};

/// Default world gravity on Earth, m/s^2 (magnitude).
pub const GRAVITY_DEFAULT: f32 = 9.81;

/// Velocity magnitude clamped by the solver to keep the world sane.
pub const MAX_LINEAR_VELOCITY: f32 = 1_000.0;

/// Integration substeps the world may run per `step`.
pub const MAX_SUBSTEPS: usize = 4;

/// Minimum mass a dynamic body may carry (kg).
pub const MIN_BODY_MASS: f32 = 0.0001;

/// Behavioral archetype of a body inside the world step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum RigidBodyKind {
    /// Never moves; kinematic anchors and static scenery.
    #[default]
    Static,
    /// Moves only by explicit velocity writes; animators own the transform.
    KinematicPosition,
    /// Fully simulated; gravity and impulses apply.
    Dynamic,
}

impl RigidBodyKind {
    /// Returns whether the solver integrates gravity for this kind.
    #[inline]
    #[must_use]
    pub const fn is_dynamically_forced(self) -> bool {
        matches!(self, Self::Dynamic)
    }

    /// Returns whether the solver allows this kind to be displaced by
    /// contact resolution positions.
    #[inline]
    #[must_use]
    pub const fn can_be_pushed(self) -> bool {
        matches!(self, Self::Dynamic)
    }

    /// Returns whether the body drives other bodies through velocity math.
    #[inline]
    #[must_use]
    pub const fn pushes_others(self) -> bool {
        matches!(self, Self::Dynamic | Self::KinematicPosition)
    }

    /// Returns a short human-readable label used by inspector tooling.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::KinematicPosition => "kinematic",
            Self::Dynamic => "dynamic",
        }
    }
}

/// Sleeping bookkeeping for the solver's hot loop.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SleepState {
    /// Consecutive frames below the motion threshold.
    pub still_frames: u32,
    /// Whether the body is currently asleep.
    pub asleep: bool,
}

/// Motion threshold in m/s below which sleep counting begins.
pub const SLEEP_MOTION_THRESHOLD: f32 = 0.05;

/// Frames a body must stay still before dropping into sleep.
pub const SLEEP_FRAME_THRESHOLD: u32 = 60;

impl SleepState {
    /// Sleep accounting per frame; returns true on transitions to sleep.
    #[must_use]
    pub fn tally(&mut self, motion: f32) -> bool {
        if motion < SLEEP_MOTION_THRESHOLD {
            self.still_frames = self.still_frames.saturating_add(1);
            if !self.asleep && self.still_frames >= SLEEP_FRAME_THRESHOLD {
                self.asleep = true;
                return true;
            }
        } else {
            self.still_frames = 0;
            self.asleep = false;
        }
        false
    }

    /// Wakes the body; returns whether this was a fresh wake.
    #[inline]
    pub fn wake(&mut self) -> bool {
        let was = self.asleep;
        self.asleep = false;
        self.still_frames = 0;
        was
    }
}

/// One rigid body in the world's slot array.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicsBody {
    /// Opaque body identifier handed out by the world.
    pub id: scenix_core::PhysicsBodyId,
    /// Behavioral kind.
    pub kind: RigidBodyKind,
    /// Current world transform.
    pub transform: Transform,
    /// Linear velocity, m/s.
    pub linear_velocity: Vec3,
    /// Angular velocity (about each tracking axis), rad/s.
    pub angular_velocity: Vec3,
    /// Mass in kg; statics ignore it.
    pub mass: f32,
    /// Gravity scale multiplier `>= 0`.
    pub gravity_scale: f32,
    /// Linear velocity damping applied per second.
    pub linear_damping: f32,
    /// Angular velocity damping applied per second.
    pub angular_damping: f32,
    /// Restitution `0..=1` used when this body hits another.
    pub restitution: f32,
    /// Friction coefficient `0..=1`.
    pub friction: f32,
    /// Sleeping bookkeeping.
    pub sleep: SleepState,
    /// Accumulated force this step (nondeterministic ordering if used
    /// from multiple systems; single-writer contract applies).
    pub force_accumulator: Vec3,
    /// Grabbed payload attached by the app (mesh id etc.).
    pub payload: u64,
}

impl PhysicsBody {
    /// Static scenery placeholder.
    #[must_use]
    pub fn new(transform: Transform) -> Self {
        Self {
            id: scenix_core::PhysicsBodyId::new(0),
            kind: RigidBodyKind::Static,
            transform,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            gravity_scale: 1.0,
            linear_damping: 0.01,
            angular_damping: 0.01,
            restitution: 0.0,
            friction: 0.5,
            sleep: SleepState::default(),
            force_accumulator: Vec3::ZERO,
            payload: 0,
        }
    }

    /// Returns the body with a new kind.
    #[inline]
    #[must_use]
    pub const fn with_kind(mut self, kind: RigidBodyKind) -> Self {
        self.kind = kind;
        self
    }

    /// Returns the body with a collider shape hint (consumed by the world).
    #[inline]
    #[must_use]
    pub const fn with_shape(mut self, shape: crate::colliders::ColliderShape) -> Self {
        self.payload = shape.as_packed_key();
        self
    }

    /// Returns the body with a new mass, clamped above
    /// [`MIN_BODY_MASS`].
    #[inline]
    #[must_use]
    pub const fn with_mass(mut self, mass: f32) -> Self {
        self.mass = if mass < MIN_BODY_MASS {
            MIN_BODY_MASS
        } else {
            mass
        };
        self
    }

    /// Returns the body with a new gravity scale.
    #[inline]
    #[must_use]
    pub const fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale.max(0.0);
        self
    }

    /// Returns the body with new surface coefficients.
    #[inline]
    #[must_use]
    pub const fn with_surface(mut self, restitution: f32, friction: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self.friction = friction.clamp(0.0, 1.0);
        self
    }

    /// Returns the body with new damping coefficients.
    #[inline]
    #[must_use]
    pub const fn with_damping(mut self, linear: f32, angular: f32) -> Self {
        self.linear_damping = linear.max(0.0);
        self.angular_damping = angular.max(0.0);
        self
    }

    /// Returns the body stamped with an app payload (`mesh id` etc).
    #[inline]
    #[must_use]
    pub const fn with_payload(mut self, payload: u64) -> Self {
        self.payload = payload;
        self
    }

    /// Sleep accounting against this body's motion energy each frame.
    #[inline]
    #[must_use]
    pub fn motion_speed(&self) -> f32 {
        self.linear_velocity.length()
    }

    /// Integrated velocity after `dt` with damping; used directly by the
    /// world's substeps (single consumer).
    pub fn integrate_velocity(&mut self, gravity: Vec3, dt: f32) {
        if self.kind.is_dynamically_forced() {
            let accel = gravity * self.gravity_scale;
            let force_accel = self.force_accumulator * (1.0 / self.mass.max(MIN_BODY_MASS));
            self.linear_velocity += (accel + force_accel) * dt;
        }
        self.linear_velocity *= damp_factor(self.linear_damping, dt);
        self.angular_velocity *= damp_factor(self.angular_damping, dt);
        let speed = self.linear_velocity.length();
        if speed > MAX_LINEAR_VELOCITY {
            let scale = MAX_LINEAR_VELOCITY / speed;
            self.linear_velocity *= scale;
        }
        self.force_accumulator = Vec3::ZERO;
    }

    /// Advance the transform using the current velocities over `dt`.
    #[must_use]
    pub fn advance_transform(&self, dt: f32) -> Transform {
        if dt <= 0.0 {
            return self.transform;
        }
        let delta = self.linear_velocity * dt;
        let spin = self.angular_velocity;
        let yaw = spin.y * dt;
        let pitch = spin.x * dt;
        let roll = spin.z * dt;
        let drift = euler_blend(pitch, yaw, roll);
        Transform::new(
            self.transform.translation + delta,
            drift * self.transform.rotation,
            self.transform.scale,
        )
    }

    /// Adds a world-space impulse; wakes the body.
    #[inline]
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.linear_velocity += impulse * (1.0 / self.mass.max(MIN_BODY_MASS));
        self.sleep.wake();
    }

    /// Adds a force contribution for the next step; wakes the body.
    #[inline]
    pub fn apply_force(&mut self, force: Vec3) {
        self.force_accumulator += force;
        self.sleep.wake();
    }

    /// Teleports the body (kinematics set velocity separately).
    #[inline]
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
        self.sleep.wake();
    }
}

/// Per-second exponential damping factor, clamped to a sane frame range.
#[inline]
#[must_use]
fn damp_factor(damping: f32, dt: f32) -> f32 {
    if damping <= 0.0 || dt <= 0.0 {
        1.0
    } else {
        (1.0 / (1.0 + damping * dt)).max(0.01)
    }
}

/// Deterministic small-angle Euler blend for integrated spin.
#[inline]
#[must_use]
fn euler_blend(pitch: f32, yaw: f32, roll: f32) -> Quat {
    let y_axis = scenix_math::Vec3::Y;
    let x_axis = scenix_math::Vec3::X;
    let z_axis = scenix_math::Vec3::Z;
    Quat::from_axis_angle(y_axis, yaw)
        .mul_quat(Quat::from_axis_angle(x_axis, pitch))
        .mul_quat(Quat::from_axis_angle(z_axis, roll))
}
