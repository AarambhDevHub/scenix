//! CPU-side physics bridge for scenix.
//!
//! This crate is the `v1.9.0` physics layer: rigid body state machines,
//! analytic colliders (spheres, axis-aligned boxes, capsules), a
//! deterministic penalty-solver world step with restitution, friction, and
//! sleeping, kinematic character controllers, per-frame transform sync
//! bookkeeping, and collider wireframe geometry for debug drawing. There
//! is precisely **no GPU code** here — collision info flows into scene
//! nodes through `body_transform` and debug pipelines through
//! `scenix_helpers`.
//!
//! # Quick start
//!
//! ```rust
//! use scenix_math::{Quat, Vec3};
//! use scenix_physics::{
//!     ColliderShape, PhysicsBody, PhysicsWorld, RigidBodyKind, Transform,
//! };
//!
//! let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
//! let ground = world.add_solid(
//!     PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
//!     ColliderShape::boxy(20.0, 0.5, 20.0),
//! );
//! let ball = world.add_solid(
//!     PhysicsBody::new(Transform::new(
//!         Vec3::new(0.0, 3.0, 0.0),
//!         Quat::IDENTITY,
//!         Vec3::ONE,
//!     ))
//!     .with_kind(RigidBodyKind::Dynamic)
//!     .with_mass(1.0),
//!     ColliderShape::sphere(0.5),
//! );
//! for _ in 0..240 {
//!     world.step(1.0 / 60.0); // deterministic: same inputs, same world
//! }
//! let settled = world.body(ball).expect("ball body");
//! assert!(settled.transform.translation.y > 0.5, "the ball rests above");
//! assert!(world.are_touching(ball, ground), "contacts keep the body up");
//! ```
//!
//! # House rules
//!
//! * The solver is deterministic: the same frame sequence always produces
//!   the same contacts, velocities, and transforms.
//! * Boxes are axis-aligned (rotations belong to spheres and capsules);
//!   the narrow phase documents this limitation rather than approximating.
//! * Sleeping bodies skip integration; wakeups propagate through impulses
//!   so stepping stays reversible in recordings.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod bodies;
mod character;
mod colliders;
mod debug;
mod sync;
mod world;

pub use bodies::{
    GRAVITY_DEFAULT, MAX_LINEAR_VELOCITY, MAX_SUBSTEPS, MIN_BODY_MASS, PhysicsBody, RigidBodyKind,
    SLEEP_FRAME_THRESHOLD, SLEEP_MOTION_THRESHOLD, SleepState,
};
pub use character::{
    CharacterInput, CharacterState, CharacterStepResult, GroundContact,
    PHYSICS_DEFAULT_STEP_OFFSET, PHYSICS_GROUND_REATTACH_SPEED, PHYSICS_MAX_CLIMBABLE_SLOPE,
};
pub use colliders::{Collider, ColliderGroup, ColliderShape};
pub use debug::{
    WIREFRAME_SEGMENTS, WireframeSeg, debug_wireframe, debug_wireframe_world, wireframe_box,
    wireframe_capsule, wireframe_sphere,
};
pub use sync::{PhysicsTransformSync, SyncEntry, SyncQueue};
pub use world::{
    CONTACT_RESOLVE_PERCENT, CONTACT_SLOP, Contact, ContactEvent, MAX_SUBSTEP_DT, MIN_FRICTION,
    PhysicsWorld, REST_CONTACT_VELOCITY, SpatialQueryHit, WORLD_MAX_BODIES, WorldEvent,
};

/// Crate version reported for physics tooling.
pub const PHYSICS_VERSION: &str = "1.9.1";

/// Re-export of the math transform for engine adapters that want one call.
pub use scenix_math::Transform;
