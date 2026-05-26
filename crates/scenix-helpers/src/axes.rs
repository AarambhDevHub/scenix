use scenix_core::Color;
use scenix_math::Vec3;

use crate::{EPSILON, LineGeometry};

/// RGB XYZ axes helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AxesHelper {
    /// Length of each positive axis.
    pub size: f32,
}

impl AxesHelper {
    /// Creates an axes helper.
    #[inline]
    pub const fn new(size: f32) -> Self {
        Self { size }
    }

    /// Generates line-list geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let size = self.size.abs().max(EPSILON);
        let mut geometry = LineGeometry::new();
        geometry.push_segment(Vec3::ZERO, Vec3::X * size, Color::RED);
        geometry.push_segment(Vec3::ZERO, Vec3::Y * size, Color::GREEN);
        geometry.push_segment(Vec3::ZERO, Vec3::Z * size, Color::BLUE);
        geometry
    }
}

impl Default for AxesHelper {
    #[inline]
    fn default() -> Self {
        Self::new(1.0)
    }
}
