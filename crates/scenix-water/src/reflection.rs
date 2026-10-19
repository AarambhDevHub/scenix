//! Planar reflection helpers: mirror transforms, mirrored cameras, and the
//! reflection/refraction texture flow the renderer water pipeline samples.
//!
//! Scenix water reflections are "render the mirrored world into a target,
//! then let the water shader sample it" — the same flow HDR render targets
//! already support. These helpers build the mirrored camera and the scene
//! mirror transform; mirroring flips triangle winding, which is harmless on
//! the environment pipelines (they render double-sided).

use scenix_camera::PerspectiveCamera;
use scenix_math::{Transform, Vec3};

/// Builds the transform that mirrors world geometry across the plane
/// `y == plane_height`.
///
/// `Transform::to_mat4` composes `T · S`, so translating after the Y-flip
/// produces the exact planar mirror. Apply it to terrain/sky/objects that
/// should appear reflected before rendering into the reflection target.
#[must_use]
#[inline]
pub fn mirror_transform(plane_height: f32) -> Transform {
    Transform::new(
        Vec3::new(0.0, 2.0 * plane_height, 0.0),
        scenix_math::Quat::IDENTITY,
        Vec3::new(1.0, -1.0, 1.0),
    )
}

/// Builds the camera mirrored across the water plane for a reflection pass.
///
/// Position and look target reflect through `y == plane_height`; the up
/// vector flips too so the target render is the mirror image the water
/// shader expects.
#[must_use]
#[inline]
pub fn reflected_camera(camera: &PerspectiveCamera, plane_height: f32) -> PerspectiveCamera {
    let mirror = |value: f32| 2.0 * plane_height - value;
    // `new` takes degrees and clamps; restore the exact radian fov after
    // construction so the mirror copies the projection bit-for-bit.
    let mut mirrored = PerspectiveCamera::new(
        camera.fov_y.to_degrees(),
        camera.aspect,
        camera.near,
        camera.far,
    );
    mirrored.fov_y = camera.fov_y;
    mirrored = mirrored
        .position(Vec3::new(
            camera.position.x,
            mirror(camera.position.y),
            camera.position.z,
        ))
        .target(Vec3::new(
            camera.target.x,
            mirror(camera.target.y),
            camera.target.z,
        ))
        .up(Vec3::new(camera.up.x, -camera.up.y, camera.up.z));
    mirrored
}

/// One registered reflection setup: the two texture ids to render the
/// mirrored world and the refracted underwater view into, plus how large
/// those targets should be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReflectionSetup {
    /// Texture id the mirrored scene renders into.
    pub reflection_texture: u64,
    /// Texture id the underwater scene renders into (`0` disables refraction).
    pub refraction_texture: u64,
    /// Target edge length in pixels (targets are square).
    pub size: u32,
}

impl ReflectionSetup {
    /// A minimal reflection-only setup at a 512 px target.
    #[inline]
    #[must_use]
    pub const fn minimal(reflection_texture: u64) -> Self {
        Self {
            reflection_texture,
            refraction_texture: 0,
            size: 512,
        }
    }

    /// Returns the setup with a refraction target.
    #[inline]
    #[must_use]
    pub const fn with_refraction(mut self, refraction_texture: u64) -> Self {
        self.refraction_texture = refraction_texture;
        self
    }

    /// Returns the setup with a square target edge length (clamped
    /// `64..=2048`).
    #[inline]
    #[must_use]
    pub fn with_size(mut self, size: u32) -> Self {
        self.size = size.clamp(64, 2048);
        self
    }
}

#[cfg(test)]
mod tests {
    use scenix_camera::PerspectiveCamera;
    use scenix_math::Vec3;

    use super::{ReflectionSetup, mirror_transform, reflected_camera};

    #[test]
    fn mirror_transform_reflects_points_through_the_plane() {
        use scenix_math::Vec4;

        let mirror = mirror_transform(3.0).to_mat4();
        let point = mirror.mul_vec4(Vec4::new(1.0, 5.0, 2.0, 1.0));
        assert!((point.y - 1.0).abs() < 1.0e-5, "5 → 2·3 − 5 = 1");
        assert!((point.x - 1.0).abs() < 1.0e-5 && (point.z - 2.0).abs() < 1.0e-5);
        // Below-plane points reflect upward symmetrically.
        let trough = mirror.mul_vec4(Vec4::new(0.0, 1.5, 0.0, 1.0));
        assert!((trough.y - 4.5).abs() < 1.0e-5);
    }

    #[test]
    fn reflected_camera_flips_position_target_and_up() {
        let camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 500.0)
            .position(Vec3::new(0.0, 8.0, 12.0))
            .target(Vec3::new(0.0, 4.0, 0.0))
            .up(Vec3::Y);
        let mirrored = reflected_camera(&camera, 2.0);
        assert!((mirrored.position.y - (4.0 - 8.0)).abs() < 1.0e-6);
        assert!((mirrored.target.y - (4.0 - 4.0)).abs() < 1.0e-6);
        assert!((mirrored.up.y + 1.0).abs() < 1.0e-6);
        assert!((mirrored.fov_y - camera.fov_y).abs() < 1.0e-6);
        // XZ stay untouched.
        assert!((mirrored.position.z - 12.0).abs() < 1.0e-6);
    }

    #[test]
    fn reflection_setup_carries_targets_and_size() {
        let setup = ReflectionSetup::minimal(7).with_refraction(9).with_size(9);
        assert_eq!(setup.reflection_texture, 7);
        assert_eq!(setup.refraction_texture, 9);
        assert_eq!(setup.size, 64, "size clamps into the allowed range");
    }
}
