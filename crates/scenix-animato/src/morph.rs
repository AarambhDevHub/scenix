//! Morph-target weight animation.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use scenix_core::{MeshId, ValidationError};

use crate::ScalarTrack;

/// Mutable morph-weight lookup used by morph animators and the mixer.
pub trait MorphWeightStoreMut {
    /// Returns the mutable weight slice for `mesh_id`, when present.
    fn morph_weights_mut(&mut self, mesh_id: MeshId) -> Option<&mut [f32]>;
}

impl MorphWeightStoreMut for BTreeMap<MeshId, Vec<f32>> {
    #[inline]
    fn morph_weights_mut(&mut self, mesh_id: MeshId) -> Option<&mut [f32]> {
        self.get_mut(&mesh_id).map(|v| v.as_mut_slice())
    }
}

/// Drives one morph-target weight on one mesh with a scalar track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MorphWeightAnimator {
    /// Target mesh.
    pub mesh_id: MeshId,
    /// Target morph index inside the mesh's weight stack.
    pub target_index: usize,
    /// Scalar weight track.
    pub track: ScalarTrack,
}

impl MorphWeightAnimator {
    /// Creates a morph-weight animator.
    #[inline]
    pub const fn new(mesh_id: MeshId, target_index: usize, track: ScalarTrack) -> Self {
        Self {
            mesh_id,
            target_index,
            track,
        }
    }

    /// Advances, applies, and returns completion.
    pub fn update(
        &mut self,
        dt: f32,
        morphs: &mut impl MorphWeightStoreMut,
    ) -> Result<bool, ValidationError> {
        self.track.update(dt);
        if let Some(weights) = morphs.morph_weights_mut(self.mesh_id)
            && let Some(w) = weights.get_mut(self.target_index)
        {
            *w = self.track.value().clamp(0.0, 1.0);
        }
        Ok(self.track.is_complete())
    }
}
