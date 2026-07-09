//! Throughput benchmark for the AnimationMixer.
//!
//! Measures per-tick sampling + blending cost for 10, 100, and 1000-channel
//! clips so regressions in the mixer hot path are visible. Uses the same
//! lightweight custom timing harness as `animato_bridge_bench` (no criterion
//! dependency).

use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use scenix_animato::{
    AnimationClip, AnimationMixer, CameraStores, ClipChannel, ClipTrack, KeyframeInterpolation,
    KeyframeVec3, LightStores, LoopMode, NodeProperty, PropertyBinding, Vec3Track,
};
use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::{CameraId, LightId, MaterialId, MeshId};
use scenix_light::{DirectionalLight, PointLight, SpotLight};
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

fn run_mixer_tick(channels: usize) {
    let mut scene = SceneGraph::with_capacity(channels);
    let mut clip = AnimationClip::empty("bench");
    for _ in 0..channels {
        let node = scene.add(SceneNode::new("n"));
        clip = clip.with_channel(ClipChannel {
            binding: PropertyBinding::Node {
                node_id: node,
                property: NodeProperty::Translation,
            },
            track: ClipTrack::Vec3(KeyframeVec3::new(
                vec![0.0, 1.0],
                vec![Vec3::ZERO, Vec3::X],
                KeyframeInterpolation::Linear,
            )),
        });
    }

    let mut perspective: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut orthographic: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut materials: BTreeMap<MaterialId, PbrMaterial> = BTreeMap::new();
    let mut point_lights: BTreeMap<LightId, PointLight> = BTreeMap::new();
    let mut spot_lights: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut directional_lights: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();

    let mut mixer = AnimationMixer::new();
    let ci = mixer.add_clip(clip);
    let action = mixer.add_action(ci);
    mixer
        .action_mut(action)
        .unwrap()
        .set_loop_mode(LoopMode::REPEAT);
    mixer.action_mut(action).unwrap().play(0.0);

    let mut camera_stores = CameraStores {
        perspective: &mut perspective,
        orthographic: &mut orthographic,
    };
    let mut light_stores = LightStores {
        point: &mut point_lights,
        spot: &mut spot_lights,
        directional: &mut directional_lights,
    };
    black_box(
        mixer
            .tick(
                0.016,
                &mut scene,
                &mut camera_stores,
                &mut materials,
                &mut light_stores,
                &mut [],
                &mut morphs,
            )
            .unwrap(),
    );
}

fn main() {
    // Build once per iteration to measure the full tick path including clip
    // registration. Smaller channel counts run more iterations for stability.
    bench("mixer_10_channels", 500, || run_mixer_tick(10));
    bench("mixer_100_channels", 100, || run_mixer_tick(100));
    bench("mixer_1000_channels", 20, || run_mixer_tick(1000));
}
