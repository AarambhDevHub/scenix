use alloc::vec::Vec;

use scenix_core::ValidationError;
use scenix_scene::SceneGraph;

use crate::{
    CameraAnimator, CameraStoreMut, MaterialAnimator, NodeAnimator, PbrMaterialStoreMut,
    SkeletonPose, SkinnedMeshAnimator,
};

/// Per-tick animation driver counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DriverStats {
    /// Node animators updated this tick.
    pub node_animators: usize,
    /// Camera animators updated this tick.
    pub camera_animators: usize,
    /// Material animators updated this tick.
    pub material_animators: usize,
    /// Skeleton animators updated this tick.
    pub skeleton_animators: usize,
    /// Completed animators pruned after this tick.
    pub completed: usize,
}

/// Deterministic scene/camera/material/skeleton animation driver.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScenixAnimationDriver {
    node_animators: Vec<NodeAnimator>,
    camera_animators: Vec<CameraAnimator>,
    material_animators: Vec<MaterialAnimator>,
    skeleton_animators: Vec<SkinnedMeshAnimator>,
    paused: bool,
}

impl ScenixAnimationDriver {
    /// Creates an empty driver.
    #[inline]
    pub const fn new() -> Self {
        Self {
            node_animators: Vec::new(),
            camera_animators: Vec::new(),
            material_animators: Vec::new(),
            skeleton_animators: Vec::new(),
            paused: false,
        }
    }

    /// Adds a node animator and returns its index.
    pub fn add_node(&mut self, animator: NodeAnimator) -> usize {
        self.node_animators.push(animator);
        self.node_animators.len() - 1
    }

    /// Adds a camera animator and returns its index.
    pub fn add_camera(&mut self, animator: CameraAnimator) -> usize {
        self.camera_animators.push(animator);
        self.camera_animators.len() - 1
    }

    /// Adds a material animator and returns its index.
    pub fn add_material(&mut self, animator: MaterialAnimator) -> usize {
        self.material_animators.push(animator);
        self.material_animators.len() - 1
    }

    /// Adds a skeleton animator and returns its index.
    pub fn add_skeleton(&mut self, animator: SkinnedMeshAnimator) -> usize {
        self.skeleton_animators.push(animator);
        self.skeleton_animators.len() - 1
    }

    /// Removes a node animator by index.
    pub fn remove_node(&mut self, index: usize) -> Option<NodeAnimator> {
        remove_stable(&mut self.node_animators, index)
    }

    /// Removes a camera animator by index.
    pub fn remove_camera(&mut self, index: usize) -> Option<CameraAnimator> {
        remove_stable(&mut self.camera_animators, index)
    }

    /// Removes a material animator by index.
    pub fn remove_material(&mut self, index: usize) -> Option<MaterialAnimator> {
        remove_stable(&mut self.material_animators, index)
    }

    /// Removes a skeleton animator by index.
    pub fn remove_skeleton(&mut self, index: usize) -> Option<SkinnedMeshAnimator> {
        remove_stable(&mut self.skeleton_animators, index)
    }

    /// Removes every animator.
    pub fn clear(&mut self) {
        self.node_animators.clear();
        self.camera_animators.clear();
        self.material_animators.clear();
        self.skeleton_animators.clear();
    }

    /// Pauses the driver.
    #[inline]
    pub const fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the driver.
    #[inline]
    pub const fn resume(&mut self) {
        self.paused = false;
    }

    /// Returns whether the driver is paused.
    #[inline]
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns whether no animators are registered.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.node_animators.is_empty()
            && self.camera_animators.is_empty()
            && self.material_animators.is_empty()
            && self.skeleton_animators.is_empty()
    }

    /// Returns total registered animator count.
    #[inline]
    pub fn len(&self) -> usize {
        self.node_animators.len()
            + self.camera_animators.len()
            + self.material_animators.len()
            + self.skeleton_animators.len()
    }

    /// Returns registered node animator count.
    #[inline]
    pub fn node_len(&self) -> usize {
        self.node_animators.len()
    }

    /// Returns registered camera animator count.
    #[inline]
    pub fn camera_len(&self) -> usize {
        self.camera_animators.len()
    }

    /// Returns registered material animator count.
    #[inline]
    pub fn material_len(&self) -> usize {
        self.material_animators.len()
    }

    /// Returns registered skeleton animator count.
    #[inline]
    pub fn skeleton_len(&self) -> usize {
        self.skeleton_animators.len()
    }

    /// Advances every animator in deterministic insertion order.
    pub fn tick(
        &mut self,
        dt: f32,
        scene: &mut SceneGraph,
        cameras: &mut impl CameraStoreMut,
        materials: &mut impl PbrMaterialStoreMut,
        skeletons: &mut [SkeletonPose],
    ) -> Result<DriverStats, ValidationError> {
        let stats = DriverStats {
            node_animators: self.node_animators.len(),
            camera_animators: self.camera_animators.len(),
            material_animators: self.material_animators.len(),
            skeleton_animators: self.skeleton_animators.len(),
            completed: 0,
        };

        if self.paused {
            return Ok(stats);
        }

        let mut completed = 0;
        prune_completed(&mut self.node_animators, &mut completed, |animator| {
            animator.update(dt, scene)
        })?;
        prune_completed(&mut self.camera_animators, &mut completed, |animator| {
            animator.update(dt, cameras)
        })?;
        prune_completed(&mut self.material_animators, &mut completed, |animator| {
            animator.update(dt, materials)
        })?;
        prune_completed(&mut self.skeleton_animators, &mut completed, |animator| {
            animator.update(dt, skeletons)
        })?;

        Ok(DriverStats { completed, ..stats })
    }
}

fn remove_stable<T>(items: &mut Vec<T>, index: usize) -> Option<T> {
    if index < items.len() {
        Some(items.remove(index))
    } else {
        None
    }
}

fn prune_completed<T>(
    items: &mut Vec<T>,
    completed: &mut usize,
    mut update: impl FnMut(&mut T) -> Result<bool, ValidationError>,
) -> Result<(), ValidationError> {
    let mut error = None;
    items.retain_mut(|item| {
        if error.is_some() {
            return true;
        }
        match update(item) {
            Ok(true) => {
                *completed += 1;
                false
            }
            Ok(false) => true,
            Err(err) => {
                error = Some(err);
                true
            }
        }
    });
    if let Some(err) = error {
        Err(err)
    } else {
        Ok(())
    }
}
