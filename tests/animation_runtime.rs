//! Integration tests for the v1.4.0 Animation Runtime.
//!
//! Exercises `AnimationClip`, `AnimationAction`, `AnimationMixer`, loop modes,
//! crossfade, markers/events, light targets, morph targets, retargeting, and
//! the facade `clip_from_loaded` bridge.
//!
//! Compiled under the `scenix` facade crate with the `animato`, `loader`,
//! `scene`, `camera`, `material`, `light`, `mesh`, and `helpers` features.

use std::collections::BTreeMap;

use scenix::{
    AnimationEvent, AnimationMarker, AnimationMixer, AnimationPathHelper, BlendMode, CameraStores,
    ClipChannel, ClipTrack, DirectionalLight, KeyframeInterpolation, KeyframeQuat, KeyframeScalar,
    KeyframeVec3, LightProperty, LightStores, LoopMode, NodeProperty, OrthographicCamera,
    PbrMaterial, PerspectiveCamera, PointLight, PoseHelper, PropertyBinding, RetargetMap,
    SceneGraph, SceneNode, SkeletonPose, SpotLight, Transform, Vec3,
};
use scenix::{CameraId, Color, LightId, MaterialId, MeshId, NodeId, Quat};

fn close(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-3, "{a} != {b}");
}

#[test]
fn mixer_plays_translation_clip_once_and_finishes() {
    let mut scene = SceneGraph::new();
    let node = scene.add(SceneNode::new("mover"));

    let mut persp: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut ortho: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut mats: BTreeMap<MaterialId, PbrMaterial> = BTreeMap::new();
    let mut lights_p: BTreeMap<LightId, PointLight> = BTreeMap::new();
    let mut lights_s: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut lights_d: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();

    let clip = scenix::AnimationClip::empty("move").with_channel(ClipChannel {
        binding: PropertyBinding::Node {
            node_id: node,
            property: NodeProperty::Translation,
        },
        track: ClipTrack::Vec3(KeyframeVec3::new(
            vec![0.0, 1.0],
            vec![Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0)],
            KeyframeInterpolation::Linear,
        )),
    });

    let mut mixer = AnimationMixer::new();
    let ci = mixer.add_clip(clip);
    let action = mixer.add_action(ci);
    mixer
        .action_mut(action)
        .unwrap()
        .set_loop_mode(LoopMode::Once);
    mixer.action_mut(action).unwrap().play(0.0);

    let mut stores = CameraStores {
        perspective: &mut persp,
        orthographic: &mut ortho,
    };
    let mut light_stores = LightStores {
        point: &mut lights_p,
        spot: &mut lights_s,
        directional: &mut lights_d,
    };

    let res = mixer
        .tick(
            0.5,
            &mut scene,
            &mut stores,
            &mut mats,
            &mut light_stores,
            &mut [],
            &mut morphs,
        )
        .unwrap();
    close(scene.get(node).unwrap().transform.translation.x, 2.5);
    assert_eq!(res.active_actions, 1);

    let res = mixer
        .tick(
            0.6,
            &mut scene,
            &mut stores,
            &mut mats,
            &mut light_stores,
            &mut [],
            &mut morphs,
        )
        .unwrap();
    close(scene.get(node).unwrap().transform.translation.x, 5.0);
    assert!(
        res.events
            .iter()
            .any(|e| matches!(e, AnimationEvent::Finished { .. }))
    );
}

#[test]
fn mixer_repeats_and_fires_loop_events() {
    let mut scene = SceneGraph::new();
    let node = scene.add(SceneNode::new("spin"));

    let mut persp: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut ortho: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut mats: BTreeMap<MaterialId, PbrMaterial> = BTreeMap::new();
    let mut lights_p: BTreeMap<LightId, PointLight> = BTreeMap::new();
    let mut lights_s: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut lights_d: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();

    let clip = scenix::AnimationClip::empty("spin").with_channel(ClipChannel {
        binding: PropertyBinding::Node {
            node_id: node,
            property: NodeProperty::Rotation,
        },
        track: ClipTrack::Quat(KeyframeQuat::new(
            vec![0.0, 1.0],
            vec![
                Quat::IDENTITY,
                Quat::from_axis_angle(Vec3::Y, core::f32::consts::TAU),
            ],
            KeyframeInterpolation::Linear,
        )),
    });

    let mut mixer = AnimationMixer::new();
    let ci = mixer.add_clip(clip);
    let a = mixer.add_action(ci);
    mixer.action_mut(a).unwrap().set_loop_mode(LoopMode::REPEAT);
    mixer.action_mut(a).unwrap().play(0.0);

    let mut stores = CameraStores {
        perspective: &mut persp,
        orthographic: &mut ortho,
    };
    let mut light_stores = LightStores {
        point: &mut lights_p,
        spot: &mut lights_s,
        directional: &mut lights_d,
    };

    let res = mixer
        .tick(
            2.5,
            &mut scene,
            &mut stores,
            &mut mats,
            &mut light_stores,
            &mut [],
            &mut morphs,
        )
        .unwrap();
    assert!(
        res.events
            .iter()
            .any(|e| matches!(e, AnimationEvent::Loop { .. }))
    );
    assert!(mixer.action(a).unwrap().iteration >= 2);
}

