//! Line-geometry helper that visualizes a sampled animation path.
//!
//! Build a polyline from sampled positions so an animation trajectory can be
//! drawn like Three.js `GridHelper`/`CameraHelper` debug overlays.

use alloc::vec::Vec;

use scenix_core::{Color, ValidationError};
use scenix_math::Vec3;

use crate::LineGeometry;

/// Polyline helper for visualizing a sampled animation trajectory.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimationPathHelper {
    /// Ordered polyline points.
    pub points: Vec<Vec3>,
    /// Line color.
    pub color: Color,
}

impl AnimationPathHelper {
    /// Creates a helper from sampled points.
    #[inline]
    pub const fn new(points: Vec<Vec3>, color: Color) -> Self {
        Self { points, color }
    }

    /// Samples a closure `f: f32 -> Vec3` across `[0, duration]` in `steps`
    /// segments and builds a path helper.
    pub fn sample(
        steps: usize,
        duration: f32,
        color: Color,
        mut f: impl FnMut(f32) -> Vec3,
    ) -> Self {
        let mut points = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let t = if steps == 0 {
                0.0
            } else {
                duration * (i as f32) / (steps as f32)
            };
            points.push(f(t));
        }
        Self::new(points, color)
    }

    /// Validates that the path has at least two points.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.points.len() < 2 {
            return Err(ValidationError::InvalidState);
        }
        Ok(())
    }

    /// Converts the path to a `LineGeometry` (line-strip segments).
    pub fn to_geometry(&self) -> LineGeometry {
        let mut geometry = LineGeometry::new();
        for window in self.points.windows(2) {
            geometry.push_segment(window[0], window[1], self.color);
        }
        geometry
    }
}
