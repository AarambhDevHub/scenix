//! Physics collider debug helpers.
//!
//! Owns no geometry of its own: wireframes come straight out of
//! scenix-physics's `debug_wireframe`, re-colored per collider into a
//! `LineGeometry` so the renderer-facing debug pipeline stays untouched.
//! Enabled behind the `physics` feature.

use crate::LineGeometry;
use scenix_core::Color;
use scenix_math::Transform;
use scenix_physics::{ColliderShape, PhysicsWorld, debug_wireframe};

/// Collider wireframe helper that colors per collider class.
///
/// Solid colliders default to cyan, sensor colliders to warm orange, so
/// trigger volumes read apart from physical walls in debug lists.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColliderDebugHelper {
    /// Wireframe color for solid colliders.
    pub solid_color: Color,
    /// Wireframe color for sensor colliders.
    pub sensor_color: Color,
}

impl ColliderDebugHelper {
    /// Solid cyan / sensor orange defaults.
    #[inline]
    pub const fn new() -> Self {
        Self {
            solid_color: Color::rgb(0.15, 0.85, 0.95),
            sensor_color: Color::rgb(1.0, 0.65, 0.15),
        }
    }

    /// Sets custom colors.
    #[inline]
    pub const fn with_colors(mut self, solid_color: Color, sensor_color: Color) -> Self {
        self.solid_color = solid_color;
        self.sensor_color = sensor_color;
        self
    }

    /// Wireframes one collider shape under a transform.
    pub fn shape_to_geometry(
        &self,
        shape: &ColliderShape,
        transform: &Transform,
        color: Color,
    ) -> LineGeometry {
        let mut geometry = LineGeometry::new();
        for (start, end) in debug_wireframe(shape, transform) {
            geometry.push_segment(start, end, color);
        }
        geometry
    }

    /// Wireframes every collider in `world`, colored by sensor/solid class.
    pub fn world_to_geometry(&self, world: &PhysicsWorld) -> LineGeometry {
        let mut geometry = LineGeometry::new();
        for collider in world.colliders() {
            let Some(body) = world.body(collider.body_id) else {
                continue;
            };
            let color = if collider.sensor {
                self.sensor_color
            } else {
                self.solid_color
            };
            for (start, end) in debug_wireframe(&collider.shape, &body.transform) {
                geometry.push_segment(start, end, color);
            }
        }
        geometry
    }
}

impl Default for ColliderDebugHelper {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
