use alloc::vec::Vec;

use scenix_core::ValidationError;
use scenix_math::Transform;

use crate::{QuatTrack, Vec3Track};

/// A CPU skeleton pose as a flat array of bone transforms.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkeletonPose {
    /// Bone transforms in caller-defined joint order.
    pub bones: Vec<Transform>,
}

impl SkeletonPose {
    /// Creates a pose from bone transforms.
    #[inline]
    pub const fn new(bones: Vec<Transform>) -> Self {
        Self { bones }
    }

    /// Creates an identity pose with `len` bones.
    pub fn identity(len: usize) -> Self {
        Self {
            bones: alloc::vec![Transform::IDENTITY; len],
        }
    }

    /// Returns the number of bones.
    #[inline]
    pub fn len(&self) -> usize {
        self.bones.len()
    }

    /// Returns whether the pose contains no bones.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bones.is_empty()
    }
}

/// Bone transform fields that can be animated.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BoneAnimationTarget {
    /// Animate local translation.
    Translation(Vec3Track),
    /// Animate local rotation.
    Rotation(QuatTrack),
    /// Animate local scale.
    Scale(Vec3Track),
}

impl BoneAnimationTarget {
    /// Advances the contained track.
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::Translation(track) | Self::Scale(track) => track.update(dt),
            Self::Rotation(track) => track.update(dt),
        }
    }

    /// Returns whether the contained track has completed.
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Translation(track) | Self::Scale(track) => track.is_complete(),
            Self::Rotation(track) => track.is_complete(),
        }
    }
}

/// Animation for one bone in a [`SkeletonPose`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoneAnimation {
    /// Bone index in the target pose.
    pub bone_index: usize,
    /// Field being animated.
    pub target: BoneAnimationTarget,
}

impl BoneAnimation {
    /// Creates a bone animation.
    #[inline]
    pub const fn new(bone_index: usize, target: BoneAnimationTarget) -> Self {
        Self { bone_index, target }
    }

    /// Advances the animator, applies the current value, and returns completion.
    pub fn update(&mut self, dt: f32, pose: &mut SkeletonPose) -> Result<bool, ValidationError> {
        self.target.update(dt);
        let bone = pose
            .bones
            .get_mut(self.bone_index)
            .ok_or(ValidationError::InvalidId)?;
        match &self.target {
            BoneAnimationTarget::Translation(track) => {
                bone.translation = track.value();
            }
            BoneAnimationTarget::Rotation(track) => {
                bone.rotation = track.value();
            }
            BoneAnimationTarget::Scale(track) => {
                bone.scale = track.value();
            }
        }
        Ok(self.target.is_complete())
    }
}

/// Drives one skeleton pose by index in the caller's skeleton store.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkinnedMeshAnimator {
    /// Target pose index in the driver's skeleton slice.
    pub skeleton_index: usize,
    /// Bone animations to update in deterministic order.
    pub bones: Vec<BoneAnimation>,
}

impl SkinnedMeshAnimator {
    /// Creates a skeleton animator.
    #[inline]
    pub const fn new(skeleton_index: usize, bones: Vec<BoneAnimation>) -> Self {
        Self {
            skeleton_index,
            bones,
        }
    }

    /// Adds a bone animation.
    #[inline]
    pub fn push(&mut self, animation: BoneAnimation) {
        self.bones.push(animation);
    }

    /// Advances all bone animations and returns completion when every track is done.
    pub fn update(
        &mut self,
        dt: f32,
        skeletons: &mut [SkeletonPose],
    ) -> Result<bool, ValidationError> {
        let pose = skeletons
            .get_mut(self.skeleton_index)
            .ok_or(ValidationError::InvalidId)?;
        let mut all_complete = true;
        for animation in &mut self.bones {
            all_complete &= animation.update(dt, pose)?;
        }
        Ok(all_complete)
    }
}
