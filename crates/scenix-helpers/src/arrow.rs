use scenix_core::Color;
use scenix_math::Vec3;

use crate::{EPSILON, LineGeometry};

/// Directional arrow helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrowHelper {
    /// Arrow origin.
    pub origin: Vec3,
    /// Arrow direction. Zero falls back to negative Z.
    pub direction: Vec3,
    /// Total arrow length.
    pub length: f32,
    /// Line color.
    pub color: Color,
    /// Arrow head length.
    pub head_length: f32,
    /// Arrow head half-width.
    pub head_width: f32,
}

impl ArrowHelper {
    /// Creates an arrow helper.
    #[inline]
    pub fn new(origin: Vec3, direction: Vec3, length: f32, color: Color) -> Self {
        let length = length.abs().max(EPSILON);
        Self {
            origin,
            direction: normalize_direction(direction),
            length,
            color,
            head_length: length * 0.2,
            head_width: length * 0.08,
        }
    }

    /// Returns this helper with explicit head dimensions.
    #[inline]
    pub fn head(mut self, length: f32, width: f32) -> Self {
        self.head_length = length.abs().min(self.length).max(EPSILON);
        self.head_width = width.abs().max(EPSILON);
        self
    }

    /// Generates line-list geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let direction = normalize_direction(self.direction);
        let length = self.length.abs().max(EPSILON);
        let head_length = self.head_length.abs().min(length).max(EPSILON);
        let head_width = self.head_width.abs().max(EPSILON);
        let tip = self.origin + direction * length;
        let shaft_end = tip - direction * head_length;
        let basis = perpendicular_basis(direction);
        let right = basis.0 * head_width;
        let up = basis.1 * head_width;

        let mut geometry = LineGeometry::new();
        geometry.push_segment(self.origin, shaft_end, self.color);
        geometry.push_segment(tip, shaft_end + right, self.color);
        geometry.push_segment(tip, shaft_end - right, self.color);
        geometry.push_segment(tip, shaft_end + up, self.color);
        geometry.push_segment(tip, shaft_end - up, self.color);
        geometry
    }
}

impl Default for ArrowHelper {
    #[inline]
    fn default() -> Self {
        Self::new(Vec3::ZERO, Vec3::NEG_Z, 1.0, Color::WHITE)
    }
}

pub(crate) fn normalize_direction(direction: Vec3) -> Vec3 {
    let direction = direction.normalize();
    if direction.length_squared() <= EPSILON {
        Vec3::NEG_Z
    } else {
        direction
    }
}

pub(crate) fn perpendicular_basis(direction: Vec3) -> (Vec3, Vec3) {
    let reference = if direction.y.abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let right = direction.cross(reference).normalize();
    let up = right.cross(direction).normalize();
    (right, up)
}
