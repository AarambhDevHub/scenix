use alloc::vec::Vec;

use scenix_core::Color;
use scenix_math::{Aabb, Plane, Quat, Ray3, Vec3};
use scenix_scene::{TransformMode, TransformSpace};

use crate::light_helper::append_circle;
use crate::{BoundingBoxHelper, LineGeometry};

/// Semantic transform-gizmo handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GizmoHandleId {
    /// X axis.
    X,
    /// Y axis.
    Y,
    /// Z axis.
    Z,
    /// XY plane.
    XY,
    /// XZ plane.
    XZ,
    /// YZ plane.
    YZ,
    /// Unconstrained center handle.
    Center,
}

/// Analytic picking volume for an editor handle.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GizmoHitShape {
    /// Capsule-like finite axis handle.
    Axis {
        /// Segment origin.
        origin: Vec3,
        /// Unit segment direction.
        direction: Vec3,
        /// Segment length.
        length: f32,
        /// Pick radius.
        radius: f32,
    },
    /// Square region on a plane.
    Plane {
        /// Region center.
        center: Vec3,
        /// Plane normal.
        normal: Vec3,
        /// First in-plane axis.
        axis_a: Vec3,
        /// Second in-plane axis.
        axis_b: Vec3,
        /// Half extent on both axes.
        half_extent: f32,
    },
    /// Circular ring.
    Ring {
        /// Ring center.
        center: Vec3,
        /// Ring normal.
        normal: Vec3,
        /// Ring radius.
        radius: f32,
        /// Radial pick thickness.
        thickness: f32,
    },
    /// Spherical center handle.
    Sphere {
        /// Sphere center.
        center: Vec3,
        /// Sphere radius.
        radius: f32,
    },
}

impl GizmoHitShape {
    /// Returns the ray distance when the analytic volume is hit.
    pub fn hit_distance(self, ray: Ray3) -> Option<f32> {
        match self {
            Self::Axis {
                origin,
                direction,
                length,
                radius,
            } => hit_axis(ray, origin, direction, length, radius),
            Self::Plane {
                center,
                normal,
                axis_a,
                axis_b,
                half_extent,
            } => {
                let plane = Plane::from_normal_and_point(normal, center);
                let distance = plane.intersect_ray(ray)?;
                let offset = ray.at(distance) - center;
                (offset.dot(axis_a).abs() <= half_extent && offset.dot(axis_b).abs() <= half_extent)
                    .then_some(distance)
            }
            Self::Ring {
                center,
                normal,
                radius,
                thickness,
            } => {
                let distance = Plane::from_normal_and_point(normal, center).intersect_ray(ray)?;
                let radial = ray.at(distance).distance(center);
                ((radial - radius).abs() <= thickness).then_some(distance)
            }
            Self::Sphere { center, radius } => ray.intersect_sphere(center, radius),
        }
    }
}

/// Pickable semantic handle.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GizmoHandle {
    /// Semantic handle identifier.
    pub id: GizmoHandleId,
    /// Analytic hit volume.
    pub shape: GizmoHitShape,
    /// Display color.
    pub color: Color,
}

/// Reusable line geometry plus analytic transform handles.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GizmoGeometry {
    /// Display line list.
    pub lines: LineGeometry,
    /// Pickable handles.
    pub handles: Vec<GizmoHandle>,
}

