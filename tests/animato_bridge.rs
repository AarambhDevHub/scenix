use std::collections::BTreeMap;

use scenix_animato::{
    BoneAnimation, BoneAnimationTarget, CameraAnimationTarget, CameraAnimator, CameraStores,
    ColorTrack, MaterialAnimationTarget, MaterialAnimator, NodeAnimationTarget, NodeAnimator,
    QuatTrack, ScalarTrack, ScenixAnimationDriver, SkeletonPose, SkinnedMeshAnimator, Vec3Track,
};
use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::{CameraId, Color, MaterialId};
use scenix_material::{AlphaMode, PbrMaterial};
use scenix_math::{Quat, Transform, Vec3};
use scenix_scene::{SceneGraph, SceneNode};

fn close(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-4, "{a} != {b}");
}

#[test]
fn tween_drives_node_translation_scale_and_visibility() {
    let mut scene = SceneGraph::new();
    let node_id = scene.add(SceneNode::new("animated"));

    let mut translation = NodeAnimator::new(
        node_id,
        NodeAnimationTarget::Translation(Vec3Track::tween(
            Vec3::ZERO,
            Vec3::new(2.0, 0.0, 0.0),
            1.0,
        )),
    );
    assert!(!translation.update(0.5, &mut scene).unwrap());
    close(scene.get(node_id).unwrap().transform.translation.x, 1.0);
    assert!(translation.update(0.5, &mut scene).unwrap());
    close(scene.get(node_id).unwrap().transform.translation.x, 2.0);

    let mut scale = NodeAnimator::new(
        node_id,
        NodeAnimationTarget::Scale(Vec3Track::tween(Vec3::ONE, Vec3::new(2.0, 3.0, 4.0), 1.0)),
    );
    scale.update(1.0, &mut scene).unwrap();
    assert_eq!(
        scene.get(node_id).unwrap().transform.scale,
        Vec3::new(2.0, 3.0, 4.0)
    );

    let mut visibility = NodeAnimator::new(
        node_id,
        NodeAnimationTarget::Visibility(scenix_animato::BoolTrack::new(true, false, 1.0)),
    );
    visibility.update(1.0, &mut scene).unwrap();
    assert!(!scene.get(node_id).unwrap().visible);
}

#[test]
fn rotation_uses_slerp_and_stays_normalized() {
    let mut scene = SceneGraph::new();
    let node_id = scene.add(SceneNode::new("rotating"));
    let end = Quat::from_axis_angle(Vec3::Y, core::f32::consts::PI);
    let mut animator = NodeAnimator::new(
        node_id,
        NodeAnimationTarget::Rotation(QuatTrack::tween(Quat::IDENTITY, end, 1.0)),
    );

    animator.update(0.5, &mut scene).unwrap();
    let rotation = scene.get(node_id).unwrap().transform.rotation;
    close(rotation.length(), 1.0);
    close(rotation.mul_vec3(Vec3::X).z, -1.0);
}

#[test]
fn camera_animator_updates_perspective_and_orthographic_stores() {
    let camera_id = CameraId::new(1);
    let ortho_id = CameraId::new(2);
    let mut perspective = BTreeMap::from([(camera_id, PerspectiveCamera::default())]);
    let mut orthographic = BTreeMap::from([(ortho_id, OrthographicCamera::default())]);
    let mut stores = CameraStores {
        perspective: &mut perspective,
        orthographic: &mut orthographic,
    };

    CameraAnimator::new(
        camera_id,
        CameraAnimationTarget::Position(Vec3Track::tween(
            Vec3::ZERO,
            Vec3::new(1.0, 2.0, 3.0),
            1.0,
        )),
    )
    .update(1.0, &mut stores)
    .unwrap();
    CameraAnimator::new(
        camera_id,
        CameraAnimationTarget::FovY(ScalarTrack::tween(0.5, 1.0, 1.0)),
    )
    .update(1.0, &mut stores)
    .unwrap();
    CameraAnimator::new(
        ortho_id,
        CameraAnimationTarget::OrthographicBounds(scenix_animato::OrthographicBoundsTrack::tween(
            scenix_animato::OrthographicBounds::new(-1.0, 1.0, -1.0, 1.0),
            scenix_animato::OrthographicBounds::new(-2.0, 2.0, -3.0, 3.0),
            1.0,
        )),
    )
    .update(1.0, &mut stores)
    .unwrap();

    assert_eq!(
        stores.perspective.get(&camera_id).unwrap().position,
        Vec3::new(1.0, 2.0, 3.0)
    );
    close(stores.perspective.get(&camera_id).unwrap().fov_y, 1.0);
    assert_eq!(stores.orthographic.get(&ortho_id).unwrap().left, -2.0);
    assert_eq!(stores.orthographic.get(&ortho_id).unwrap().top, 3.0);
}

