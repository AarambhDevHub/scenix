use scenix_core::Color;
use scenix_math::Vec3;

use crate::{EPSILON, LineGeometry};

/// XZ-plane grid helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridHelper {
    /// Total grid extent.
    pub size: f32,
    /// Number of subdivisions per axis.
    pub divisions: u32,
    /// Center-axis color.
    pub color1: Color,
    /// Regular grid-line color.
    pub color2: Color,
}

impl GridHelper {
    /// Creates a grid helper.
    #[inline]
    pub const fn new(size: f32, divisions: u32) -> Self {
        Self {
            size,
            divisions,
            color1: Color::from_rgba(0.45, 0.45, 0.45, 1.0),
            color2: Color::from_rgba(0.2, 0.2, 0.2, 1.0),
        }
    }

    /// Returns this helper with custom colors.
    #[inline]
    pub const fn colors(mut self, center: Color, grid: Color) -> Self {
        self.color1 = center;
        self.color2 = grid;
        self
    }

    /// Generates line-list geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let size = self.size.abs().max(EPSILON);
        let divisions = self.divisions.max(1);
        let half = size * 0.5;
        let step = size / divisions as f32;
        let mut geometry = LineGeometry::new();
        geometry.positions.reserve(((divisions + 1) * 4) as usize);
        geometry.colors.reserve(((divisions + 1) * 4) as usize);

        for i in 0..=divisions {
            let k = -half + i as f32 * step;
            let color = if k.abs() <= EPSILON {
                self.color1
            } else {
                self.color2
            };
            geometry.push_segment(Vec3::new(-half, 0.0, k), Vec3::new(half, 0.0, k), color);
            geometry.push_segment(Vec3::new(k, 0.0, -half), Vec3::new(k, 0.0, half), color);
        }

        geometry
    }
}

impl Default for GridHelper {
    #[inline]
    fn default() -> Self {
        Self::new(10.0, 10)
    }
}