impl GizmoGeometry {
    /// Clears content while retaining capacities.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.handles.clear();
    }

    /// Returns the closest hit handle.
    pub fn hit_test(&self, ray: Ray3) -> Option<(GizmoHandleId, f32)> {
        self.handles
            .iter()
            .filter_map(|handle| {
                handle
                    .shape
                    .hit_distance(ray)
                    .map(|distance| (handle.id, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}

/// Translate, rotate, or scale gizmo generator.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransformGizmoHelper {
    /// World-space origin.
    pub origin: Vec3,
    /// Local orientation.
    pub rotation: Quat,
    /// Overall gizmo size.
    pub size: f32,
    /// Active transform mode.
    pub mode: TransformMode,
    /// World or local axis mode.
    pub space: TransformSpace,
}

impl TransformGizmoHelper {
    /// Creates a world-space transform gizmo.
    pub fn new(origin: Vec3, mode: TransformMode) -> Self {
        Self {
            origin,
            rotation: Quat::IDENTITY,
            size: 1.0,
            mode,
            space: TransformSpace::World,
        }
    }

    /// Writes into reusable geometry storage.
    pub fn write_geometry(&self, geometry: &mut GizmoGeometry) {
        geometry.clear();
        geometry.lines.reserve(256, 0);
        geometry.handles.reserve(7);
        let size = self.size.abs().max(1.0e-4);
        let rotation = match self.space {
            TransformSpace::World => Quat::IDENTITY,
            TransformSpace::Local => self.rotation,
        };
        let axes = [
            (GizmoHandleId::X, rotation.mul_vec3(Vec3::X), Color::RED),
            (GizmoHandleId::Y, rotation.mul_vec3(Vec3::Y), Color::GREEN),
            (GizmoHandleId::Z, rotation.mul_vec3(Vec3::Z), Color::BLUE),
        ];
        match self.mode {
            TransformMode::Translate | TransformMode::Scale => {
                for (id, axis, color) in axes {
                    geometry
                        .lines
                        .push_segment(self.origin, self.origin + axis * size, color);
                    geometry.handles.push(GizmoHandle {
                        id,
                        shape: GizmoHitShape::Axis {
                            origin: self.origin,
                            direction: axis,
                            length: size,
                            radius: size * 0.08,
                        },
                        color,
                    });
                    if self.mode == TransformMode::Scale {
                        let center = self.origin + axis * size;
                        let bounds = Aabb::new(
                            center - Vec3::ONE * size * 0.07,
                            center + Vec3::ONE * size * 0.07,
                        );
                        geometry
                            .lines
                            .merge(&BoundingBoxHelper::new(bounds, color).to_geometry());
                    }
                }
                append_plane_handles(geometry, self.origin, rotation, size);
                geometry.handles.push(GizmoHandle {
                    id: GizmoHandleId::Center,
                    shape: GizmoHitShape::Sphere {
                        center: self.origin,
                        radius: size * 0.12,
                    },
                    color: Color::WHITE,
                });
            }
            TransformMode::Rotate => {
                for (id, normal, color) in axes {
                    let (axis_a, axis_b) = perpendicular_basis(normal);
                    append_circle(
                        &mut geometry.lines,
                        self.origin,
                        axis_a,
                        axis_b,
                        size,
                        64,
                        color,
                    );
                    geometry.handles.push(GizmoHandle {
                        id,
                        shape: GizmoHitShape::Ring {
                            center: self.origin,
                            normal,
                            radius: size,
                            thickness: size * 0.08,
                        },
                        color,
                    });
                }
            }
        }
    }

    /// Generates owned geometry for convenience.
    pub fn to_geometry(&self) -> GizmoGeometry {
        let mut geometry = GizmoGeometry::default();
        self.write_geometry(&mut geometry);
        geometry
    }
}

fn append_plane_handles(geometry: &mut GizmoGeometry, origin: Vec3, rotation: Quat, size: f32) {
    let planes = [
        (
            GizmoHandleId::XY,
            Vec3::X,
            Vec3::Y,
            Vec3::Z,
            Color::from_rgba(1.0, 1.0, 0.0, 1.0),
        ),
        (
            GizmoHandleId::XZ,
            Vec3::X,
            Vec3::Z,
            Vec3::Y,
            Color::from_rgba(1.0, 0.0, 1.0, 1.0),
        ),
        (
            GizmoHandleId::YZ,
            Vec3::Y,
            Vec3::Z,
            Vec3::X,
            Color::from_rgba(0.0, 1.0, 1.0, 1.0),
        ),
    ];
    for (id, a, b, normal, color) in planes {
        let a = rotation.mul_vec3(a);
        let b = rotation.mul_vec3(b);
        let normal = rotation.mul_vec3(normal);
        let half = size * 0.12;
        let center = origin + (a + b) * size * 0.28;
        let corners = [
            center - a * half - b * half,
            center + a * half - b * half,
            center + a * half + b * half,
            center - a * half + b * half,
        ];
        for index in 0..4 {
            geometry
                .lines
                .push_segment(corners[index], corners[(index + 1) % 4], color);
        }
        geometry.handles.push(GizmoHandle {
            id,
            shape: GizmoHitShape::Plane {
                center,
                normal,
                axis_a: a,
                axis_b: b,
                half_extent: half,
            },
            color,
        });
    }
}

fn hit_axis(ray: Ray3, origin: Vec3, direction: Vec3, length: f32, radius: f32) -> Option<f32> {
    let direction = direction.normalize();
    let w0 = ray.origin - origin;
    let a = ray.direction.dot(ray.direction);
    let b = ray.direction.dot(direction);
    let c = direction.dot(direction);
    let d = ray.direction.dot(w0);
    let e = direction.dot(w0);
    let denominator = a * c - b * b;
    let mut axis_t = if denominator.abs() > 1.0e-6 {
        (a * e - b * d) / denominator
    } else {
        e
    };
    axis_t = axis_t.clamp(0.0, length.abs());
    let ray_t = (b * axis_t - d) / a.max(1.0e-6);
    if ray_t < 0.0 {
        return None;
    }
    let ray_point = ray.at(ray_t);
    let axis_point = origin + direction * axis_t;
    (ray_point.distance(axis_point) <= radius.abs()).then_some(ray_t)
}

fn perpendicular_basis(normal: Vec3) -> (Vec3, Vec3) {
    let normal = normal.normalize();
    let axis_a = if normal.x.abs() < 0.9 {
        normal.cross(Vec3::X).normalize()
    } else {
        normal.cross(Vec3::Y).normalize()
    };
    (axis_a, normal.cross(axis_a).normalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_gizmo_is_deterministic_and_pickable() {
        let helper = TransformGizmoHelper::new(Vec3::ZERO, TransformMode::Translate);
        let mut geometry = GizmoGeometry::default();
        helper.write_geometry(&mut geometry);
        assert_eq!(geometry.handles.len(), 7);
        let capacity = geometry.lines.positions.capacity();
        helper.write_geometry(&mut geometry);
        assert_eq!(geometry.lines.positions.capacity(), capacity);
        let ray = Ray3::new(Vec3::new(0.5, 0.02, 1.0), Vec3::NEG_Z);
        assert_eq!(geometry.hit_test(ray).unwrap().0, GizmoHandleId::X);
    }
}
