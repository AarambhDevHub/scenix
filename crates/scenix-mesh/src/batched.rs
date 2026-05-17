use alloc::vec::Vec;

use scenix_core::{Bounded, MaterialId, ValidationError};
use scenix_math::{Aabb, Vec3};

use crate::Geometry;

/// Draw range metadata for one geometry inside a batched mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BatchedGeometryRange {
    /// First vertex in the merged geometry.
    pub vertex_start: u32,
    /// Number of vertices in this range.
    pub vertex_count: u32,
    /// First index in the merged index buffer.
    pub index_start: u32,
    /// Number of indices in this range.
    pub index_count: u32,
    /// Material used by this range.
    pub material_id: MaterialId,
}

/// Multiple geometries merged into one CPU-side geometry with draw ranges.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BatchedMesh {
    /// Merged CPU-side geometry.
    pub geometry: Geometry,
    /// Per-source geometry ranges.
    pub ranges: Vec<BatchedGeometryRange>,
}

impl BatchedMesh {
    /// Creates an empty batched mesh.
    #[inline]
    pub const fn new() -> Self {
        Self {
            geometry: Geometry::new(),
            ranges: Vec::new(),
        }
    }

    /// Adds geometry to the batch and returns the range index.
    pub fn add_geometry(
        &mut self,
        geometry: &Geometry,
        material_id: MaterialId,
    ) -> Result<usize, ValidationError> {
        geometry.validate()?;
        let vertex_start = self.geometry.positions.len() as u32;
        let index_start = self.geometry.indices.len() as u32;
        let index_count = if geometry.indices.is_empty() {
            geometry.positions.len()
        } else {
            geometry.indices.len()
        } as u32;
        let range = BatchedGeometryRange {
            vertex_start,
            vertex_count: geometry.positions.len() as u32,
            index_start,
            index_count,
            material_id,
        };
        self.geometry.merge(geometry);
        self.ranges.push(range);
        Ok(self.ranges.len() - 1)
    }

    /// Returns all geometry ranges.
    #[inline]
    pub fn ranges(&self) -> &[BatchedGeometryRange] {
        &self.ranges
    }
}

impl Bounded for BatchedMesh {
    #[inline]
    fn aabb(&self) -> Aabb {
        self.geometry.aabb()
    }

    #[inline]
    fn bounding_sphere(&self) -> (Vec3, f32) {
        self.geometry.bounding_sphere()
    }
}
