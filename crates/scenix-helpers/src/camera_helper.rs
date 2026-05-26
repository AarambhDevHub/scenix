use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::Color;
use scenix_math::{Mat4, Vec3};

use crate::LineGeometry;

/// Camera frustum wireframe helper.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CameraHelper {
    /// Camera view-projection matrix.
    pub view_projection: Mat4,
    /// Line color.
    pub color: Color,
}

impl CameraHelper {
    /// Creates a helper from a view-projection matrix.
    #[inline]
    pub const fn new(view_projection: Mat4, color: Color) -> Self {
        Self {
            view_projection,
            color,
        }
    }

    /// Creates a helper from a perspective camera.
    #[inline]
    pub fn from_perspective(camera: &PerspectiveCamera, color: Color) -> Self {
        Self::new(camera.view_projection(), color)
    }

    /// Creates a helper from an orthographic camera.
    #[inline]
    pub fn from_orthographic(camera: &OrthographicCamera, color: Color) -> Self {
        Self::new(camera.view_projection(), color)
    }

    /// Generates frustum edge geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let inverse = self.view_projection.inverse().unwrap_or(Mat4::IDENTITY);
        let ndc = [
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-1.0, 1.0, 0.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
        ];
        let mut corners = [Vec3::ZERO; 8];
        for (out, corner) in corners.iter_mut().zip(ndc) {
            *out = inverse.mul_vec3(corner);
        }

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
        for (a, b) in edges {
            geometry.push_segment(corners[a], corners[b], self.color);
        }
        geometry
    }
}
