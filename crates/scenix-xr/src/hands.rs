//! Articulated hand tracking frames.

use scenix_input::{XrHand, XrTrackingStatus};
use scenix_math::{Quat, Vec3};

use crate::controllers::XrPose;

/// Number of hand joints the XR hand-tracking profile exposes.
pub const XR_HAND_JOINT_COUNT: usize = 26;

/// Predicted joint slot indices for common pinch/grab detection.
pub const JOINT_THUMB_TIP: usize = 4;
pub const JOINT_INDEX_TIP: usize = 8;
/// Joint-list position of the wrist.
pub const JOINT_WRIST: usize = 0;

/// One tracked hand joint: pose plus contact radius.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrJointPose {
    /// Joint pose in tracking space.
    pub pose: XrPose,
    /// Estimated contact radius in meters.
    pub radius: f32,
    /// Whether the runtime reports this joint as tracked right now.
    pub tracked: bool,
}

impl XrJointPose {
    /// Creates a tracked joint pose with a radius.
    #[inline]
    #[must_use]
    pub const fn tracked(pose: XrPose, radius: f32) -> Self {
        Self {
            pose,
            radius,
            tracked: true,
        }
    }

    /// Creates an untracked placeholder joint.
    #[inline]
    #[must_use]
    pub const fn untracked() -> Self {
        Self {
            pose: XrPose::new(Vec3::ZERO, Quat::IDENTITY),
            radius: 0.0,
            tracked: false,
        }
    }
}

/// One hand's skeleton frame plus grip/pinch predictions.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrHandFrame {
    /// Which hand this frame describes.
    pub hand: XrHand,
    /// Per-joint poses, index-matched to the joint list in runtimes.
    ///
    /// Slot `0` is the wrist, followed by carpals, then digits.
    pub joints: [XrJointPose; XR_HAND_JOINT_COUNT],
    /// Grip strength `0..=1` from a closed-fist classifier.
    pub grip_strength: f32,
    /// Pinch strength `0..=1` from a thumb-index distance classifier.
    pub pinch_strength: f32,
    /// Pose quality reported this frame.
    pub tracking: XrTrackingStatus,
}

impl XrHandFrame {
    /// Creates an untracked hand frame for `hand`.
    #[inline]
    #[must_use]
    pub fn idle(hand: XrHand) -> Self {
        Self {
            hand,
            joints: core::array::from_fn(|_| XrJointPose::untracked()),
            grip_strength: 0.0,
            pinch_strength: 0.0,
            tracking: XrTrackingStatus::Untracked,
        }
    }

    /// Returns the joint pose for `index`, `None` when out of bounds or
    /// untracked.
    #[inline]
    #[must_use]
    pub fn joint(&self, index: usize) -> Option<&XrJointPose> {
        self.joints.get(index).filter(|joint| joint.tracked)
    }

    /// Replaces one joint's pose and marks it tracked.
    #[inline]
    pub fn track_joint(&mut self, index: usize, pose: XrPose, radius: f32) {
        if let Some(slot) = self.joints.get_mut(index) {
            *slot = XrJointPose::tracked(pose, radius);
        }
    }

    /// Returns the thumb-tip to index-tip distance in meters, `None` when
    /// either endpoint is untracked.
    #[inline]
    #[must_use]
    pub fn thumb_index_distance(&self) -> Option<f32> {
        let thumb = self.joint(JOINT_THUMB_TIP)?.pose.position;
        let index = self.joint(JOINT_INDEX_TIP)?.pose.position;
        Some((index - thumb).length())
    }

    /// Returns how many joints are tracked in this frame.
    #[inline]
    #[must_use]
    pub fn tracked_joints(&self) -> usize {
        self.joints.iter().filter(|joint| joint.tracked).count()
    }

    /// Returns the world positions of all tracked joints, nearest first
    /// alternating left-to-right for deterministic ordering.
    #[must_use]
    pub fn tracked_positions(&self) -> alloc::vec::Vec<Vec3> {
        let mut positions: alloc::vec::Vec<Vec3> = self
            .joints
            .iter()
            .filter(|joint| joint.tracked)
            .map(|joint| joint.pose.position)
            .collect();
        positions.sort_by(|a, b| {
            a.x.total_cmp(&b.x)
                .then(a.y.total_cmp(&b.y))
                .then(a.z.total_cmp(&b.z))
        });
        positions
    }

    /// Returns whether the hand's grip classifier fired strongly.
    #[inline]
    #[must_use]
    pub fn is_gripping(&self) -> bool {
        self.grip_strength >= 0.5
    }

    /// Returns whether the thumb-index pinch classifier fired strongly.
    #[inline]
    #[must_use]
    pub fn is_pinching(&self) -> bool {
        self.pinch_strength >= 0.5
    }
}

impl Default for XrHandFrame {
    #[inline]
    fn default() -> Self {
        Self::idle(XrHand::Left)
    }
}
