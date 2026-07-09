//! GPU skinning + morph-upload hooks (additive to the renderer registries).
//!
//! GPU resources stay renderer-owned. These hooks hold per-mesh joint matrix
//! and morph-weight buffers that the skinning WGSL path consumes. The CPU
//! fallback (`scenix_mesh::skin::cpu_skin`) is always available for
//! headless/no-GPU tests.
//!
//! v1.4.0 ships the registry + upload hooks and the `SKINNING_WGSL` snippet.
//! Full shader-pipeline wiring reuses the existing `PipelineCache` key
//! extension behind a `SKINNING` shader define.

use std::collections::BTreeMap;

use scenix_core::MeshId;
use scenix_math::Mat4;

/// Per-mesh GPU skinning state owned by the renderer.
#[derive(Default)]
pub struct GpuSkinningRegistry {
    joints: BTreeMap<MeshId, Vec<Mat4>>,
    morph_weights: BTreeMap<MeshId, Vec<f32>>,
}

impl GpuSkinningRegistry {
    /// Creates an empty registry.
    pub const fn new() -> Self {
        Self {
            joints: BTreeMap::new(),
            morph_weights: BTreeMap::new(),
        }
    }

    /// Registers (or replaces) the bone matrix buffer for `mesh_id`.
    pub fn register_skin(&mut self, mesh_id: MeshId, bones: Vec<Mat4>) {
        self.joints.insert(mesh_id, bones);
    }

    /// Updates the bone matrix buffer for `mesh_id`. Returns `false` if the
    /// mesh has no registered skin.
    pub fn update_bone_matrices(&mut self, mesh_id: MeshId, bones: &[Mat4]) -> bool {
        if let Some(slot) = self.joints.get_mut(&mesh_id) {
            slot.clear();
            slot.extend_from_slice(bones);
            true
        } else {
            false
        }
    }

    /// Unregisters the skin for `mesh_id`.
    pub fn unregister_skin(&mut self, mesh_id: MeshId) -> bool {
        self.joints.remove(&mesh_id).is_some()
    }

    /// Registers (or replaces) the morph-weight buffer for `mesh_id`.
    pub fn register_morph_targets(&mut self, mesh_id: MeshId, weights: Vec<f32>) {
        self.morph_weights.insert(mesh_id, weights);
    }

    /// Updates the morph-weight buffer for `mesh_id`.
    pub fn update_morph_weights(&mut self, mesh_id: MeshId, weights: &[f32]) -> bool {
        if let Some(slot) = self.morph_weights.get_mut(&mesh_id) {
            slot.clear();
            slot.extend_from_slice(weights);
            true
        } else {
            false
        }
    }

    /// Unregisters morph weights for `mesh_id`.
    pub fn unregister_morph(&mut self, mesh_id: MeshId) -> bool {
        self.morph_weights.remove(&mesh_id).is_some()
    }

    /// Returns the bone matrix slice for `mesh_id`, if registered.
    #[inline]
    pub fn bones(&self, mesh_id: MeshId) -> Option<&[Mat4]> {
        self.joints.get(&mesh_id).map(|v| v.as_slice())
    }

    /// Returns the morph-weight slice for `mesh_id`, if registered.
    #[inline]
    pub fn morph_weights(&self, mesh_id: MeshId) -> Option<&[f32]> {
        self.morph_weights.get(&mesh_id).map(|v| v.as_slice())
    }

    /// Returns whether `mesh_id` has a registered skin.
    #[inline]
    pub fn has_skin(&self, mesh_id: MeshId) -> bool {
        self.joints.contains_key(&mesh_id)
    }

    /// Returns whether `mesh_id` has registered morph weights.
    #[inline]
    pub fn has_morph(&self, mesh_id: MeshId) -> bool {
        self.morph_weights.contains_key(&mesh_id)
    }

    /// Returns the number of registered skins.
    #[inline]
    pub fn skin_count(&self) -> usize {
        self.joints.len()
    }

    /// Returns the number of registered morph-weight stacks.
    #[inline]
    pub fn morph_count(&self) -> usize {
        self.morph_weights.len()
    }
}

/// WGSL snippet appended to skinned vertex shaders. The renderer embeds this
/// behind a `SKINNING` shader define when a mesh has a registered skin.
pub const SKINNING_WGSL: &str = r#"// Scenix GPU skinning snippet (v1.4.0).
// Bind a storage buffer of joint mat4x4<f32> at group 1, binding 0.
struct Joint { matrix: mat4x4<f32> };
@group(1) @binding(0) var<storage, read> joints: array<Joint>;

// skin_vertex transforms a position by four weighted joint matrices.
fn skin_position(in_pos: vec3<f32>, joint_indices: vec4<u32>, weights: vec4<f32>) -> vec4<f32> {
    let m = joints[joint_indices.x].matrix * weights.x
          + joints[joint_indices.y].matrix * weights.y
          + joints[joint_indices.z].matrix * weights.z
          + joints[joint_indices.w].matrix * weights.w;
    return m * vec4<f32>(in_pos, 1.0);
}

// skin_normal transforms a normal as a direction (w = 0).
fn skin_normal(in_normal: vec3<f32>, joint_indices: vec4<u32>, weights: vec4<f32>) -> vec3<f32> {
    let m = joints[joint_indices.x].matrix * weights.x
          + joints[joint_indices.y].matrix * weights.y
          + joints[joint_indices.z].matrix * weights.z
          + joints[joint_indices.w].matrix * weights.w;
    let n = m * vec4<f32>(in_normal, 0.0);
    return normalize(n.xyz);
}
"#;
