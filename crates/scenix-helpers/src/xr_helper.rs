//! XR pose and hand-skeleton gizmo helpers.
//!
//! XR hardware state reaches host applications as plain `Vec3`/quaternion
//! poses; scenix-helpers draws them without a scenix-xr dependency (the
//! wasm inspector and the facade own the runtime vocabulary). These
//! helpers project any tracked pose source into debug lines: an oriented
//! axis triad for controllers and head rigs, and a skeleton link pass for
//! 26-slot hand frames.

use scenix_core::Color;
use scenix_math::{Quat, Vec3};

use crate::{EPSILON, LineGeometry};

/// Number of hand joints the XR hand profile exposes.
///
/// Matches scenix-xr's `XR_HAND_JOINT_COUNT`: slot `0` is the wrist, digit
/// chains occupy four slots each (thumb `1..=4`, index `5..=8`, middle
/// `9..=12`, ring `13..=16`, pinky `17..=20`), and slots `21..25` hold
/// wrist carpals without digit links.
pub const HAND_JOINT_COUNT: usize = 26;

/// Skeleton links between hand-joint slots.
pub const HAND_LINKS: &[(usize, usize)] = &[
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (0, 5),
    (5, 6),
    (6, 7),
    (7, 8),
    (0, 9),
    (9, 10),
    (10, 11),
    (11, 12),
    (0, 13),
    (13, 14),
    (14, 15),
    (15, 16),
    (0, 17),
    (17, 18),
    (18, 19),
    (19, 20),
];

/// Oriented axis triad at one tracked rigid pose (controllers, head rigs).
///
/// Draws local +X/+Y/+Z sticks rotated by the pose orientation plus a
/// white forward stick along `-Z`, the XR convention.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoseAxesHelper {
    /// World-space origin of the triad.
    pub position: Vec3,
    /// Orientation of the tracked pose.
    pub orientation: Quat,
    /// Length of each axis stick.
    pub size: f32,
}

impl PoseAxesHelper {
    /// Creates an oriented triad helper.
    #[inline]
    pub const fn new(position: Vec3, orientation: Quat, size: f32) -> Self {
        Self {
            position,
            orientation,
            size,
        }
    }

    /// Generates line-list geometry.
    pub fn to_geometry(&self) -> LineGeometry {
        let size = self.size.abs().max(EPSILON);
        let mut geometry = LineGeometry::new();
        let x = self.orientation.mul_vec3(Vec3::X) * size;
        let y = self.orientation.mul_vec3(Vec3::Y) * size;
        let z = self.orientation.mul_vec3(Vec3::Z) * size;
        let forward = self.orientation.mul_vec3(Vec3::NEG_Z) * (size * 0.75);
        geometry.push_segment(self.position, self.position + x, Color::RED);
        geometry.push_segment(self.position, self.position + y, Color::GREEN);
        geometry.push_segment(self.position, self.position + z, Color::BLUE);
        geometry.push_segment(self.position, self.position + forward, Color::WHITE);
        geometry
    }
}

impl Default for PoseAxesHelper {
    #[inline]
    fn default() -> Self {
        Self::new(Vec3::ZERO, Quat::IDENTITY, 1.0)
    }
}

/// Hand skeleton line gizmo for one tracked 26-slot hand frame.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HandSkeletonHelper {
    /// World-space joint positions in XR hand slot order.
    pub joint_positions: [Vec3; HAND_JOINT_COUNT],
    /// Link color.
    pub color: Color,
}

impl HandSkeletonHelper {
    /// Creates a hand skeleton helper with the default link color.
    #[inline]
    pub const fn new(joint_positions: [Vec3; HAND_JOINT_COUNT]) -> Self {
        Self {
            joint_positions,
            color: Color::rgb(0.35, 0.95, 0.9),
        }
    }

    /// Sets a custom link color.
    #[inline]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Generates line-list geometry along [`HAND_LINKS`].
    pub fn to_geometry(&self) -> LineGeometry {
        let mut geometry = LineGeometry::new();
        for &(from, to) in HAND_LINKS {
            geometry.push_segment(
                self.joint_positions[from],
                self.joint_positions[to],
                self.color,
            );
        }
        geometry
    }
}
