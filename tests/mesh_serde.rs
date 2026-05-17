#![cfg(feature = "serde")]

use scenix_core::{MaterialId, MeshId};
use scenix_math::Mat4;
use scenix_mesh::{InstancedMesh, Mesh, MorphTarget, box_geometry};

#[test]
fn mesh_types_round_trip_with_serde() {
    let geometry = box_geometry(1.0, 1.0, 1.0, 1, 1, 1);
    let mesh = Mesh::new(geometry.clone(), MaterialId::new(4)).render_order(9);
    let morph = MorphTarget::new("smile")
        .positions_delta(vec![scenix_math::Vec3::ZERO; geometry.positions.len()]);
    let mut instances = InstancedMesh::new(MeshId::new(2), MaterialId::new(4));
    instances.push_transform(Mat4::IDENTITY);

    let mesh_json = serde_json::to_string(&mesh).unwrap();
    let morph_json = serde_json::to_string(&morph).unwrap();
    let instances_json = serde_json::to_string(&instances).unwrap();

    assert_eq!(serde_json::from_str::<Mesh>(&mesh_json).unwrap(), mesh);
    assert_eq!(
        serde_json::from_str::<MorphTarget>(&morph_json).unwrap(),
        morph
    );
    assert_eq!(
        serde_json::from_str::<InstancedMesh>(&instances_json).unwrap(),
        instances
    );
}
