//! Stereo/XR camera support for scenix (v1.9).
//!
//! WebXR and most OpenXR runtimes describe eye views with an *asymmetric*
//! four-angle frustum rather than a symmetric vertical FOV plus aspect
//! ratio. [`XrEyeProjection`] stores that raw per-eye frustum and builds
//! the matching projection with the same right-handed, WebGPU depth
//! conventions as [`Mat4::perspective`]. [`XrCameraRig`] couples a tracked
//! head transform with the per-eye frustum and eye separation to emit
//! left/right view-projections for stereo render targets.

use core::f32::consts::{FRAC_PI_2, FRAC_PI_3};
use scenix_math::float::{atan2, tan};
use scenix_math::{Mat4, Transform, Vec3};

/// Per-eye tangent-angle frustum in radians, as reported by XR runtimes.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrFov {
    /// Angle from the view-center ray to the left edge.
    pub angle_left: f32,
    /// Angle from the view-center ray to the right edge.
    pub angle_right: f32,
    /// Angle from the view-center ray to the up edge.
    pub angle_up: f32,
    /// Angle from the view-center ray to the down edge.
    pub angle_down: f32,
}

impl XrFov {
    /// Builds a symmetric frustum from a vertical FOV in radians and aspect.
    #[inline]
    #[must_use]
    pub fn symmetric(fov_y_rad: f32, aspect: f32) -> Self {
        let half = (fov_y_rad * 0.5).max(0.0);
        let angle_right = atan2(tan(half) * aspect.abs(), 1.0);
        Self {
            angle_left: -angle_right,
            angle_right,
            angle_up: half,
            angle_down: -half,
        }
    }

    /// Horizontal FOV coverage in radians, always positive.
    #[inline]
    #[must_use]
    pub const fn fov_x(&self) -> f32 {
        let span = self.angle_right - self.angle_left;
        if span <= 0.0 { 0.0 } else { span }
    }

    /// Vertical FOV coverage in radians, always positive.
    #[inline]
    #[must_use]
    pub const fn fov_y(&self) -> f32 {
        let span = self.angle_up - self.angle_down;
        if span <= 0.0 { 0.0 } else { span }
    }

    /// Horizontal/vertical tan ratio usable as a symmetric aspect.
    #[inline]
    #[must_use]
    pub fn aspect(&self) -> f32 {
        let tan_y = tan(self.fov_y() * 0.5);
        if tan_y.abs() <= 1.0e-6 {
            1.0
        } else {
            tan(self.fov_x() * 0.5) / tan_y
        }
    }

    /// Interpolates toward `rhs` by `t` (`0.0` = self, `1.0` = rhs).
    #[inline]
    #[must_use]
    pub fn lerp(self, rhs: Self, t: f32) -> Self {
        Self {
            angle_left: self.angle_left + (rhs.angle_left - self.angle_left) * t,
            angle_right: self.angle_right + (rhs.angle_right - self.angle_right) * t,
            angle_up: self.angle_up + (rhs.angle_up - self.angle_up) * t,
            angle_down: self.angle_down + (rhs.angle_down - self.angle_down) * t,
        }
    }
}

impl Default for XrFov {
    #[inline]
    fn default() -> Self {
        // ~100° horizontal by ~90° vertical, typical of modern headsets.
        Self {
            angle_left: -FRAC_PI_3,
            angle_right: FRAC_PI_3,
            angle_up: FRAC_PI_2 - 0.6108, // ~55° from straight up
            angle_down: -(FRAC_PI_2 - 0.6108),
        }
    }
}

/// Near/far planes plus the per-eye tangent frustum.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrEyeProjection {
    /// Tangent frustum of the eye.
    pub fov: XrFov,
    /// Near plane distance; must be `> 0`.
    pub near: f32,
    /// Far plane distance; must be `> near`.
    pub far: f32,
}

impl XrEyeProjection {
    /// Creates a projection with a symmetric `fov` and explicit planes.
    #[inline]
    #[must_use]
    pub fn symmetric(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            fov: XrFov::symmetric(fov_y_rad, aspect),
            near,
            far,
        }
    }

    /// Builds the off-center perspective matrix with WebGPU depth mapping.
    ///
    /// Degenerate planes fall back to [`Mat4::IDENTITY`], matching the
    /// house behaviour of [`Mat4::perspective`].
    #[inline]
    #[must_use]
    pub fn projection_matrix(&self) -> Mat4 {
        let near = if self.near <= 0.0 { 0.05 } else { self.near };
        let far = if self.far <= near {
            near + 1.0
        } else {
            self.far
        };
        let tan_l = tan(self.fov.angle_left);
        let tan_r = tan(self.fov.angle_right);
        let tan_t = tan(self.fov.angle_up);
        let tan_b = tan(self.fov.angle_down);
        let span_x = tan_r - tan_l;
        let span_y = tan_t - tan_b;
        let span_z = near - far;
        if span_x.abs() <= 1.0e-6 || span_y.abs() <= 1.0e-6 || span_z.abs() <= 1.0e-6 {
            return Mat4::IDENTITY;
        }
        Mat4::from_cols_array([
            2.0 / span_x,
            0.0,
            (tan_r + tan_l) / span_x,
            0.0,
            0.0,
            2.0 / span_y,
            (tan_t + tan_b) / span_y,
            0.0,
            0.0,
            0.0,
            far / span_z,
            -1.0,
            0.0,
            0.0,
            (near * far) / span_z,
            0.0,
        ])
    }
}

