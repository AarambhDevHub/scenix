use alloc::vec::Vec;

use scenix_core::{Bounded, Color, ValidationError};
use scenix_math::{Aabb, Vec2, Vec3, Vec4};

use crate::EPSILON;

/// Indexed triangle geometry stored in CPU-side arrays.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Geometry {
    /// Vertex positions.
    pub positions: Vec<Vec3>,
    /// Vertex normals.
    pub normals: Vec<Vec3>,
    /// Primary UV coordinates.
    pub uvs: Vec<Vec2>,
    /// Secondary UV coordinates.
    pub uvs2: Vec<Vec2>,
    /// Vertex colors.
    pub colors: Vec<Color>,
    /// Triangle indices.
    pub indices: Vec<u32>,
    /// Tangent vectors with handedness in `w`.
    pub tangents: Vec<Vec4>,
}

impl Geometry {
    /// Creates an empty geometry.
    #[inline]
    pub const fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            uvs2: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
            tangents: Vec::new(),
        }
    }

    /// Creates geometry from positions.
    #[inline]
    pub fn with_positions(positions: Vec<Vec3>) -> Self {
        Self {
            positions,
            ..Self::new()
        }
    }

    /// Returns the number of vertices.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns the number of triangles.
    #[inline]
    pub fn triangle_count(&self) -> usize {
        if self.indices.is_empty() {
            self.positions.len() / 3
        } else {
            self.indices.len() / 3
        }
    }

    /// Returns whether the geometry has no positions.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Validates attribute lengths, triangle arity, and index ranges.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let vertices = self.positions.len();
        for len in [
            self.normals.len(),
            self.uvs.len(),
            self.uvs2.len(),
            self.colors.len(),
            self.tangents.len(),
        ] {
            if len != 0 && len != vertices {
                return Err(ValidationError::InvalidState);
            }
        }

        if self.indices.is_empty() {
            if !vertices.is_multiple_of(3) {
                return Err(ValidationError::InvalidState);
            }
        } else {
            if !self.indices.len().is_multiple_of(3) {
                return Err(ValidationError::InvalidState);
            }
            for index in &self.indices {
                if *index as usize >= vertices {
                    return Err(ValidationError::OutOfRange);
                }
            }
        }

        Ok(())
    }

    /// Computes face-weighted vertex normals.
    pub fn compute_normals(&mut self) {
        self.normals.clear();
        self.normals.resize(self.positions.len(), Vec3::ZERO);

        if self.indices.is_empty() {
            for triangle in (0..self.positions.len()).step_by(3) {
                if triangle + 2 >= self.positions.len() {
                    break;
                }
                self.accumulate_normal(triangle, triangle + 1, triangle + 2);
            }
        } else {
            let indices = self.indices.clone();
            for triangle in indices.chunks_exact(3) {
                self.accumulate_normal(
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                );
            }
        }

        for normal in &mut self.normals {
            *normal = normal.normalize();
        }
    }

    /// Computes MikkTSpace-style tangent frames from positions, normals, and UVs.
    pub fn compute_tangents(&mut self) {
        if self.positions.is_empty() || self.uvs.len() != self.positions.len() {
            self.tangents.clear();
            return;
        }
        if self.normals.len() != self.positions.len() {
            self.compute_normals();
        }

        let mut tan1 = alloc::vec![Vec3::ZERO; self.positions.len()];
        let mut tan2 = alloc::vec![Vec3::ZERO; self.positions.len()];

        if self.indices.is_empty() {
            for triangle in (0..self.positions.len()).step_by(3) {
                if triangle + 2 >= self.positions.len() {
                    break;
                }
                accumulate_tangent(
                    self,
                    &mut tan1,
                    &mut tan2,
                    triangle,
                    triangle + 1,
                    triangle + 2,
                );
            }
        } else {
            for triangle in self.indices.chunks_exact(3) {
                accumulate_tangent(
                    self,
                    &mut tan1,
                    &mut tan2,
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                );
            }
        }

        self.tangents.clear();
        self.tangents.reserve(self.positions.len());
        for index in 0..self.positions.len() {
            let normal = self.normals[index].normalize();
            let mut tangent = (tan1[index] - normal * normal.dot(tan1[index])).normalize();
            if tangent.length_squared() <= EPSILON {
                tangent = fallback_tangent(normal);
            }
            let handedness = if normal.cross(tangent).dot(tan2[index]) < 0.0 {
                -1.0
            } else {
                1.0
            };
            self.tangents
                .push(Vec4::new(tangent.x, tangent.y, tangent.z, handedness));
        }
    }

    /// Returns the axis-aligned bounding box of the positions.
    #[inline]
    pub fn aabb(&self) -> Aabb {
        Aabb::from_points(&self.positions)
    }

    /// Returns a conservative bounding sphere as `(center, radius)`.
    #[inline]
    pub fn bounding_sphere(&self) -> (Vec3, f32) {
        let sphere = self.aabb().bounding_sphere();
        (sphere.center, sphere.radius)
    }

    /// Appends another geometry and offsets incoming indices.
    pub fn merge(&mut self, other: &Self) {
        let base = self.positions.len();
        let incoming = other.positions.len();
        let use_indices = !self.indices.is_empty() || !other.indices.is_empty();

        if use_indices && self.indices.is_empty() {
            self.indices.reserve(base + incoming);
            for index in 0..base {
                self.indices.push(index as u32);
            }
        }

        merge_vec3_attr(
            &mut self.normals,
            &other.normals,
            base,
            incoming,
            Vec3::ZERO,
        );
        merge_vec2_attr(&mut self.uvs, &other.uvs, base, incoming, Vec2::ZERO);
        merge_vec2_attr(&mut self.uvs2, &other.uvs2, base, incoming, Vec2::ZERO);
        merge_color_attr(
            &mut self.colors,
            &other.colors,
            base,
            incoming,
            Color::WHITE,
        );
        merge_vec4_attr(
            &mut self.tangents,
            &other.tangents,
            base,
            incoming,
            Vec4::ZERO,
        );

        self.positions.extend_from_slice(&other.positions);

        if use_indices {
            if other.indices.is_empty() {
                for index in 0..incoming {
                    self.indices.push((base + index) as u32);
                }
            } else {
                for index in &other.indices {
                    self.indices.push(index + base as u32);
                }
            }
        }
    }

    fn accumulate_normal(&mut self, a: usize, b: usize, c: usize) {
        if a >= self.positions.len() || b >= self.positions.len() || c >= self.positions.len() {
            return;
        }
        let p0 = self.positions[a];
        let p1 = self.positions[b];
        let p2 = self.positions[c];
        let face = (p1 - p0).cross(p2 - p0);
        if face.length_squared() <= EPSILON {
            return;
        }
        self.normals[a] += face;
        self.normals[b] += face;
        self.normals[c] += face;
    }
}

