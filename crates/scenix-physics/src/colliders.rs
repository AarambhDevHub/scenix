//! Analytic colliders and group filtering.

use scenix_math::{Aabb, Transform, Vec3};

/// Available collider shapes.
///
/// Boxes are axis-aligned only: aligned boxes are solved exactly and
/// deterministically, where rotated OBB SAT would multiply solver cost and
/// ambiguity for the scenes scenix targets (rows, floors, triggers). For
/// rotated volumes use capsules and spheres.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColliderShape {
    /// Analytic sphere with a radius.
    Sphere(f32),
    /// Axis-aligned box given by half extents on each axis.
    Box(Vec3),
    /// Capsule along the local `Y` axis: segment half height plus radius.
    Capsule {
        /// Half distance between the two cap centers.
        half_height: f32,
        /// Radius around the axis.
        radius: f32,
    },
    /// Vertical plane (infinite floor at local `Y = 0`).
    Plane,
}

impl ColliderShape {
    /// Unit sphere collider.
    #[inline]
    #[must_use]
    pub const fn sphere(radius: f32) -> Self {
        Self::Sphere(radius.max(0.001))
    }

    /// Box collider from half extents.
    #[inline]
    #[must_use]
    pub const fn boxy(half_x: f32, half_y: f32, half_z: f32) -> Self {
        Self::Box(Vec3::new(
            half_x.abs().max(0.001),
            half_y.abs().max(0.001),
            half_z.abs().max(0.001),
        ))
    }

    /// Vertical capsule collider.
    #[inline]
    #[must_use]
    pub const fn capsule(radius: f32, half_height: f32) -> Self {
        Self::Capsule {
            half_height: half_height.max(0.0),
            radius: radius.max(0.001),
        }
    }

    /// Packs the shape tag for lightweight embedding in `PhysicsBody`.
    #[inline]
    #[must_use]
    pub const fn as_packed_key(&self) -> u64 {
        match self {
            Self::Sphere(_) => 1,
            Self::Box(_) => 2,
            Self::Capsule { .. } => 3,
            Self::Plane => 4,
        }
    }

    /// Local-space AABB bounds of the shape.
    #[must_use]
    pub fn local_aabb(&self) -> Aabb {
        match self {
            Self::Sphere(radius) => Aabb::new(
                Vec3::new(-*radius, -*radius, -*radius),
                Vec3::new(*radius, *radius, *radius),
            ),
            Self::Box(half) => Aabb::new(-*half, *half),
            Self::Capsule {
                half_height,
                radius,
            } => Aabb::new(
                Vec3::new(-*radius, -(*half_height + *radius), -*radius),
                Vec3::new(*radius, *half_height + *radius, *radius),
            ),
            Self::Plane => Aabb::new(
                Vec3::new(-1.0e5, -0.01, -1.0e5),
                Vec3::new(1.0e5, 0.01, 1.0e5),
            ),
        }
    }

    /// World-space AABB under `transform` (per-axis scale honored).
    #[must_use]
    pub fn world_aabb(&self, transform: &Transform) -> Aabb {
        let local = self.local_aabb();
        let half = local.half_extents().mul_elements(transform.scale);
        let center = transform.translation;
        Aabb::new(center - half, center + half)
    }

    /// Largest radius of the shape for broad-phase capsule spheres.
    #[must_use]
    pub const fn bounding_radius(&self) -> f32 {
        match self {
            Self::Sphere(radius) => *radius,
            Self::Box(half) => half.x.max(half.y).max(half.z),
            Self::Capsule {
                half_height,
                radius,
            } => half_height.abs() + *radius,
            Self::Plane => 1.0e5,
        }
    }
}

/// Collision group filter; bodies collide freely when masks intersect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColliderGroup {
    /// Bitmask of groups this collider belongs to.
    pub membership: u32,
    /// Bitmask of groups this collider scans for.
    pub filter: u32,
}

impl ColliderGroup {
    /// Every role, every filter: default sandbox behavior.
    #[inline]
    #[must_use]
    pub const fn all() -> Self {
        Self {
            membership: u32::MAX,
            filter: u32::MAX,
        }
    }

    /// Returns the group with a new membership mask.
    #[inline]
    #[must_use]
    pub const fn with_membership(mut self, membership: u32) -> Self {
        self.membership = membership;
        self
    }

    /// Returns the group with a new filter mask.
    #[inline]
    #[must_use]
    pub const fn with_filter(mut self, filter: u32) -> Self {
        self.filter = filter;
        self
    }

    /// Whether `self` and `other` should interact given mutual filters.
    #[inline]
    #[must_use]
    pub const fn interacts_with(&self, other: &Self) -> bool {
        let a = (self.membership & other.filter) != 0;
        let b = (other.membership & self.filter) != 0;
        a && b
    }
}

/// Colliders couple shapes to solver-facing surface parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Collider {
    /// Opaque collider identifier.
    pub id: scenix_core::ColliderId,
    /// Owning body whose transform the collider follows.
    pub body_id: scenix_core::PhysicsBodyId,
    /// Geometric shape.
    pub shape: ColliderShape,
    /// Group filtering for broad-phase candidates.
    pub group: ColliderGroup,
    /// Sensor colliders report hits but never resolve.
    pub sensor: bool,
    /// Per-collider restitution override; `< 0` uses the body's value.
    pub restitution: f32,
    /// Per-collider friction override; `< 0` uses the body's value.
    pub friction: f32,
}

impl Collider {
    /// Creates a collider matching `body_id` with the default surface.
    #[must_use]
    pub fn new(body_id: scenix_core::PhysicsBodyId, shape: ColliderShape) -> Self {
        Self {
            id: scenix_core::ColliderId::new(1),
            body_id,
            shape,
            group: ColliderGroup::all(),
            sensor: false,
            restitution: -1.0,
            friction: -1.0,
        }
    }

    /// Returns the collider with a new id.
    #[inline]
    #[must_use]
    pub const fn with_id(mut self, id: scenix_core::ColliderId) -> Self {
        self.id = id;
        self
    }

    /// Returns the collider restricted to `group` masks.
    #[inline]
    #[must_use]
    pub const fn with_group(mut self, group: ColliderGroup) -> Self {
        self.group = group;
        self
    }

    /// Returns the collider marked as a trigger volume.
    #[inline]
    #[must_use]
    pub const fn as_sensor(mut self) -> Self {
        self.sensor = true;
        self
    }

    /// Surface restitution the solver should use against `other`.
    #[inline]
    #[must_use]
    pub const fn resolved_restitution(&self, body: &crate::bodies::PhysicsBody) -> f32 {
        if self.restitution < 0.0 {
            body.restitution
        } else {
            self.restitution
        }
    }

    /// Surface friction the solver should use against both bodies.
    #[inline]
    #[must_use]
    pub const fn resolved_friction(&self, body: &crate::bodies::PhysicsBody) -> f32 {
        if self.friction < 0.0 {
            body.friction
        } else {
            self.friction
        }
    }
}
