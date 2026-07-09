//! Skeleton retargeting: map source-skeleton bones onto a target skeleton.

use alloc::string::String;
use alloc::vec::Vec;

use crate::skeleton::SkeletonPose;

/// One source→target bone mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RetargetEntry {
    /// Source-skeleton bone index.
    pub source: usize,
    /// Target-skeleton bone index.
    pub target: usize,
}

/// A retarget map applied to copy + adjust a pose from one skeleton to another.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RetargetMap {
    /// Mapping entries in source order.
    pub entries: Vec<RetargetEntry>,
}

impl RetargetMap {
    /// Builds a map by matching bone names between `source_names` and
    /// `target_names`.
    pub fn from_names(source_names: &[String], target_names: &[String]) -> Self {
        let mut entries = Vec::new();
        for (s_idx, s_name) in source_names.iter().enumerate() {
            if let Some(t_idx) = target_names.iter().position(|t| t == s_name) {
                entries.push(RetargetEntry {
                    source: s_idx,
                    target: t_idx,
                });
            }
        }
        Self { entries }
    }

    /// Adds an explicit source→target entry.
    #[inline]
    pub fn with_entry(mut self, source: usize, target: usize) -> Self {
        self.entries.push(RetargetEntry { source, target });
        self
    }

    /// Copies `source` pose bones into `target` according to the map.
    ///
    /// Unmapped target bones keep their existing transforms.
    pub fn apply(&self, source: &SkeletonPose, target: &mut SkeletonPose) {
        for entry in &self.entries {
            if let (Some(src), Some(dst)) = (
                source.bones.get(entry.source),
                target.bones.get_mut(entry.target),
            ) {
                dst.clone_from(src);
            }
        }
    }
}
