use scenix_core::{NodeId, ValidationError};
use scenix_scene::SceneGraph;

use crate::{BoolTrack, QuatTrack, Vec3Track};

/// Scene node fields that can be driven by an animation track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeAnimationTarget {
    /// Animate local translation.
    Translation(Vec3Track),
    /// Animate local rotation.
    Rotation(QuatTrack),
    /// Animate local scale.
    Scale(Vec3Track),
    /// Animate visibility.
    Visibility(BoolTrack),
}

impl NodeAnimationTarget {
    /// Advances the contained track and returns whether it is still running.
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::Translation(track) | Self::Scale(track) => track.update(dt),
            Self::Rotation(track) => track.update(dt),
            Self::Visibility(track) => track.update(dt),
        }
    }

    /// Returns whether the contained track has completed.
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Translation(track) | Self::Scale(track) => track.is_complete(),
            Self::Rotation(track) => track.is_complete(),
            Self::Visibility(track) => track.is_complete(),
        }
    }
}

/// Applies an Animato-backed track to one scene node.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeAnimator {
    /// Target scene node.
    pub node_id: NodeId,
    /// Field being animated.
    pub target: NodeAnimationTarget,
}

impl NodeAnimator {
    /// Creates a node animator.
    #[inline]
    pub const fn new(node_id: NodeId, target: NodeAnimationTarget) -> Self {
        Self { node_id, target }
    }

    /// Advances the animator, applies the current value, and returns completion.
    pub fn update(&mut self, dt: f32, scene: &mut SceneGraph) -> Result<bool, ValidationError> {
        self.target.update(dt);
        let node = scene
            .get_mut(self.node_id)
            .ok_or(ValidationError::InvalidId)?;
        match &self.target {
            NodeAnimationTarget::Translation(track) => {
                node.transform.translation = track.value();
            }
            NodeAnimationTarget::Rotation(track) => {
                node.transform.rotation = track.value();
            }
            NodeAnimationTarget::Scale(track) => {
                node.transform.scale = track.value();
            }
            NodeAnimationTarget::Visibility(track) => {
                node.visible = track.value();
            }
        }
        Ok(self.target.is_complete())
    }
}
