use alloc::vec::Vec;

use scenix_core::MeshId;

/// Mesh levels of detail selected by camera distance.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LodGroup {
    levels: Vec<(f32, MeshId)>,
}

impl LodGroup {
    /// Creates an LOD group sorted from nearest threshold to farthest threshold.
    pub fn new(mut levels: Vec<(f32, MeshId)>) -> Self {
        levels.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { levels }
    }

    /// Creates an empty LOD group.
    #[inline]
    pub const fn empty() -> Self {
        Self { levels: Vec::new() }
    }

    /// Returns the sorted levels.
    #[inline]
    pub fn levels(&self) -> &[(f32, MeshId)] {
        &self.levels
    }

    /// Adds a mesh level and keeps thresholds sorted from near to far.
    pub fn add_level(&mut self, max_distance: f32, mesh_id: MeshId) {
        self.levels.push((max_distance, mesh_id));
        self.levels.sort_by(|a, b| a.0.total_cmp(&b.0));
    }

    /// Selects the first mesh whose threshold contains `distance`.
    pub fn select(&self, distance: f32) -> Option<MeshId> {
        let farthest = self.levels.last().map(|(_, mesh_id)| *mesh_id);
        self.levels
            .iter()
            .find(|(max_distance, _)| distance <= *max_distance)
            .map(|(_, mesh_id)| *mesh_id)
            .or(farthest)
    }
}
