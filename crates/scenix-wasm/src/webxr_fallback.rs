//! WebXR runtime fallback planning.
//!
//! When `navigator.xr` is missing, cannot grant the session, or the user
//! denies the permission prompt, the front-end still needs a deterministic
//! presentation decision plus an emulated headset rig so demos remain
//! usable. This module turns a session-request outcome into a CPU plan;
//! browser glue owns the actual runtime probing and DOM wiring.

use scenix_camera::{XrCameraRig, XrEyeProjection};
use scenix_math::{Quat, Transform, Vec3};
use scenix_xr::XrPose;

/// Presentation decision after the browser runtime probe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WebXrFallbackPlan {
    /// A live runtime granted the session; scenix-xr plans drive frames.
    #[default]
    RuntimeGranted,
    /// No runtime: the canvas keeps rendering with a desktop camera and
    /// an emulated controller bay fed from keyboard/pointer input.
    EmulatedDesktop,
}

impl WebXrFallbackPlan {
    /// Whether the pipeline should stream both immersive eye channels.
    #[inline]
    #[must_use]
    pub const fn immersive(self) -> bool {
        matches!(self, Self::RuntimeGranted)
    }

    /// Whether an emulated controller should mirror the desktop cursor.
    #[inline]
    #[must_use]
    pub const fn emulated_controller(self) -> bool {
        matches!(self, Self::EmulatedDesktop)
    }

    /// Visible label for the loader toast.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RuntimeGranted => "xr-runtime",
            Self::EmulatedDesktop => "xr-emulated-desktop",
        }
    }
}

/// Resolves the presentation plan for a session request outcome.
///
/// `granted` mirrors `navigator.xr.requestSession(...)`: granted requests
/// stream immersive presentation, everything else falls back to the
/// deterministic desktop emulation so a missing runtime never blocks the
/// demo from rendering.
#[inline]
#[must_use]
pub const fn webxr_fallback_plan(granted: bool) -> WebXrFallbackPlan {
    if granted {
        WebXrFallbackPlan::RuntimeGranted
    } else {
        WebXrFallbackPlan::EmulatedDesktop
    }
}

/// Builds the emulated desk headset rig used when no runtime granted the
/// session.
///
/// The head parks standing desk height 2.4 m back from the origin looking
/// down `-Z` at the scene center; both eyes share the given projection
/// with the headset-standard 64 mm separation.
#[must_use]
pub fn emulated_headset_rig(projection: XrEyeProjection) -> XrCameraRig {
    XrCameraRig::new()
        .with_head(Transform::new(
            Vec3::new(0.0, 1.6, 2.4),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_eye(projection)
        .with_eye_separation(0.064)
}

/// Emulated right-hand controller pose for the fallback bay: one arm's
/// length down-forward from the head along the head's own axes.
#[must_use]
pub fn emulated_controller_pose(head: &Transform) -> XrPose {
    let forward = head.rotation.mul_vec3(Vec3::NEG_Z);
    let down = head.rotation.mul_vec3(-Vec3::Y);
    XrPose {
        position: head.translation + forward * 0.7 + down * 0.35,
        orientation: head.rotation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn granted_sessions_go_immersive() {
        let granted = webxr_fallback_plan(true);
        assert!(granted.immersive());
        assert!(!granted.emulated_controller());
        assert_eq!(granted.label(), "xr-runtime");

        let denied = webxr_fallback_plan(false);
        assert!(!denied.immersive());
        assert!(denied.emulated_controller());
        assert_eq!(denied.label(), "xr-emulated-desktop");
    }

    #[test]
    fn emulated_rig_and_controller_are_desktop_deterministic() {
        let rig = emulated_headset_rig(XrEyeProjection::symmetric(1.2, 1.0, 0.05, 50.0));
        assert_eq!(
            rig.head.translation,
            Vec3::new(0.0, 1.6, 2.4),
            "the emulated head parks at the standing desk pose"
        );
        assert!((rig.eye_separation - 0.064).abs() < 1.0e-6);

        let pose = emulated_controller_pose(&rig.head);
        assert!((pose.position.z - (2.4 - 0.7)).abs() < 1.0e-5);
        assert!((pose.position.y - (1.6 - 0.35)).abs() < 1.0e-5);
        assert_eq!(pose.orientation, rig.head.rotation);
    }
}