impl Default for XrEyeProjection {
    #[inline]
    fn default() -> Self {
        Self {
            fov: XrFov::default(),
            near: 0.05,
            far: 500.0,
        }
    }
}

/// Head transform coupled with per-eye stereo projections and offsets.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrCameraRig {
    /// Tracked head transform in tracking space.
    pub head: Transform,
    /// Per-eye projection planes.
    pub eye: XrEyeProjection,
    /// Horizontal eye separation from the head axis; magnitude is the IPD.
    pub eye_separation: f32,
}

impl XrCameraRig {
    /// Creates a rig with default headset values.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            head: Transform::IDENTITY,
            eye: XrEyeProjection::default(),
            eye_separation: 0.0635,
        }
    }

    /// Returns the rig with a new head transform.
    #[inline]
    #[must_use]
    pub const fn with_head(mut self, head: Transform) -> Self {
        self.head = head;
        self
    }

    /// Returns the rig with new per-eye projection planes.
    #[inline]
    #[must_use]
    pub const fn with_eye(mut self, eye: XrEyeProjection) -> Self {
        self.eye = eye;
        self
    }

    /// Returns the rig with a new eye separation (interpupillary distance).
    #[inline]
    #[must_use]
    pub const fn with_eye_separation(mut self, separation: f32) -> Self {
        self.eye_separation = if separation < 0.0 { 0.0 } else { separation };
        self
    }

    /// World-space origin of the left eye.
    #[inline]
    #[must_use]
    pub fn left_position(&self) -> Vec3 {
        self.eye_position(-1.0)
    }

    /// World-space origin of the right eye.
    #[inline]
    #[must_use]
    pub fn right_position(&self) -> Vec3 {
        self.eye_position(1.0)
    }

    /// View matrix for the left eye.
    #[inline]
    #[must_use]
    pub fn left_view(&self) -> Mat4 {
        self.eye_view(-1.0)
    }

    /// View matrix for the right eye.
    #[inline]
    #[must_use]
    pub fn right_view(&self) -> Mat4 {
        self.eye_view(1.0)
    }

    /// Off-center projection for the left eye.
    #[inline]
    #[must_use]
    pub fn left_projection(&self) -> Mat4 {
        self.eye.projection_matrix()
    }

    /// Off-center projection for the right eye.
    #[inline]
    #[must_use]
    pub fn right_projection(&self) -> Mat4 {
        self.eye.projection_matrix()
    }

    /// Projection times view for the left eye.
    #[inline]
    #[must_use]
    pub fn left_view_projection(&self) -> Mat4 {
        self.left_projection() * self.left_view()
    }

    /// Projection times view for the right eye.
    #[inline]
    #[must_use]
    pub fn right_view_projection(&self) -> Mat4 {
        self.right_projection() * self.right_view()
    }

    /// Builds the symmetric-equivalent left-eye camera.
    #[inline]
    #[must_use]
    pub fn left_camera(&self) -> crate::PerspectiveCamera {
        eye_camera(self.eye, self.left_position(), self.head)
    }

    /// Builds the symmetric-equivalent right-eye camera.
    #[inline]
    #[must_use]
    pub fn right_camera(&self) -> crate::PerspectiveCamera {
        eye_camera(self.eye, self.right_position(), self.head)
    }

    #[inline]
    fn eye_position(&self, side: f32) -> Vec3 {
        self.head.translation + (self.head.right() * (self.eye_separation * 0.5 * side))
    }

    #[inline]
    fn eye_view(&self, side: f32) -> Mat4 {
        let eye_transform =
            Transform::new(self.eye_position(side), self.head.rotation, self.head.scale);
        Mat4::look_at(
            eye_transform.translation,
            eye_transform.translation + eye_transform.forward(),
            eye_transform.up(),
        )
    }
}

