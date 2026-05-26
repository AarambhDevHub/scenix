use scenix_core::Color;
use scenix_light::{DirectionalLight, PointLight, SpotLight};
use scenix_math::{Quat, Vec3};

use crate::arrow::{ArrowHelper, normalize_direction, perpendicular_basis};
use crate::{EPSILON, LineGeometry};

/// Wireframe point-light helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointLightHelper {
    /// Point light to visualize.
    pub light: PointLight,
    /// World-space light position.
    pub position: Vec3,
    /// Helper color.
    pub color: Color,
    /// Circle segment count.
    pub segments: u32,
}

impl PointLightHelper {
    /// Creates a point-light helper.
    #[inline]
    pub const fn new(light: PointLight, position: Vec3, color: Color) -> Self {
        Self {
            light,
            position,
            color,
            segments: 32,
        }
    }

    /// Generates three orthogonal range circles.
    pub fn to_geometry(&self) -> LineGeometry {
        let radius = if self.light.range > EPSILON {
            self.light.range
        } else {
            1.0
        };
        let mut geometry = LineGeometry::new();
        append_circle(
            &mut geometry,
            self.position,
            Vec3::X,
            Vec3::Y,
            radius,
            self.segments,
            self.color,
        );
        append_circle(
            &mut geometry,
            self.position,
            Vec3::X,
            Vec3::Z,
            radius,
            self.segments,
            self.color,
        );
        append_circle(
            &mut geometry,
            self.position,
            Vec3::Y,
            Vec3::Z,
            radius,
            self.segments,
            self.color,
        );
        geometry
    }
}

/// Wireframe spot-light cone helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpotLightHelper {
    /// Spot light to visualize.
    pub light: SpotLight,
    /// World-space light position.
    pub position: Vec3,
    /// World-space cone direction.
    pub direction: Vec3,
    /// Helper color.
    pub color: Color,
    /// Circle segment count.
    pub segments: u32,
}

impl SpotLightHelper {
    /// Creates a spot-light helper.
    #[inline]
    pub fn new(light: SpotLight, position: Vec3, direction: Vec3, color: Color) -> Self {
        Self {
            light,
            position,
            direction: normalize_direction(direction),
            color,
            segments: 32,
        }
    }

    /// Generates cone edges and the outer cone ring.
    pub fn to_geometry(&self) -> LineGeometry {
        let direction = normalize_direction(self.direction);
        let length = if self.light.range > EPSILON {
            self.light.range
        } else {
            1.0
        };
        let radius = length * tan_approx(self.light.angle.clamp(0.0, 1.55));
        let center = self.position + direction * length;
        let (right, up) = perpendicular_basis(direction);
        let mut geometry = LineGeometry::new();
        append_circle(
            &mut geometry,
            center,
            right,
            up,
            radius,
            self.segments,
            self.color,
        );
        geometry.push_segment(self.position, center + right * radius, self.color);
        geometry.push_segment(self.position, center - right * radius, self.color);
        geometry.push_segment(self.position, center + up * radius, self.color);
        geometry.push_segment(self.position, center - up * radius, self.color);
        geometry
    }
}

/// Directional-light arrow helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirectionalLightHelper {
    /// Directional light to visualize.
    pub light: DirectionalLight,
    /// World-space arrow origin.
    pub origin: Vec3,
    /// Arrow length.
    pub length: f32,
    /// Helper color.
    pub color: Color,
}

impl DirectionalLightHelper {
    /// Creates a directional-light helper.
    #[inline]
    pub const fn new(light: DirectionalLight, origin: Vec3, length: f32, color: Color) -> Self {
        Self {
            light,
            origin,
            length,
            color,
        }
    }

    /// Generates an arrow in the light direction.
    #[inline]
    pub fn to_geometry(&self) -> LineGeometry {
        ArrowHelper::new(self.origin, self.light.direction, self.length, self.color).to_geometry()
    }
}

pub(crate) fn append_circle(
    geometry: &mut LineGeometry,
    center: Vec3,
    axis_a: Vec3,
    axis_b: Vec3,
    radius: f32,
    segments: u32,
    color: Color,
) {
    let radius = radius.abs().max(EPSILON);
    let segments = segments.max(3);
    let axis_a = axis_a.normalize();
    let axis_b = axis_b.normalize();
    let step = core::f32::consts::TAU / segments as f32;
    let mut previous = center + axis_a * radius;
    for index in 1..=segments {
        let angle = step * index as f32;
        let point = center + rotate_in_plane(axis_a, axis_b, angle) * radius;
        geometry.push_segment(previous, point, color);
        previous = point;
    }
}

fn rotate_in_plane(axis_a: Vec3, axis_b: Vec3, angle: f32) -> Vec3 {
    let rotation = Quat::from_axis_angle(axis_a.cross(axis_b).normalize(), angle);
    rotation.mul_vec3(axis_a).normalize()
}

fn tan_approx(value: f32) -> f32 {
    let x2 = value * value;
    value + value * x2 / 3.0 + 2.0 * value * x2 * x2 / 15.0
}