#[test]
fn material_animator_updates_pbr_fields() {
    let material_id = MaterialId::new(1);
    let mut materials = BTreeMap::from([(material_id, PbrMaterial::new())]);

    MaterialAnimator::new(
        material_id,
        MaterialAnimationTarget::Albedo(ColorTrack::tween(Color::WHITE, Color::RED, 1.0)),
    )
    .update(1.0, &mut materials)
    .unwrap();
    MaterialAnimator::new(
        material_id,
        MaterialAnimationTarget::Opacity(ScalarTrack::tween(1.0, 0.25, 1.0)),
    )
    .update(1.0, &mut materials)
    .unwrap();
    MaterialAnimator::new(
        material_id,
        MaterialAnimationTarget::Emissive(Vec3Track::tween(
            Vec3::ZERO,
            Vec3::new(0.1, 0.2, 0.3),
            1.0,
        )),
    )
    .update(1.0, &mut materials)
    .unwrap();
    MaterialAnimator::new(
        material_id,
        MaterialAnimationTarget::Roughness(ScalarTrack::tween(1.0, 0.2, 1.0)),
    )
    .update(1.0, &mut materials)
    .unwrap();
    MaterialAnimator::new(
        material_id,
        MaterialAnimationTarget::Metallic(ScalarTrack::tween(0.0, 0.8, 1.0)),
    )
    .update(1.0, &mut materials)
    .unwrap();

    let material = materials.get(&material_id).unwrap();
    assert_eq!(material.albedo.r, 1.0);
    close(material.albedo.a, 0.25);
    assert_eq!(material.alpha_mode, AlphaMode::Blend);
    assert_eq!(material.emissive, Vec3::new(0.1, 0.2, 0.3));
    close(material.roughness, 0.2);
    close(material.metallic, 0.8);
}

#[test]
fn skeleton_animation_updates_bone_transforms() {
    let mut skeletons = vec![SkeletonPose::identity(2)];
    let bone = BoneAnimation::new(
        1,
        BoneAnimationTarget::Translation(Vec3Track::tween(
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
            1.0,
        )),
    );
    let mut animator = SkinnedMeshAnimator::new(0, vec![bone]);
    assert!(animator.update(1.0, &mut skeletons).unwrap());
    assert_eq!(skeletons[0].bones[1].translation, Vec3::Y);
}

#[test]
fn driver_pause_resume_prune_and_clear_work() {
    let mut scene = SceneGraph::new();
    let node_id = scene.add(SceneNode::new("node"));
    let mut perspective = BTreeMap::<CameraId, PerspectiveCamera>::new();
    let mut orthographic = BTreeMap::<CameraId, OrthographicCamera>::new();
    let mut stores = CameraStores {
        perspective: &mut perspective,
        orthographic: &mut orthographic,
    };
    let mut materials = BTreeMap::<MaterialId, PbrMaterial>::new();
    let mut skeletons = vec![SkeletonPose::new(vec![Transform::IDENTITY])];
    let mut driver = ScenixAnimationDriver::new();
    driver.add_node(NodeAnimator::new(
        node_id,
        NodeAnimationTarget::Translation(Vec3Track::tween(Vec3::ZERO, Vec3::X, 1.0)),
    ));

    driver.pause();
    let stats = driver
        .tick(1.0, &mut scene, &mut stores, &mut materials, &mut skeletons)
        .unwrap();
    assert_eq!(stats.completed, 0);
    assert_eq!(driver.node_len(), 1);

    driver.resume();
    let stats = driver
        .tick(1.0, &mut scene, &mut stores, &mut materials, &mut skeletons)
        .unwrap();
    assert_eq!(stats.completed, 1);
    assert_eq!(driver.node_len(), 0);

    driver.add_skeleton(SkinnedMeshAnimator::new(0, Vec::new()));
    assert_eq!(driver.remove_skeleton(0).unwrap().skeleton_index, 0);
    driver.add_node(NodeAnimator::new(
        node_id,
        NodeAnimationTarget::Visibility(scenix_animato::BoolTrack::immediate(true)),
    ));
    driver.clear();
    assert!(driver.is_empty());
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trip_for_track_metadata() {
    let track = Vec3Track::tween(Vec3::ZERO, Vec3::ONE, 1.0);
    let json = serde_json::to_string(&track).unwrap();
    let round_trip: Vec3Track = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip.value(), Vec3::ZERO);
}
