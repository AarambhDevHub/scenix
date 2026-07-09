//! Clip-based animation: loop a translation clip, then crossfade to a second clip.
//!
//! Run with:
//!   cargo run -p scenix --example animation_runtime --features animato,scene

use std::collections::BTreeMap;

use scenix::{
    AnimationClip, AnimationMixer, CameraId, CameraStores, ClipChannel, ClipTrack,
    DirectionalLight, KeyframeInterpolation, KeyframeVec3, LightId, LightStores, LoopMode,
    MaterialId, MeshId, NodeProperty, OrthographicCamera, PbrMaterial, PerspectiveCamera,
    PointLight, PropertyBinding, SceneGraph, SceneNode, SpotLight, Vec3,
};

fn main() {
    let mut scene = SceneGraph::new();
    let node = scene.add(SceneNode::new("cube"));

    // A looping orbit clip: 5 keyframes tracing a square-ish path.
    let clip = AnimationClip::empty("orbit").with_channel(ClipChannel {
        binding: PropertyBinding::Node {
            node_id: node,
            property: NodeProperty::Translation,
        },
        track: ClipTrack::Vec3(KeyframeVec3::new(
            vec![0.0, 1.0, 2.0, 3.0, 4.0],
            vec![
                Vec3::ZERO,
                Vec3::X,
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::Y,
                Vec3::ZERO,
            ],
            KeyframeInterpolation::Linear,
        )),
    });

    let mut mixer = AnimationMixer::new();
    let clip_index = mixer.add_clip(clip);
    let action = mixer.add_action(clip_index);
    mixer
        .action_mut(action)
        .unwrap()
        .set_loop_mode(LoopMode::REPEAT);
    mixer.action_mut(action).unwrap().play(0.0);

    // Empty stores the mixer requires (v1.4 signature).
    let mut perspective: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut orthographic: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut materials: BTreeMap<MaterialId, PbrMaterial> = BTreeMap::new();
    let mut point_lights: BTreeMap<LightId, PointLight> = BTreeMap::new();
    let mut spot_lights: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut directional_lights: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();

    let dt = 1.0 / 30.0;
    for frame in 0..120 {
        let mut camera_stores = CameraStores {
            perspective: &mut perspective,
            orthographic: &mut orthographic,
        };
        let mut light_stores = LightStores {
            point: &mut point_lights,
            spot: &mut spot_lights,
            directional: &mut directional_lights,
        };
        let result = mixer
            .tick(
                dt,
                &mut scene,
                &mut camera_stores,
                &mut materials,
                &mut light_stores,
                &mut [],
                &mut morphs,
            )
            .expect("mixer tick");

        if frame % 30 == 0 {
            let pos = scene.get(node).unwrap().transform.translation;
            println!(
                "frame={frame:>3} pos=({:.2}, {:.2}, {:.2}) events={}",
                pos.x,
                pos.y,
                pos.z,
                result.events.len()
            );
        }
    }

    println!("animation_runtime: done (120 frames, looping orbit clip)");
}
