use scenix_math::{Aabb, Mat4, Sphere, Vec3, Vec4};

use crate::EPSILON;

/// A frustum visibility result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Visibility {
    /// The object is outside the frustum.
    Outside,
    /// The object intersects one or more frustum planes.
    Intersects,
    /// The object is fully inside the frustum.
    Inside,
}

/// Six frustum planes in `[left, right, bottom, top, near, far]` order.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frustum {
    /// Plane equations as `(normal.xyz, distance)`.
    pub planes: [Vec4; 6],
}

impl Frustum {
    /// Extracts frustum planes from a WebGPU-depth view-projection matrix.
    pub fn from_view_projection(vp: Mat4) -> Self {
        let planes = [
            plane_from_rows(vp, 3, 0, 1.0),
            plane_from_rows(vp, 3, 0, -1.0),
            plane_from_rows(vp, 3, 1, 1.0),
            plane_from_rows(vp, 3, 1, -1.0),
            row_as_plane(vp, 2),
            plane_from_rows(vp, 3, 2, -1.0),
        ];
        Self { planes }
    }

    /// Returns whether a point is inside or on the frustum.
    #[inline]
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.planes
            .iter()
            .all(|plane| signed_distance(*plane, point) >= -EPSILON)
    }

    /// Tests a bounding sphere.
    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> Visibility {
        let mut intersects = false;
        let radius = radius.max(0.0);
        for plane in self.planes {
            let distance = signed_distance(plane, center);
            if distance < -radius {
                return Visibility::Outside;
            }
            if distance < radius {
                intersects = true;
            }
        }
        if intersects {
            Visibility::Intersects
        } else {
            Visibility::Inside
        }
    }

    /// Tests a bounding sphere.
    #[inline]
    pub fn contains_bounding_sphere(&self, sphere: Sphere) -> Visibility {
        self.contains_sphere(sphere.center, sphere.radius)
    }

    /// Tests an axis-aligned bounding box.
    pub fn contains_aabb(&self, aabb: &Aabb) -> Visibility {
        let mut intersects = false;
        for plane in self.planes {
            let normal = plane.truncate();
            let positive = Vec3::new(
                if normal.x >= 0.0 {
                    aabb.max.x
                } else {
                    aabb.min.x
                },
                if normal.y >= 0.0 {
                    aabb.max.y
                } else {
                    aabb.min.y
                },
                if normal.z >= 0.0 {
                    aabb.max.z
                } else {
                    aabb.min.z
                },
            );
            if signed_distance(plane, positive) < 0.0 {
                return Visibility::Outside;
            }

            let negative = Vec3::new(
                if normal.x >= 0.0 {
                    aabb.min.x
                } else {
                    aabb.max.x
                },
                if normal.y >= 0.0 {
                    aabb.min.y
                } else {
                    aabb.max.y
                },
                if normal.z >= 0.0 {
                    aabb.min.z
                } else {
                    aabb.max.z
                },
            );
            if signed_distance(plane, negative) < 0.0 {
                intersects = true;
            }
        }

        if intersects {
            Visibility::Intersects
        } else {
            Visibility::Inside
        }
    }

    /// Returns true when an AABB is not fully outside the frustum.
    #[inline]
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        self.contains_aabb(aabb) != Visibility::Outside
    }
}

impl Default for Frustum {
    #[inline]
    fn default() -> Self {
        Self::from_view_projection(Mat4::IDENTITY)
    }
}

fn plane_from_rows(matrix: Mat4, a: usize, b: usize, sign: f32) -> Vec4 {
    normalize_plane(Vec4::new(
        matrix.get(a, 0) + sign * matrix.get(b, 0),
        matrix.get(a, 1) + sign * matrix.get(b, 1),
        matrix.get(a, 2) + sign * matrix.get(b, 2),
        matrix.get(a, 3) + sign * matrix.get(b, 3),
    ))
}

fn row_as_plane(matrix: Mat4, row: usize) -> Vec4 {
    normalize_plane(Vec4::new(
        matrix.get(row, 0),
        matrix.get(row, 1),
        matrix.get(row, 2),
        matrix.get(row, 3),
    ))
}

fn normalize_plane(plane: Vec4) -> Vec4 {
    let normal = plane.truncate();
    let length = normal.length();
    if length <= EPSILON {
        plane
    } else {
        plane / length
    }
}

#[inline]
fn signed_distance(plane: Vec4, point: Vec3) -> f32 {
    plane.x * point.x + plane.y * point.y + plane.z * point.z + plane.w
}
