use scenix_core::{MaterialId, MeshId, NodeId};
use scenix_math::{Vec2, Vec3};

/// A world-space ray intersection against a scene mesh node.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Intersection {
    /// Scene node hit by the ray.
    pub node_id: NodeId,
    /// Mesh resource attached to the hit node.
    pub mesh_id: MeshId,
    /// Material resource attached to the hit node.
    pub material_id: MaterialId,
    /// Parametric distance along the ray.
    pub distance: f32,
    /// World-space hit point.
    pub point: Vec3,
    /// World-space surface normal.
    pub normal: Vec3,
    /// Interpolated primary UV coordinates, or zero when unavailable.
    pub uv: Vec2,
}
