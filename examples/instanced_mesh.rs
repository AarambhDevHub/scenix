use scenix::{InstancedMesh, MaterialId, MeshId, Transform, Vec3};

fn main() {
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    let mut instances = InstancedMesh::with_capacity(mesh_id, material_id, 10_000);

    for z in 0..100 {
        for x in 0..100 {
            let position = Vec3::new(x as f32 * 1.25, 0.0, z as f32 * 1.25);
            instances.push_transform(Transform::from_translation(position).to_mat4());
        }
    }

    println!(
        "prepared {} instances for mesh {:?} / material {:?}",
        instances.len(),
        instances.mesh_id,
        instances.material_id
    );
}
