//! Sprite and billboard quad helpers for particle rendering.
//!
//! Particle billboards and sprites are camera-facing quads. scenix-renderer
//! expands billboards on the GPU from a single vertex, but CPU-side
//! consumers — WebGPU/WebGL fallbacks, editor previews, and tests — need the
//! same quad as real geometry. [`quad_geometry`] produces that canonical
//! unit quad; its UV layout matches [`crate::primitives::plane_geometry`]
//! (bottom-left vertex carries UV `(0, 1)`).

use alloc::vec;

use scenix_math::{Vec2, Vec3};

use crate::Geometry;

/// Creates a single centered quad facing `+Z`.
///
/// The quad spans `width × height` in the XY plane with corner UVs. Invalid
/// (`non-positive`) sizes return an empty [`Geometry`].
#[must_use]
pub fn quad_geometry(width: f32, height: f32) -> Geometry {
    if width <= 0.0 || height <= 0.0 {
        return Geometry::new();
    }
    let (half_w, half_h) = (width * 0.5, height * 0.5);
    // Vertex order: bottom-left, bottom-right, top-right, top-left.
    // Corner UVs follow the plane convention: uv v == 1 at the bottom.
    let positions = [
        Vec3::new(-half_w, -half_h, 0.0),
        Vec3::new(half_w, -half_h, 0.0),
        Vec3::new(half_w, half_h, 0.0),
        Vec3::new(-half_w, half_h, 0.0),
    ];
    let uvs = [
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
    ];
    let normal = Vec3::Z;
    let mut geometry = Geometry::new();
    geometry.positions.extend(positions);
    geometry.normals.extend(vec![normal; 4]);
    geometry.uvs.extend(uvs);
    // Two CCW triangles so the quad faces +Z.
    geometry.indices.extend(vec![0, 1, 2, 0, 2, 3]);
    geometry
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use scenix_math::{Vec2, Vec3};

    use crate::quads::quad_geometry;

    #[test]
    fn quad_spans_requested_size() {
        let quad = quad_geometry(2.0, 1.0);
        assert_eq!(quad.positions.len(), 4);
        assert_eq!(quad.indices, [0, 1, 2, 0, 2, 3]);
        assert_eq!(quad.positions[0], Vec3::new(-1.0, -0.5, 0.0));
        assert_eq!(quad.positions[2], Vec3::new(1.0, 0.5, 0.0));
        assert_eq!(quad.triangle_count(), 2);
    }

    #[test]
    fn quad_corners_carry_plane_convention_uvs() {
        let quad = quad_geometry(1.0, 1.0);
        assert_eq!(quad.uvs[0], Vec2::new(0.0, 1.0), "bottom-left");
        assert_eq!(quad.uvs[2], Vec2::new(1.0, 0.0), "top-right");
    }

    #[test]
    fn quad_indices_face_positive_z() {
        let quad = quad_geometry(1.0, 1.0);
        let [a, b, c] = [quad.positions[0], quad.positions[1], quad.positions[2]];
        let e1 = b - a;
        let e2 = c - a;
        assert!(e1.cross(e2).z > 0.0, "first triangle is CCW facing +Z");
        let mut normals: Vec<Vec3> = Vec::new();
        normals.extend(quad.normals.iter().copied());
        for normal in &normals {
            assert_eq!(*normal, Vec3::Z);
        }
    }

    #[test]
    fn invalid_sizes_return_empty_geometry() {
        assert!(quad_geometry(0.0, 1.0).positions.is_empty());
        assert!(quad_geometry(1.0, -1.0).positions.is_empty());
    }
}
