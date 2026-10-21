//! Tracked controller poses and the fixed two-slot controller bay.

use scenix_input::{XrHand, XrTrackingStatus};
use scenix_math::{Quat, Transform, Vec3};

/// A tracked 6DoF pose: position plus orientation in tracking space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrPose {
    /// Position in the session's reference space.
    pub position: Vec3,
    /// Orientation around the tracking origin.
    pub orientation: Quat,
}

impl XrPose {
    /// Creates a pose from position and orientation.
    #[inline]
    #[must_use]
    pub const fn new(position: Vec3, orientation: Quat) -> Self {
        Self {
            position,
            orientation,
        }
    }

    /// Creates a translation-only pose with identity orientation.
    #[inline]
    #[must_use]
    pub fn from_position(position: Vec3) -> Self {
        Self::new(position, Quat::IDENTITY)
    }

    /// Returns the pose as a scene `Transform`.
    #[inline]
    #[must_use]
    pub const fn to_transform(self) -> Transform {
        Transform::new(self.position, self.orientation, Vec3::ONE)
    }

    /// Interpolates toward `rhs` by `t` using lerp + slerp.
    #[inline]
    #[must_use]
    pub fn lerp(self, rhs: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(rhs.position, t),
            orientation: self.orientation.slerp(rhs.orientation, t),
        }
    }

    /// Moves a point from pose-local space to tracking space.
    #[inline]
    #[must_use]
    pub fn transform_point(self, point: Vec3) -> Vec3 {
        self.orientation * point + self.position
    }

    /// Gaze direction after orientation.
    #[inline]
    #[must_use]
    pub fn forward(self) -> Vec3 {
        self.orientation * Vec3::NEG_Z
    }

    /// Right axis after orientation.
    #[inline]
    #[must_use]
    pub fn right(self) -> Vec3 {
        self.orientation * Vec3::X
    }

    /// Up axis after orientation.
    #[inline]
    #[must_use]
    pub fn up(self) -> Vec3 {
        self.orientation * Vec3::Y
    }
}

/// Which transform an XR input source exposes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrSpace {
    /// The aiming ray origin; keypoints map to `pointer` space.
    #[default]
    TargetRay,
    /// The hand holding the controller; grip content anchors here.
    Grip,
}

impl XrSpace {
    /// Returns whether the space is used for holding rendered props.
    #[inline]
    #[must_use]
    pub const fn is_grip(self) -> bool {
        matches!(self, Self::Grip)
    }
}

/// One tracked controller: pose, motion, and tracking quality.
///
/// Button and axis values stay in `scenix-input`'s `XrControllerInput`;
/// pairing happens in application frame code so neither crate duplicates
/// the other's vocabulary.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrController {
    /// Which hand the controller is in.
    pub hand: XrHand,
    /// Pointer-ray pose, `XrSpace::TargetRay` semantics.
    pub pointer: XrPose,
    /// Grip pose, `XrSpace::Grip` semantics.
    pub grip: XrPose,
    /// Linear velocity in m/s in tracking space.
    pub linear_velocity: Vec3,
    /// Angular velocity in rad/s around each tracking axis.
    pub angular_velocity: Vec3,
    /// Pose quality reported this frame.
    pub tracking: XrTrackingStatus,
}

impl XrController {
    /// Creates a controller for `hand` at identity pose.
    #[inline]
    #[must_use]
    pub fn idle(hand: XrHand) -> Self {
        Self {
            hand,
            pointer: XrPose::default(),
            grip: XrPose::default(),
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            tracking: XrTrackingStatus::Untracked,
        }
    }

    /// Updates the controller's pointer and grip poses for this frame.
    ///
    /// Grip pose is derived from the pointer pose by rotating the grip
    /// backwards along the ray and mirroring it so held content paints
    /// into the hand, the deterministic fallback runtimes use when they
    /// only publish target-ray space.
    #[must_use]
    pub fn with_pose(mut self, pose: XrPose, tracking: XrTrackingStatus) -> Self {
        self.pointer = pose;
        self.tracking = tracking;
        let back = Quat::from_axis_angle(Vec3::Y, 0.35) * Quat::from_axis_angle(Vec3::X, -0.25);
        self.grip = XrPose::new(
            pose.transform_point(Vec3::new(0.0, -0.05, -0.03)),
            pose.orientation * back,
        );
        self
    }

    /// Returns the pose for `space`.
    #[inline]
    #[must_use]
    pub const fn pose(&self, space: XrSpace) -> &XrPose {
        match space {
            XrSpace::TargetRay => &self.pointer,
            XrSpace::Grip => &self.grip,
        }
    }

    /// Returns whether this controller's pose is trustworthy this frame.
    #[inline]
    #[must_use]
    pub const fn is_tracking(&self) -> bool {
        self.tracking.is_usable()
    }

    /// Predicts the pointer pose `dt` seconds ahead using current velocity.
    #[must_use]
    pub fn predicted_pose(&self, dt: f32) -> XrPose {
        let dt = dt.clamp(0.0, 0.25);
        let dt_axis = self.angular_velocity * dt;
        let yaw = dt_axis.y * 0.5;
        let pitch = dt_axis.x * 0.5;
        let roll = dt_axis.z * 0.5;
        let drift = Quat::from_axis_angle(Vec3::Y, yaw)
            * Quat::from_axis_angle(Vec3::X, pitch)
            * Quat::from_axis_angle(Vec3::Z, roll);
        XrPose {
            position: self.pointer.position + self.linear_velocity * dt,
            orientation: drift * self.pointer.orientation,
        }
    }
}

/// Fixed two-slot bay keyed by hand; the pose side of the XR frame loop.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrControllerBay {
    slots: [XrController; 2],
}

impl XrControllerBay {
    /// Creates a bay with both hands idle.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: [
                XrController::idle(XrHand::Left),
                XrController::idle(XrHand::Right),
            ],
        }
    }

    /// Returns the controller for `hand`.
    #[inline]
    #[must_use]
    pub fn get(&self, hand: XrHand) -> &XrController {
        &self.slots[hand.index()]
    }

    /// Returns the mutable controller for `hand`.
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, hand: XrHand) -> &mut XrController {
        &mut self.slots[hand.index()]
    }

    /// Replaces the pose for `hand`, mirroring scenix-input's reset flow.
    #[inline]
    pub fn update(&mut self, hand: XrHand, pose: XrPose, tracking: XrTrackingStatus) {
        self.slots[hand.index()] = self.slots[hand.index()].with_pose(pose, tracking);
    }

    /// Returns the aiming ray of `hand` as `(origin, direction)`.
    #[inline]
    #[must_use]
    pub fn aim_ray(&self, hand: XrHand) -> (Vec3, Vec3) {
        let pose = self.get(hand).pose(XrSpace::TargetRay);
        (pose.position, pose.forward())
    }

    /// Returns the number of slots currently publishing usable poses.
    #[inline]
    #[must_use]
    pub const fn tracked(&self) -> usize {
        let mut count = 0;
        if self.slots[0].is_tracking() {
            count += 1;
        }
        if self.slots[1].is_tracking() {
            count += 1;
        }
        count
    }

    /// Marks every slot untracked without touching analog input state.
    #[inline]
    pub fn reset(&mut self) {
        for hand in [XrHand::Left, XrHand::Right] {
            let slot = &mut self.slots[hand.index()];
            slot.tracking = XrTrackingStatus::Untracked;
            slot.linear_velocity = Vec3::ZERO;
            slot.angular_velocity = Vec3::ZERO;
        }
    }
}
