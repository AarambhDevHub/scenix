//! Bone-axis gizmo helper for a `SkeletonPose`.
//!
//! Draws a small axis triad at each bone origin for pose debugging.

use alloc::vec::Vec;

use scenix_core::{Color, ValidationError};
use scenix_math::Vec3;

use crate::LineGeometry;

/// Draws a small axis triad at each bone origin for pose debugging.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoseHelper {
    /// One triad per bone: `(origin, x_end, y_end, z_end)`.
    pub triads: Vec<(Vec3, Vec3, Vec3, Vec3)>,
    /// Axis colors `(x, y, z)`.
    pub colors: [Color; 3],
}

impl PoseHelper {
    /// Builds triads from bone origins and a per-axis `size` length.
    pub fn from_origins(origins: &[Vec3], size: f32) -> Self {
        let triads = origins
            .iter()
            .map(|&o| {
                (
                    o,
                    o + Vec3::new(size, 0.0, 0.0),
                    o + Vec3::new(0.0, size, 0.0),
                    o + Vec3::new(0.0, 0.0, size),
                )
            })
            .collect();
        Self {
            triads,
            colors: [
                Color::rgb(1.0, 0.0, 0.0),
                Color::rgb(0.0, 1.0, 0.0),
                Color::rgb(0.0, 0.0, 1.0),
            ],
        }
    }

    /// Sets custom axis colors.
    #[inline]
    pub const fn with_colors(mut self, colors: [Color; 3]) -> Self {
        self.colors = colors;
        self
    }

    /// Validates that at least one triad exists.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.triads.is_empty() {
            return Err(ValidationError::InvalidState);
        }
        Ok(())
    }

    /// Converts the triads to a multi-colored `[LineGeometry; 3]` (x/y/z).
    pub fn to_geometries(&self) -> [LineGeometry; 3] {
        let mut gx = LineGeometry::new();
        let mut gy = LineGeometry::new();
        let mut gz = LineGeometry::new();
        for &(o, x, y, z) in &self.triads {
            gx.push_segment(o, x, self.colors[0]);
            gy.push_segment(o, y, self.colors[1]);
            gz.push_segment(o, z, self.colors[2]);
        }
        [gx, gy, gz]
    }
}
