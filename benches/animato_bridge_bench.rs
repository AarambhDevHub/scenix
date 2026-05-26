use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use scenix_animato::{
    CameraStores, MaterialAnimationTarget, MaterialAnimator, NodeAnimationTarget, NodeAnimator,
    ScalarTrack, ScenixAnimationDriver, SkeletonPose, Vec3Track,
};
use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::{CameraId, MaterialId, NodeId};
use scenix_material::PbrMaterial;
use scenix_math::Vec3;
use scenix_scene::{SceneGraph, SceneNode};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn driver(count: usize, scene: &mut SceneGraph) -> ScenixAnimationDriver {
    let mut driver = ScenixAnimationDriver::new();
    for index in 0..count {
        let node = scene.add(SceneNode::new(format!("node-{index}")));
        driver.add_node(NodeAnimator::new(
            node,
            NodeAnimationTarget::Translation(Vec3Track::tween(Vec3::ZERO, Vec3::X, 1.0)),
        ));
        driver.add_material(MaterialAnimator::new(
            MaterialId::new(index as u64 + 1),
            MaterialAnimationTarget::Roughness(ScalarTrack::tween(1.0, 0.2, 1.0)),
        ));
    }
    driver
}

fn run_tick(count: usize) {
    let mut scene = SceneGraph::with_capacity(count);
    let mut driver = driver(count, &mut scene);
    let mut perspective = BTreeMap::<CameraId, PerspectiveCamera>::new();
    let mut orthographic = BTreeMap::<CameraId, OrthographicCamera>::new();
    let mut cameras = CameraStores {
        perspective: &mut perspective,
        orthographic: &mut orthographic,
    };
    let mut materials = (0..count)
        .map(|index| (MaterialId::new(index as u64 + 1), PbrMaterial::new()))
        .collect::<BTreeMap<_, _>>();
    let mut skeletons = Vec::<SkeletonPose>::new();
    black_box(
        driver
            .tick(
                1.0 / 60.0,
                &mut scene,
                &mut cameras,
                &mut materials,
                &mut skeletons,
            )
            .unwrap(),
    );
}

fn main() {
    let _ = black_box(NodeId::new(1));
    bench("animato_bridge_tick_1k", 200, || run_tick(1_000));
    bench("animato_bridge_tick_10k", 20, || run_tick(10_000));
}