#[test]
fn crossfade_blends_two_actions() {
    let mut scene = SceneGraph::new();
    let node = scene.add(SceneNode::new("blend"));

    let mut persp: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut ortho: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut mats: BTreeMap<MaterialId, PbrMaterial> = BTreeMap::new();
    let mut lights_p: BTreeMap<LightId, PointLight> = BTreeMap::new();
    let mut lights_s: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut lights_d: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();

    let clip_a = scenix::AnimationClip::empty("a").with_channel(ClipChannel {
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
    let clip_b = scenix::AnimationClip::empty("b").with_channel(ClipChannel {
        binding: PropertyBinding::Node {
            node_id: node,
            property: NodeProperty::Translation,
        },
        track: ClipTrack::Vec3(KeyframeVec3::new(
            vec![0.0, 1.0],
            vec![Vec3::ZERO, Vec3::new(0.0, 0.0, 5.0)],
            KeyframeInterpolation::Linear,
        )),
    });

    let mut mixer = AnimationMixer::new();
    let ca = mixer.add_clip(clip_a);
    let cb = mixer.add_clip(clip_b);
    let aa = mixer.add_action(ca);
    let ab = mixer.add_action(cb);
    mixer.action_mut(aa).unwrap().play(0.0);
    mixer.action_mut(ab).unwrap().play(0.0);
    // Crossfade: A starts at full weight and fades out, B starts at zero and
    // fades in, both over 1 second.
    mixer.action_mut(aa).unwrap().set_weight(1.0);
    mixer.action_mut(aa).unwrap().fade_to(0.0, 1.0);
    mixer.action_mut(ab).unwrap().set_weight(0.0);
    mixer.action_mut(ab).unwrap().fade_to(1.0, 1.0);

    let mut stores = CameraStores {
        perspective: &mut persp,
        orthographic: &mut ortho,
    };
    let mut light_stores = LightStores {
        point: &mut lights_p,
        spot: &mut lights_s,
        directional: &mut lights_d,
    };

    mixer
        .tick(
            0.5,
            &mut scene,
            &mut stores,
            &mut mats,
            &mut light_stores,
            &mut [],
            &mut morphs,
        )
        .unwrap();
    let x = scene.get(node).unwrap().transform.translation.x;
    let z = scene.get(node).unwrap().transform.translation.z;
    // Both actions at 50% of their clips with equal weight 0.5 → weighted
    // average of (0.5,0,0) and (0,0,2.5) = (0.25, 0, 1.25), so x+z = 1.5.
    close(x, 0.25);
    close(z, 1.25);
    let _ = BlendMode::Normal;
}

#[test]
fn marker_fires_when_crossed() {
    let mut scene = SceneGraph::new();
    let node = scene.add(SceneNode::new("m"));

    let mut persp: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut ortho: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut mats: BTreeMap<MaterialId, PbrMaterial> = BTreeMap::new();
    let mut lights_p: BTreeMap<LightId, PointLight> = BTreeMap::new();
    let mut lights_s: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut lights_d: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();

    let clip = scenix::AnimationClip::empty("c")
        .with_channel(ClipChannel {
            binding: PropertyBinding::Node {
                node_id: node,
                property: NodeProperty::Translation,
            },
            track: ClipTrack::Vec3(KeyframeVec3::new(
                vec![0.0, 1.0],
                vec![Vec3::ZERO, Vec3::X],
                KeyframeInterpolation::Linear,
            )),
        })
        .with_marker(AnimationMarker::new("hit", 0.4));

    let mut mixer = AnimationMixer::new();
    let ci = mixer.add_clip(clip);
    let a = mixer.add_action(ci);
    mixer.action_mut(a).unwrap().play(0.0);

    let mut stores = CameraStores {
        perspective: &mut persp,
        orthographic: &mut ortho,
    };
    let mut light_stores = LightStores {
        point: &mut lights_p,
        spot: &mut lights_s,
        directional: &mut lights_d,
    };

    let res = mixer
        .tick(
            0.5,
            &mut scene,
            &mut stores,
            &mut mats,
            &mut light_stores,
            &mut [],
            &mut morphs,
        )
        .unwrap();
    assert!(
        res.events
            .iter()
            .any(|e| matches!(e, AnimationEvent::Marker { name, .. } if name == "hit"))
    );
}

#[test]
fn light_intensity_target_drives_point_light() {
    let mut scene = SceneGraph::new();
    let mut persp: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut ortho: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut mats: BTreeMap<MaterialId, PbrMaterial> = BTreeMap::new();
    let id = LightId::new(1);
    let mut lights_p = BTreeMap::from([(id, PointLight::new(Color::WHITE, 0.0, 10.0))]);
    let mut lights_s: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut lights_d: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::new();

    let clip = scenix::AnimationClip::empty("glow").with_channel(ClipChannel {
        binding: PropertyBinding::Light {
            light_id: id,
            property: LightProperty::Intensity,
        },
        track: ClipTrack::Scalar(KeyframeScalar::new(
            vec![0.0, 1.0],
            vec![0.0, 5.0],
            KeyframeInterpolation::Linear,
        )),
    });
    let mut mixer = AnimationMixer::new();
    let ci = mixer.add_clip(clip);
    let a = mixer.add_action(ci);
    mixer.action_mut(a).unwrap().play(0.0);

    let mut stores = CameraStores {
        perspective: &mut persp,
        orthographic: &mut ortho,
    };
    let mut light_stores = LightStores {
        point: &mut lights_p,
        spot: &mut lights_s,
        directional: &mut lights_d,
    };

    mixer
        .tick(
            0.5,
            &mut scene,
            &mut stores,
            &mut mats,
            &mut light_stores,
            &mut [],
            &mut morphs,
        )
        .unwrap();
    close(light_stores.point.get(&id).unwrap().intensity, 2.5);
}

#[test]
fn morph_weight_target_drives_mesh_weights() {
    let mut scene = SceneGraph::new();
    let mut persp: BTreeMap<CameraId, PerspectiveCamera> = BTreeMap::new();
    let mut ortho: BTreeMap<CameraId, OrthographicCamera> = BTreeMap::new();
    let mut mats: BTreeMap<MaterialId, PbrMaterial> = BTreeMap::new();
    let mut lights_p: BTreeMap<LightId, PointLight> = BTreeMap::new();
    let mut lights_s: BTreeMap<LightId, SpotLight> = BTreeMap::new();
    let mut lights_d: BTreeMap<LightId, DirectionalLight> = BTreeMap::new();
    let mesh_id = MeshId::new(7);
    let mut morphs: BTreeMap<MeshId, Vec<f32>> = BTreeMap::from([(mesh_id, vec![0.0, 0.0])]);

    let clip = scenix::AnimationClip::empty("morph").with_channel(ClipChannel {
        binding: PropertyBinding::MorphWeight {
            mesh_id,
            target_index: 1,
        },
        track: ClipTrack::Scalar(KeyframeScalar::new(
            vec![0.0, 1.0],
            vec![0.0, 1.0],
            KeyframeInterpolation::Linear,
        )),
    });
    let mut mixer = AnimationMixer::new();
    let ci = mixer.add_clip(clip);
    let a = mixer.add_action(ci);
    mixer.action_mut(a).unwrap().play(0.0);

    let mut stores = CameraStores {
        perspective: &mut persp,
        orthographic: &mut ortho,
    };
    let mut light_stores = LightStores {
        point: &mut lights_p,
        spot: &mut lights_s,
        directional: &mut lights_d,
    };

    mixer
        .tick(
            0.5,
            &mut scene,
            &mut stores,
            &mut mats,
            &mut light_stores,
            &mut [],
            &mut morphs,
        )
        .unwrap();
    close(morphs.get(&mesh_id).unwrap()[1], 0.5);
}

#[test]
fn retarget_map_copies_matched_bones() {
    let src = SkeletonPose::new(vec![
        Transform::IDENTITY,
        Transform {
            translation: Vec3::Y,
            ..Default::default()
        },
    ]);
    let mut dst = SkeletonPose::identity(2);
    let map = RetargetMap::from_names(
        &["root".to_string(), "head".to_string()],
        &["root".to_string(), "head".to_string()],
    );
    map.apply(&src, &mut dst);
    assert_eq!(dst.bones[1].translation, Vec3::Y);
}

#[test]
fn animation_path_helper_builds_polyline() {
    let helper = AnimationPathHelper::sample(4, 1.0, Color::GREEN, |t| Vec3::new(t, 0.0, 0.0));
    assert_eq!(helper.points.len(), 5);
    let geom = helper.to_geometry();
    assert_eq!(geom.positions.len(), 8); // 4 segments * 2 endpoints
}

#[test]
fn pose_helper_emits_three_axis_geometries() {
    let helper = PoseHelper::from_origins(&[Vec3::ZERO, Vec3::Y], 0.5);
    let [gx, gy, gz] = helper.to_geometries();
    assert_eq!(gx.positions.len(), 4); // 2 bones * 2 endpoints
    assert_eq!(gy.positions.len(), 4);
    assert_eq!(gz.positions.len(), 4);
}

#[test]
fn clip_from_loaded_builds_runtime_clip() {
    use scenix::{
        LoadedAnimationChannel, LoadedAnimationClip, LoadedAnimationInterpolation,
        LoadedAnimationProperty,
    };

    let loaded = LoadedAnimationClip {
        id: scenix::AnimationClipId::new(1),
        name: "imported".to_string(),
        duration: 1.0,
        channels: vec![LoadedAnimationChannel {
            node_index: 0,
            property: LoadedAnimationProperty::Translation,
            interpolation: LoadedAnimationInterpolation::Linear,
            times: vec![0.0, 1.0],
            output: vec![0.0, 0.0, 0.0, 5.0, 0.0, 0.0],
            output_components: 3,
        }],
    };
    let node_id = NodeId::new(42);
    let clip = scenix::clip_from_loaded(&loaded, &[node_id]);
    assert_eq!(clip.name, "imported");
    assert_eq!(clip.channels.len(), 1);
}
