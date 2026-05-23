use scenix_math::{Mat4, Ray3, Vec2, Vec3};

use crate::{Frustum, sanitize_near_far};

/// Orthographic camera with right-handed WebGPU-depth projection.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrthographicCamera {
    /// Left projection bound.
    pub left: f32,
    /// Right projection bound.
    pub right: f32,
    /// Bottom projection bound.
    pub bottom: f32,
    /// Top projection bound.
    pub top: f32,
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

impl OrthographicCamera {
    /// Creates an orthographic camera.
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let (near, far) = sanitize_near_far(near, far);
        Self {
            left,
            right,
            bottom,
            top,
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
        Mat4::orthographic(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
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

impl Default for OrthographicCamera {
    #[inline]
    fn default() -> Self {
        Self::new(-1.0, 1.0, -1.0, 1.0, 0.1, 1000.0)
    }
}
