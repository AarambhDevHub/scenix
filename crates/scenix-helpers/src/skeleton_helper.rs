use alloc::vec::Vec;

use scenix_core::{Color, ValidationError};
use scenix_math::Vec3;

use crate::LineGeometry;

/// Bone-line skeleton helper.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkeletonHelper {
    /// World-space joint positions.
    pub joints: Vec<Vec3>,
    /// Parent index per joint. Root joints use `None`.
    pub parents: Vec<Option<usize>>,
    /// Line color.
    pub color: Color,
}

impl SkeletonHelper {
    /// Creates a skeleton helper from joint positions and parent indices.
    #[inline]
    pub fn new(joints: Vec<Vec3>, parents: Vec<Option<usize>>, color: Color) -> Self {
        Self {
            joints,
            parents,
            color,
        }
    }

    /// Validates parent list lengths and ranges.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.joints.len() != self.parents.len() {
            return Err(ValidationError::InvalidState);
        }
        for (index, parent) in self.parents.iter().enumerate() {
            if let Some(parent) = parent
                && (*parent >= self.joints.len() || *parent == index)
            {
                return Err(ValidationError::OutOfRange);
            }
        }
        Ok(())
    }

    /// Generates one line segment from every child joint to its parent.
    pub fn to_geometry(&self) -> LineGeometry {
        let mut geometry = LineGeometry::new();
        if self.validate().is_err() {
            return geometry;
        }
        for (index, parent) in self.parents.iter().enumerate() {
            if let Some(parent) = parent {
                geometry.push_segment(self.joints[*parent], self.joints[index], self.color);
            }
        }
        geometry
    }
}
