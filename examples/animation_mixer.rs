//! Two-action blend + additive + marker events through the AnimationMixer.
//!
//! Run with:
//!   cargo run -p scenix --example animation_mixer --features animato,scene,material,light

use std::collections::BTreeMap;

use scenix::{
    AnimationClip, AnimationMarker, AnimationMixer, BlendMode, CameraId, CameraStores, ClipChannel,
    ClipTrack, Color, DirectionalLight, KeyframeColor, KeyframeInterpolation, KeyframeScalar,
    KeyframeVec3, LightId, LightStores, LoopMode, MaterialId, MaterialProperty, MeshId,
    NodeProperty, OrthographicCamera, PbrMaterial, PerspectiveCamera, PointLight, PropertyBinding,
    SceneGraph, SceneNode, SpotLight, Vec3,
};

fn main() {
    let mut scene = SceneGraph::new();
    let node = scene.add(SceneNode::new("actor"));
    let material_id = MaterialId::new(1);

    // Clip A: a base translation that loops.
    let clip_a = AnimationClip::empty("walk")
        .with_channel(ClipChannel {
            binding: PropertyBinding::Node {
                node_id: node,
                property: NodeProperty::Translation,
            },
            track: ClipTrack::Vec3(KeyframeVec3::new(
                vec![0.0, 0.5, 1.0],
                vec![Vec3::ZERO, Vec3::new(0.5, 0.0, 0.0), Vec3::X],
                KeyframeInterpolation::Linear,
            )),
        })
        .with_marker(AnimationMarker::new("footstep", 0.5));

    // Clip B: an additive color pulse on the material.
    let clip_b = AnimationClip::empty("glow").with_channel(ClipChannel {
        binding: PropertyBinding::Material {
            material_id,
            property: MaterialProperty::Albedo,
        },
        track: ClipTrack::Color(KeyframeColor::new(
            vec![0.0, 1.0],
            vec![Color::rgb(0.2, 0.2, 0.2), Color::rgb(1.0, 0.8, 0.2)],
            KeyframeInterpolation::Linear,
        )),
    });

    // Clip C: a scalar intensity pulse on a point light.
    let light_id = LightId::new(1);
    let clip_c = AnimationClip::empty("pulse").with_channel(ClipChannel {
        binding: PropertyBinding::Light {
            light_id,
            property: scenix::LightProperty::Intensity,
        },
        track: ClipTrack::Scalar(KeyframeScalar::new(
            vec![0.0, 0.5, 1.0],
            vec![0.0, 5.0, 0.0],
            KeyframeInterpolation::Linear,
        )),
    });

    let mut mixer = AnimationMixer::new();
    let a = mixer.add_clip(clip_a);
    let b = mixer.add_clip(clip_b);
    let c = mixer.add_clip(clip_c);

    let action_a = mixer.add_action(a);
    mixer
        .action_mut(action_a)
        .unwrap()
        .set_loop_mode(LoopMode::REPEAT);
    mixer.action_mut(action_a).unwrap().play(0.0);

    let action_b = mixer.add_action(b);
    mixer
        .action_mut(action_b)
        .unwrap()
        .set_blend_mode(BlendMode::Additive);
    mixer
        .action_mut(action_b)
        .unwrap()
        .set_loop_mode(LoopMode::REPEAT);
    mixer.action_mut(action_b).unwrap().play(0.0);

    let action_c = mixer.add_action(c);
    mixer
        .action_mut(action_c)
        .unwrap()
        .set_loop_mode(LoopMode::REPEAT);
    mixer.action_mut(action_c).unwrap().play(0.0);

    // Stores.
    let mut perspective: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut orthographic: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut materials: BTreeMap<MaterialId, PbrMaterial> =
        BTreeMap::from([(material_id, PbrMaterial::new())]);
    let mut point_lights: BTreeMap<LightId, PointLight> =
        BTreeMap::from([(light_id, PointLight::new(Color::WHITE, 0.0, 10.0))]);
    let mut spot_lights: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut directional_lights: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();

    let dt = 1.0 / 30.0;
    for frame in 0..90 {
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

        if frame % 15 == 0 {
            let pos = scene.get(node).unwrap().transform.translation;
            let mat = materials.get(&material_id).unwrap();
            let li = point_lights.get(&light_id).unwrap().intensity;
            println!(
                "frame={frame:>3} pos=({:.2},{:.2},{:.2}) albedo=({:.2},{:.2},{:.2}) light={:.2} events={}",
                pos.x,
                pos.y,
                pos.z,
                mat.albedo.r,
                mat.albedo.g,
                mat.albedo.b,
                li,
                result.events.len()
            );
        }
    }

    println!("animation_mixer: done (3 blended actions over 90 frames)");
}
