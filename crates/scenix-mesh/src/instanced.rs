use alloc::vec::Vec;

use scenix_core::{MaterialId, MeshId, Renderable, ValidationError};
use scenix_math::Mat4;

/// Draw metadata for many instances of the same mesh and material.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InstancedMesh {
    /// Mesh resource identifier.
    pub mesh_id: MeshId,
    /// Material resource identifier.
    pub material_id: MaterialId,
    /// Per-instance world transforms.
    pub transforms: Vec<Mat4>,
    /// Stable render order. Lower values render earlier.
    pub render_order: u32,
}

impl InstancedMesh {
    /// Creates an empty instanced mesh.
    #[inline]
    pub const fn new(mesh_id: MeshId, material_id: MaterialId) -> Self {
        Self {
            mesh_id,
            material_id,
            transforms: Vec::new(),
            render_order: 0,
        }
    }

    /// Creates an instanced mesh with reserved transform capacity.
    #[inline]
    pub fn with_capacity(mesh_id: MeshId, material_id: MaterialId, capacity: usize) -> Self {
        Self {
            mesh_id,
            material_id,
            transforms: Vec::with_capacity(capacity),
            render_order: 0,
        }
    }

    /// Returns the number of instances.
    #[inline]
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Returns whether there are no instances.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Appends an instance transform.
    #[inline]
    pub fn push_transform(&mut self, transform: Mat4) {
        self.transforms.push(transform);
    }

    /// Sets an instance transform by index.
    pub fn set_transform_at(
        &mut self,
        index: usize,
        transform: Mat4,
    ) -> Result<(), ValidationError> {
        let Some(slot) = self.transforms.get_mut(index) else {
            return Err(ValidationError::OutOfRange);
        };
        *slot = transform;
        Ok(())
    }

    /// Returns an instance transform by index.
    #[inline]
    pub fn transform_at(&self, index: usize) -> Option<Mat4> {
        self.transforms.get(index).copied()
    }

    /// Returns this instanced mesh with a render order.
    #[inline]
    pub const fn render_order(mut self, render_order: u32) -> Self {
        self.render_order = render_order;
        self
    }
}

impl Renderable for InstancedMesh {
    #[inline]
    fn render_order(&self) -> u32 {
        self.render_order
    }
}
