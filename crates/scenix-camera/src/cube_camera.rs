use scenix_math::{Mat4, Vec3};

use crate::sanitize_near_far;

/// Cube-map face order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CubeFace {
    /// Positive X face.
    PositiveX,
    /// Negative X face.
    NegativeX,
    /// Positive Y face.
    PositiveY,
    /// Negative Y face.
    NegativeY,
    /// Positive Z face.
    PositiveZ,
    /// Negative Z face.
    NegativeZ,
}

/// Camera that produces six 90-degree view-projection matrices for cubemaps.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CubeCamera {
    /// Capture position.
    pub position: Vec3,
    /// Near clipping plane distance.
    pub near: f32,
    /// Far clipping plane distance.
    pub far: f32,
}

impl CubeFace {
    /// Returns all cube faces in storage order.
    #[inline]
    pub const fn all() -> [Self; 6] {
        [
            Self::PositiveX,
            Self::NegativeX,
            Self::PositiveY,
            Self::NegativeY,
            Self::PositiveZ,
            Self::NegativeZ,
        ]
    }

    /// Returns the face direction and up vector.
    #[inline]
    pub const fn basis(self) -> (Vec3, Vec3) {
        match self {
            Self::PositiveX => (Vec3::X, Vec3::new(0.0, -1.0, 0.0)),
            Self::NegativeX => (Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
            Self::PositiveY => (Vec3::Y, Vec3::Z),
            Self::NegativeY => (Vec3::new(0.0, -1.0, 0.0), Vec3::NEG_Z),
            Self::PositiveZ => (Vec3::Z, Vec3::new(0.0, -1.0, 0.0)),
            Self::NegativeZ => (Vec3::NEG_Z, Vec3::new(0.0, -1.0, 0.0)),
        }
    }
}

impl CubeCamera {
    /// Creates a cube camera.
    #[inline]
    pub fn new(position: Vec3, near: f32, far: f32) -> Self {
        let (near, far) = sanitize_near_far(near, far);
        Self {
            position,
            near,
            far,
        }
    }

    /// Returns the projection matrix shared by all faces.
    #[inline]
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(core::f32::consts::FRAC_PI_2, 1.0, self.near, self.far)
    }

    /// Returns the view matrix for a face.
    #[inline]
    pub fn view_matrix(&self, face: CubeFace) -> Mat4 {
        let (direction, up) = face.basis();
        Mat4::look_at(self.position, self.position + direction, up)
    }

    /// Returns projection multiplied by view for a face.
    #[inline]
    pub fn view_projection(&self, face: CubeFace) -> Mat4 {
        self.projection_matrix() * self.view_matrix(face)
    }

    /// Returns all six view-projection matrices.
    #[inline]
    pub fn view_projections(&self) -> [Mat4; 6] {
        let faces = CubeFace::all();
        [
            self.view_projection(faces[0]),
            self.view_projection(faces[1]),
            self.view_projection(faces[2]),
            self.view_projection(faces[3]),
            self.view_projection(faces[4]),
            self.view_projection(faces[5]),
        ]
    }
}

impl Default for CubeCamera {
    #[inline]
    fn default() -> Self {
        Self::new(Vec3::ZERO, 0.1, 1000.0)
    }
}