impl Default for XrCameraRig {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Symmetric-equivalent camera for one eye, aimed along the head gaze.
#[inline]
#[must_use]
fn eye_camera(eye: XrEyeProjection, origin: Vec3, head: Transform) -> crate::PerspectiveCamera {
    let aspect = eye.fov.aspect().max(0.01);
    let fov_y_deg: f32 = if eye.fov.aspect() < 1.0 {
        // Square-ish frustums flip the dominant axis; keep FOV sane.
        (eye.fov.fov_x() * 180.0 / core::f32::consts::PI).clamp(1.0, 170.0)
    } else {
        (eye.fov.fov_y() * 180.0 / core::f32::consts::PI).clamp(1.0, 170.0)
    };
    crate::PerspectiveCamera::new(fov_y_deg, aspect, eye.near, eye.far)
        .position(origin)
        .target(origin + head.forward())
        .up(head.up())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::FRAC_PI_2;
    use scenix_math::Quat;

    #[test]
    fn symmetric_fov_roundtrip() {
        let fov = XrFov::symmetric(FRAC_PI_2, 1.0);
        assert!((fov.fov_x() - FRAC_PI_2).abs() < 1e-4);
        assert!((fov.fov_y() - FRAC_PI_2).abs() < 1e-4);
        assert!((fov.aspect() - 1.0).abs() < 1e-4);
        let wide = XrFov::symmetric(FRAC_PI_2, 2.0);
        assert!(wide.fov_x() > wide.fov_y());
        assert!((wide.aspect() - 2.0).abs() < 1e-3);
    }

    #[test]
    fn projection_matches_symmetric_perspective() {
        let eye = XrEyeProjection::symmetric(FRAC_PI_2, 1.6, 0.1, 100.0);
        let ours = eye.projection_matrix();
        let reference = Mat4::perspective(FRAC_PI_2, 1.6, 0.1, 100.0);
        for row in 0..4 {
            for col in 0..4 {
                let a = ours.get(row, col);
                let b = reference.get(row, col);
                assert!(
                    (a - b).abs() < 1e-4,
                    "mismatch at ({row}, {col}): {a} vs {b}"
                );
            }
        }
    }

    #[test]
    fn off_center_shifts_optical_axis() {
        let mut fov = XrFov::symmetric(FRAC_PI_2, 1.0);
        fov.angle_right += 0.1; // right edge pushed out
        let eye = XrEyeProjection {
            fov,
            near: 0.1,
            far: 100.0,
        };
        let matrix = eye.projection_matrix();
        // Third column, second row = (r+l)/(r-l) shifts positive.
        assert!(matrix.get(2, 0) > 0.0, "optical axis should shift right");
    }

    #[test]
    fn degenerate_planes_fall_back_to_identity() {
        // Degenerate planes get sanitized into a valid matrix...
        let mut eye = XrEyeProjection::default();
        eye.far = eye.near;
        assert!(eye.projection_matrix().get(0, 0).is_finite());
        // ...but a degenerate frustum falls back to identity.
        let mut flat = XrEyeProjection::default();
        flat.fov.angle_right = flat.fov.angle_left;
        assert_eq!(flat.projection_matrix(), Mat4::IDENTITY);
    }

    #[test]
    fn rig_positions_separate_eyes() {
        let head = Transform::new(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE);
        let rig = XrCameraRig::new()
            .with_head(head)
            .with_eye_separation(0.064);
        assert!((rig.left_position().x - (1.0 - 0.032)).abs() < 1e-5);
        assert!((rig.right_position().x - (1.0 + 0.032)).abs() < 1e-5);
        assert!((rig.left_position().y - 2.0).abs() < 1e-5);
        assert!(!rig.left_view_projection().get(3, 3).is_nan());
        assert!(!rig.right_view_projection().get(3, 3).is_nan());
    }

    #[test]
    fn fov_lerp_and_default() {
        let mid = XrFov::symmetric(1.0, 1.0).lerp(XrFov::symmetric(2.0, 1.0), 0.5);
        assert!((mid.fov_y() - 1.5).abs() < 1e-4);
        let d = XrFov::default();
        assert!((d.fov_x() - (d.angle_right - d.angle_left)).abs() < 1e-6);
    }

    #[test]
    fn rig_eye_cameras_pid_away() {
        let head = Transform::new(
            Vec3::new(0.0, 1.6, 4.0),
            Quat::from_axis_angle(scenix_math::Vec3::Y, -FRAC_PI_2),
            Vec3::ONE,
        );
        let rig = XrCameraRig::new().with_head(head);
        let left = rig.left_camera();
        let right = rig.right_camera();
        // Both eye cameras sit apart in world space, aimed parallel.
        let apart = (right.position - left.position).length();
        assert!(
            apart > 0.05,
            "eye cameras should be separated, apart={apart}"
        );
        assert!((left.fov_y - right.fov_y).abs() < 1e-6);
    }
}