impl Bounded for Geometry {
    #[inline]
    fn aabb(&self) -> Aabb {
        self.aabb()
    }

    #[inline]
    fn bounding_sphere(&self) -> (Vec3, f32) {
        self.bounding_sphere()
    }
}

fn accumulate_tangent(
    geometry: &Geometry,
    tan1: &mut [Vec3],
    tan2: &mut [Vec3],
    a: usize,
    b: usize,
    c: usize,
) {
    if a >= geometry.positions.len()
        || b >= geometry.positions.len()
        || c >= geometry.positions.len()
    {
        return;
    }

    let p0 = geometry.positions[a];
    let p1 = geometry.positions[b];
    let p2 = geometry.positions[c];
    let uv0 = geometry.uvs[a];
    let uv1 = geometry.uvs[b];
    let uv2 = geometry.uvs[c];

    let edge1 = p1 - p0;
    let edge2 = p2 - p0;
    let duv1 = uv1 - uv0;
    let duv2 = uv2 - uv0;
    let determinant = duv1.x * duv2.y - duv2.x * duv1.y;
    if determinant.abs() <= EPSILON {
        return;
    }

    let reciprocal = 1.0 / determinant;
    let sdir = (edge1 * duv2.y - edge2 * duv1.y) * reciprocal;
    let tdir = (edge2 * duv1.x - edge1 * duv2.x) * reciprocal;

    tan1[a] += sdir;
    tan1[b] += sdir;
    tan1[c] += sdir;
    tan2[a] += tdir;
    tan2[b] += tdir;
    tan2[c] += tdir;
}

fn fallback_tangent(normal: Vec3) -> Vec3 {
    let axis = if normal.x.abs() < 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };
    normal.cross(axis).normalize()
}

fn merge_vec3_attr(
    target: &mut Vec<Vec3>,
    source: &[Vec3],
    existing: usize,
    incoming: usize,
    fill: Vec3,
) {
    merge_attr(target, source, existing, incoming, fill);
}

fn merge_vec2_attr(
    target: &mut Vec<Vec2>,
    source: &[Vec2],
    existing: usize,
    incoming: usize,
    fill: Vec2,
) {
    merge_attr(target, source, existing, incoming, fill);
}

fn merge_vec4_attr(
    target: &mut Vec<Vec4>,
    source: &[Vec4],
    existing: usize,
    incoming: usize,
    fill: Vec4,
) {
    merge_attr(target, source, existing, incoming, fill);
}

fn merge_color_attr(
    target: &mut Vec<Color>,
    source: &[Color],
    existing: usize,
    incoming: usize,
    fill: Color,
) {
    merge_attr(target, source, existing, incoming, fill);
}

fn merge_attr<T: Copy>(
    target: &mut Vec<T>,
    source: &[T],
    existing: usize,
    incoming: usize,
    fill: T,
) {
    if target.is_empty() && source.is_empty() {
        return;
    }
    if target.is_empty() {
        target.resize(existing, fill);
    }
    if source.is_empty() {
        target.resize(existing + incoming, fill);
    } else {
        target.extend_from_slice(source);
    }
}
