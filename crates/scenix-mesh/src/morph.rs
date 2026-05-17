use alloc::string::{String, ToString};
use alloc::vec::Vec;

use scenix_core::ValidationError;
use scenix_math::Vec3;

/// Vertex delta data for blend-shape style mesh deformation.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MorphTarget {
    /// Human-readable target name.
    pub name: String,
    /// Per-vertex position deltas.
    pub positions_delta: Vec<Vec3>,
    /// Per-vertex normal deltas.
    pub normals_delta: Vec<Vec3>,
    /// Blend weight in the target stack.
    pub weight: f32,
}

impl MorphTarget {
    /// Creates an empty morph target.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            positions_delta: Vec::new(),
            normals_delta: Vec::new(),
            weight: 0.0,
        }
    }

    /// Returns this morph target with position deltas.
    #[inline]
    pub fn positions_delta(mut self, positions_delta: Vec<Vec3>) -> Self {
        self.positions_delta = positions_delta;
        self
    }

    /// Returns this morph target with normal deltas.
    #[inline]
    pub fn normals_delta(mut self, normals_delta: Vec<Vec3>) -> Self {
        self.normals_delta = normals_delta;
        self
    }

    /// Returns this morph target with a blend weight.
    #[inline]
    pub const fn weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Validates that non-empty delta arrays match the vertex count.
    pub fn validate(&self, vertex_count: usize) -> Result<(), ValidationError> {
        if !self.positions_delta.is_empty() && self.positions_delta.len() != vertex_count {
            return Err(ValidationError::InvalidState);
        }
        if !self.normals_delta.is_empty() && self.normals_delta.len() != vertex_count {
            return Err(ValidationError::InvalidState);
        }
        Ok(())
    }
}

impl Default for MorphTarget {
    #[inline]
    fn default() -> Self {
        Self::new("morph".to_string())
    }
}
