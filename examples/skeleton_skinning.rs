//! CPU skinning + GPU hook registration smoke test.
//!
//! Demonstrates building a skinning data model, computing final joint
//! matrices from a `SkeletonPose`, CPU-skinning a small geometry, and
//! registering the result with the renderer GPU skinning registry.
//!
//! Run with:
//!   cargo run -p scenix --example skeleton_skinning --features mesh,animato

use scenix::{
    Geometry, GpuSkinningRegistry, Mat4, MeshId, MorphTarget, SkeletonPose, SkinningAttributes,
    Transform, Vec3, apply_morph, cpu_skin, final_joint_matrices,
};

fn main() {
    // Build a tiny 2-vertex geometry and a 1-joint skin.
    let mut geometry = Geometry::new();
    geometry.positions = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
    geometry.normals = vec![Vec3::Z, Vec3::Z];

    let skin = SkinningAttributes {
        joints: vec![[0, 0, 0, 0], [0, 0, 0, 0]],
        weights: vec![[1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]],
    };

    // A pose that translates the single joint up by 2 units.
    let pose = SkeletonPose::new(vec![Transform {
        translation: Vec3::new(0.0, 2.0, 0.0),
        ..Default::default()
    }]);
    let bone_world: Vec<Mat4> = pose
        .bones
        .iter()
        .map(|t| Mat4::from_translation(t.translation))
        .collect();

    let inverse_binds = vec![Mat4::IDENTITY];
    let final_mats = final_joint_matrices(&bone_world, &inverse_binds);
    let skinned = cpu_skin(&geometry, &skin, &final_mats);

    println!(
        "skeleton_skinning: original[1].y={:.2} skinned[1].y={:.2}",
        geometry.positions[1].y, skinned.positions[1].y
    );
    assert!((skinned.positions[1].y - 3.0).abs() < 1e-3);

    // Register with the GPU skinning registry (renderer-owned upload hooks).
    let mut registry = GpuSkinningRegistry::new();
    let mesh_id = MeshId::new(1);
    registry.register_skin(mesh_id, final_mats.clone());
    registry.register_morph_targets(mesh_id, vec![0.0]);
    assert!(registry.has_skin(mesh_id));
    assert!(registry.has_morph(mesh_id));

    // Apply a morph target to a cloned geometry.
    let mut target = MorphTarget::new("smile".to_string());
    target.positions_delta = vec![Vec3::new(0.5, 0.0, 0.0), Vec3::new(0.0, 0.5, 0.0)];
    let morphed = apply_morph(&geometry, &[target], &[1.0]);
    println!(
        "skeleton_skinning: morphed[0].x={:.2} (expected 0.5)",
        morphed.positions[0].x
    );

    println!("skeleton_skinning: done (CPU skin + GPU registry + morph apply)");
}
