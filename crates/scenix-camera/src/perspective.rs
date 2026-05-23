use scenix_math::{Mat4, Ray3, Vec2, Vec3};

use crate::{Frustum, clamp, sanitize_near_far};

/// Perspective camera with right-handed WebGPU-depth projection.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PerspectiveCamera {
    /// Vertical field of view in radians.
    pub fov_y: f32,
    /// Aspect ratio, width divided by height.
    pub aspect: f32,
    /// Near clipping plane distance.
    pub near: f32,
    /// Far clipping plane distance.
    pub far: f32,
    /// Camera position in world space.
    pub position: Vec3,
    /// Look target in world space.
    pub target: Vec3,
    /// Up direction.
    pub up: Vec3,
}

impl PerspectiveCamera {
    /// Creates a camera from a vertical field of view in degrees.
    pub fn new(fov_y_deg: f32, aspect: f32, near: f32, far: f32) -> Self {
        let (near, far) = sanitize_near_far(near, far);
        let min_fov = core::f32::consts::PI / 180.0;
        let max_fov = 179.0 * core::f32::consts::PI / 180.0;
        Self {
            fov_y: clamp(fov_y_deg * core::f32::consts::PI / 180.0, min_fov, max_fov),
            aspect: if aspect.abs() > crate::EPSILON {
                aspect.abs()
            } else {
                1.0
            },
            near,
            far,
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
        }
    }

    /// Returns this camera with a world-space position.
    #[inline]
    pub const fn position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Returns this camera with a look target.
    #[inline]
    pub const fn target(mut self, target: Vec3) -> Self {
        self.target = target;
        self
    }

    /// Returns this camera with an up vector.
    #[inline]
    pub const fn up(mut self, up: Vec3) -> Self {
        self.up = up;
        self
    }

    /// Returns the projection matrix.
    #[inline]
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov_y, self.aspect, self.near, self.far)
    }

    /// Returns the view matrix.
    #[inline]
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.target, self.up)
    }

    /// Returns projection multiplied by view.
    #[inline]
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Extracts the view frustum.
    #[inline]
    pub fn frustum(&self) -> Frustum {
        Frustum::from_view_projection(self.view_projection())
    }

    /// Builds a ray from normalized device coordinates in `[-1, 1]`.
    pub fn screen_to_ray(&self, ndc: Vec2) -> Ray3 {
        let inverse = self.view_projection().inverse().unwrap_or(Mat4::IDENTITY);
        let near = inverse.mul_vec3(Vec3::new(ndc.x, ndc.y, 0.0));
        let far = inverse.mul_vec3(Vec3::new(ndc.x, ndc.y, 1.0));
        Ray3::new(near, far - near)
    }
}

impl Default for PerspectiveCamera {
    #[inline]
    fn default() -> Self {
        Self::new(60.0, 1.0, 0.1, 1000.0)
    }
}
