use std::collections::BTreeMap;

use scenix::{
    Geometry, MaterialId, MeshId, PerspectiveCamera, Raycaster, SceneGraph, SceneNode, Transform,
    Vec2, Vec3, box_geometry,
};

fn main() -> Result<(), scenix::ValidationError> {
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    let mut meshes = BTreeMap::<MeshId, Geometry>::new();
    meshes.insert(mesh_id, box_geometry(1.0, 1.0, 1.0, 1, 1, 1));

    let mut scene = SceneGraph::new();
    scene.add(
        SceneNode::mesh("pickable-cube", mesh_id, material_id)
            .transform(Transform::from_translation(Vec3::ZERO)),
    );
    scene.update_world_transforms();

    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 0.0, 4.0))
        .target(Vec3::ZERO);
    let ray = Raycaster::from_camera_ndc(&camera, Vec2::ZERO);

    let mut raycaster = Raycaster::new();
    raycaster.build_bvh(&scene, &meshes)?;

    if let Some(hit) = raycaster.cast_ray(ray, &scene, &meshes) {
        println!(
            "picked node {:?} at distance {:.3}, point {:?}",
            hit.node_id, hit.distance, hit.point
        );
    }

    Ok(())
}
