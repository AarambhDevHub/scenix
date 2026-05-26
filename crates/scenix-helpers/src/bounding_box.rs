use scenix_core::Color;
use scenix_math::{Aabb, Vec3};

use crate::LineGeometry;

/// Wireframe AABB helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundingBoxHelper {
    /// Bounds to visualize.
    pub aabb: Aabb,
    /// Line color.
    pub color: Color,
}

impl BoundingBoxHelper {
    /// Creates a helper from bounds.
    #[inline]
    pub const fn new(aabb: Aabb, color: Color) -> Self {
        Self { aabb, color }
    }

    /// Generates line-list geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let min = self.aabb.min;
        let max = self.aabb.max;
        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ];
        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        let mut geometry = LineGeometry::new();
        geometry.positions.reserve(edges.len() * 2);
        geometry.colors.reserve(edges.len() * 2);
        for (a, b) in edges {
            geometry.push_segment(corners[a], corners[b], self.color);
        }
        geometry
    }
}
