use scenix_core::{Bounded, MaterialId, Renderable};
use scenix_math::{Aabb, Vec3};

use crate::Geometry;

/// A renderable mesh made from geometry and a material identifier.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mesh {
    /// CPU-side geometry data.
    pub geometry: Geometry,
    /// Material resource identifier.
    pub material_id: MaterialId,
    /// Stable render order. Lower values render earlier.
    pub render_order: u32,
}

impl Mesh {
    /// Creates a mesh from geometry and material ID.
    #[inline]
    pub const fn new(geometry: Geometry, material_id: MaterialId) -> Self {
        Self {
            geometry,
            material_id,
            render_order: 0,
        }
    }

    /// Returns this mesh with a render order.
    #[inline]
    pub const fn render_order(mut self, render_order: u32) -> Self {
        self.render_order = render_order;
        self
    }
}

impl Renderable for Mesh {
    #[inline]
    fn render_order(&self) -> u32 {
        self.render_order
    }
}

impl Bounded for Mesh {
    #[inline]
    fn aabb(&self) -> Aabb {
        self.geometry.aabb()
    }

    #[inline]
    fn bounding_sphere(&self) -> (Vec3, f32) {
        self.geometry.bounding_sphere()
    }
}
