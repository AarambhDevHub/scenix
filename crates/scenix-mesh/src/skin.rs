//! Skinning data model + CPU skinning fallback.
//!
//! CPU skinning keeps headless/no-GPU tests able to validate poses. The
//! renderer's GPU skinning path lives in `scenix-renderer::skinning`.

use alloc::vec::Vec;

use scenix_math::{Mat4, Vec3};

use crate::Geometry;

/// Per-vertex skinning attributes (glTF `JOINTS_0` + `WEIGHTS_0`).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkinningAttributes {
    /// Joint indices, four per vertex.
    pub joints: Vec<[u16; 4]>,
    /// Joint weights, four per vertex (callers should normalize to sum 1).
    pub weights: Vec<[f32; 4]>,
}

impl SkinningAttributes {
    /// Returns whether the attribute arrays match `vertex_count`.
    #[inline]
    pub fn matches(&self, vertex_count: usize) -> bool {
        self.joints.len() == vertex_count && self.weights.len() == vertex_count
    }
}

/// Full skinning data: per-vertex attributes + inverse-bind matrices.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkinningData {
    /// Per-vertex joint/weight attributes.
    pub attributes: SkinningAttributes,
    /// Inverse-bind matrices, one per joint.
    pub inverse_bind_matrices: Vec<Mat4>,
}

/// Live morph-weight storage for a mesh (one weight per morph target).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MorphWeights {
    /// Weights in target-stack order.
    pub weights: Vec<f32>,
}

impl MorphWeights {
    /// Creates a zero-initialized weight stack of `len` targets.
    pub fn zero(len: usize) -> Self {
        Self {
            weights: alloc::vec![0.0; len],
        }
    }

    /// Sets the weight at `index`, clamped to `[0, 1]`.
    #[inline]
    pub fn set(&mut self, index: usize, weight: f32) {
        if let Some(w) = self.weights.get_mut(index) {
            *w = weight.clamp(0.0, 1.0);
        }
    }

    /// Returns the weight at `index`, or `0.0` if out of range.
    #[inline]
    pub fn get(&self, index: usize) -> f32 {
        self.weights.get(index).copied().unwrap_or(0.0)
    }

    /// Returns the number of weights.
    #[inline]
    pub fn len(&self) -> usize {
        self.weights.len()
    }

    /// Returns whether there are no weights.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }
}

/// Computes final per-joint matrices from bone world transforms + inverse binds.
///
/// `bone_world` are the concatenated world transforms of each joint
/// (caller-computed from a `SkeletonPose` + hierarchy). Missing inverse-binds
/// default to identity.
pub fn final_joint_matrices(bone_world: &[Mat4], inverse_bind: &[Mat4]) -> Vec<Mat4> {
    bone_world
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let ibm = inverse_bind.get(i).copied().unwrap_or(Mat4::IDENTITY);
            m.mul_mat4(ibm)
        })
        .collect()
}

/// CPU-skins `geometry` by the given final joint matrices.
///
/// Returns a new deformed `Geometry`. Normals are deformed as directions
/// (ignoring homogeneous divide). If attribute/matrix counts mismatch the
/// input geometry is returned unchanged.
pub fn cpu_skin(geometry: &Geometry, skin: &SkinningAttributes, final_mats: &[Mat4]) -> Geometry {
    let positions = &geometry.positions;
    let normals = &geometry.normals;
    if !skin.matches(positions.len()) || final_mats.is_empty() {
        return geometry.clone();
    }
    let mut out = geometry.clone();
    for i in 0..positions.len() {
        let [j0, j1, j2, j3] = skin.joints[i];
        let [w0, w1, w2, w3] = skin.weights[i];
        let mut p = Vec3::ZERO;
        let mut n = Vec3::ZERO;
        for (j, w) in [(j0, w0), (j1, w1), (j2, w2), (j3, w3)] {
            if w <= 0.0 {
                continue;
            }
            let m = final_mats
                .get(j as usize)
                .copied()
                .unwrap_or(Mat4::IDENTITY);
            // Point transform (w=1) and direction transform (w=0) via mul_vec4.
            let p4 = m.mul_vec4(scenix_math::Vec4::new(
                positions[i].x,
                positions[i].y,
                positions[i].z,
                1.0,
            ));
            p += Vec3::new(p4.x, p4.y, p4.z) * w;
            let n4 = m.mul_vec4(scenix_math::Vec4::new(
                normals[i].x,
                normals[i].y,
                normals[i].z,
                0.0,
            ));
            n += Vec3::new(n4.x, n4.y, n4.z) * w;
        }
        out.positions[i] = p;
        out.normals[i] = n.normalize();
    }
    out
}

/// Applies morph-target deltas to `geometry` by `weights` in-place on a clone.
///
/// Only position deltas are applied for v1.4; normal deltas can be added
/// later by extending this function.
pub fn apply_morph(
    geometry: &Geometry,
    targets: &[crate::MorphTarget],
    weights: &[f32],
) -> Geometry {
    if targets.is_empty() || weights.is_empty() {
        return geometry.clone();
    }
    let mut out = geometry.clone();
    for (ti, target) in targets.iter().enumerate() {
        let w = weights.get(ti).copied().unwrap_or(0.0);
        if w == 0.0 {
            continue;
        }
        for (i, delta) in target.positions_delta.iter().enumerate() {
            if let Some(p) = out.positions.get_mut(i) {
                *p += *delta * w;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_math::Vec4;

    #[test]
    fn final_joint_matrices_combine_world_and_inverse_bind() {
        let world = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let ibm = Mat4::from_translation(Vec3::new(-1.0, 0.0, 0.0));
        let mats = final_joint_matrices(&[world], &[ibm]);
        // world * ibm == identity translation.
        let p = mats[0].mul_vec4(Vec4::new(5.0, 5.0, 5.0, 1.0));
        assert!((p.x - 5.0).abs() < 1e-4);
    }

    #[test]
    fn morph_weights_set_clamps() {
        let mut w = MorphWeights::zero(2);
        w.set(0, 1.5);
        assert_eq!(w.get(0), 1.0);
        w.set(1, -0.5);
        assert_eq!(w.get(1), 0.0);
    }
}
