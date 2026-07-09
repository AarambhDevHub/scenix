use std::collections::BTreeMap;

use scenix::{
    CameraAnimationTarget, CameraId, CameraStores, Color, ColorTrack, DirectionalLight, LightId,
    LightStores, MaterialAnimationTarget, MaterialAnimator, MaterialId, MeshId,
    NodeAnimationTarget, NodeAnimator, PbrMaterial, PerspectiveCamera, PointLight, ScalarTrack,
    SceneGraph, SceneNode, ScenixAnimationDriver, SpotLight, SpringConfig, Vec3, Vec3Track,
};

fn main() {
    let mut scene = SceneGraph::new();
    let node_id = scene.add(SceneNode::new("animated-node"));

    let camera_id = CameraId::new(1);
    let mut perspective = BTreeMap::from([(
        camera_id,
        PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 1.0, 4.0))
            .target(Vec3::ZERO),
    )]);
    let mut orthographic = BTreeMap::new();

    let material_id = MaterialId::new(1);
    let mut materials = BTreeMap::from([(material_id, PbrMaterial::new())]);

    let mut driver = ScenixAnimationDriver::new();
    driver.add_node(NodeAnimator::new(
        node_id,
        NodeAnimationTarget::Translation(Vec3Track::tween(
            Vec3::ZERO,
            Vec3::new(2.0, 0.0, 0.0),
            1.0,
        )),
    ));
    driver.add_camera(scenix::CameraAnimator::new(
        camera_id,
        CameraAnimationTarget::Target(Vec3Track::spring(
            Vec3::ZERO,
            Vec3::new(0.0, 0.5, 0.0),
            SpringConfig::gentle(),
        )),
    ));
    driver.add_material(MaterialAnimator::new(
        material_id,
        MaterialAnimationTarget::Albedo(ColorTrack::tween(Color::WHITE, Color::BLUE, 1.0)),
    ));
    driver.add_material(MaterialAnimator::new(
        material_id,
        MaterialAnimationTarget::Opacity(ScalarTrack::tween(1.0, 0.35, 1.0)),
    ));

    let mut skeletons = Vec::new();
    let mut point_lights: BTreeMap<LightId, PointLight> = BTreeMap::new();
    let mut spot_lights: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut directional_lights: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();
    for _frame in 0..60 {
        let mut cameras = CameraStores {
            perspective: &mut perspective,
            orthographic: &mut orthographic,
        };
        let mut light_stores = LightStores {
            point: &mut point_lights,
            spot: &mut spot_lights,
            directional: &mut directional_lights,
        };
        driver
            .tick(
                1.0 / 60.0,
                &mut scene,
                &mut cameras,
                &mut materials,
                &mut light_stores,
                &mut morphs,
                &mut skeletons,
            )
            .expect("animation tick");
    }

    println!(
        "node={:?} camera_target={:?} opacity={:.2}",
        scene.get(node_id).unwrap().transform.translation,
        perspective.get(&camera_id).unwrap().target,
        materials.get(&material_id).unwrap().albedo.a
    );
}
