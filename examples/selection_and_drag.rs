use std::collections::BTreeMap;

use scenix::{
    DragController, DragPlane, MaterialId, MeshId, PerspectiveCamera, Ray3, Raycaster, SceneGraph,
    SceneNode, SelectionFrustum, SelectionMode, SelectionRect, SnapSettings, Transform, Vec2, Vec3,
    box_geometry,
};

fn main() {
    let mesh_id = MeshId::new(1);
    let mut geometries = BTreeMap::new();
    geometries.insert(mesh_id, box_geometry(1.0, 1.0, 1.0, 1, 1, 1));

    let mut scene = SceneGraph::new();
    let node = scene.add(SceneNode::mesh("Movable cube", mesh_id, MaterialId::new(1)));
    scene.update_world_transforms();

    let camera = PerspectiveCamera::default().position(Vec3::new(0.0, 0.0, 5.0));
    let mut raycaster = Raycaster::new();
    raycaster.build_bvh(&scene, &geometries).unwrap();
    let frustum = SelectionFrustum::from_perspective(
        &camera,
        SelectionRect::from_ndc(Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5)),
    );
    let selected = raycaster.select_in_frustum(frustum, &scene, &geometries);
    scene.select_many(selected).unwrap();
    scene.select(node, SelectionMode::Add).unwrap();

    let plane = DragPlane::camera_facing(Vec3::ZERO, camera.position);
    let mut drag = DragController::default();
    drag.begin(
        &mut scene,
        node,
        Ray3::new(camera.position, Vec3::NEG_Z),
        plane,
        SnapSettings {
            translation: Vec3::new(0.5, 0.5, 0.5),
            ..SnapSettings::default()
        },
    )
    .unwrap();
    drag.update(
        &mut scene,
        Ray3::new(camera.position + Vec3::X, Vec3::NEG_Z),
    )
    .unwrap();
    drag.end(&mut scene).unwrap();
    assert_ne!(scene.get(node).unwrap().transform, Transform::IDENTITY);
}
