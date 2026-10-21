//! Distributed `scenix-xr` v1.9.0 CPU validation suite.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo test -p scenix-xr --test xr_cpu
//! ```

use core::time::Duration;

use scenix_camera::XrEyeProjection;
use scenix_core::XrAnchorId;
use scenix_input::{XrHand, XrTrackingStatus};
use scenix_math::{Quat, Transform, Vec3};
use scenix_xr::{
    XR_ANCHOR_MAX_AGE, XR_FEATURE_COUNT, XR_MAX_VIEWPORT_SCALE, XrAmbientEstimate, XrAnchor,
    XrAnchorStore, XrControllerBay, XrFeatures, XrFoveation, XrFramePlan, XrFrameStats,
    XrHandFrame, XrLightEstimationMode, XrPlane, XrPlaneKind, XrPlaneStore, XrPose,
    XrReferenceSpace, XrSessionMode, XrSessionPlan, XrSessionPlanError, XrSessionState, XrSpace,
    XrViewport,
};

const EPS: f32 = 1.0e-5;

#[test]
fn session_plan_negotiates_immersive_ar() {
    let mut plan = XrSessionPlan::immersive_ar()
        .with_reference_space(XrReferenceSpace::BoundedFloor)
        .with_foveation(XrFoveation::High);
    assert_eq!(plan.reference_space.label(), "local-floor");
    assert_eq!(plan.foveation.to_unit(), 3);
    assert!(plan.validate().is_ok());

    plan.advance(XrSessionState::Requesting)
        .expect("idle -> requesting");
    plan.advance(XrSessionState::Starting)
        .expect("requesting -> starting");
    plan.advance(XrSessionState::Active)
        .expect("starting -> active");
    assert!(plan.is_presenting());
    assert_eq!(plan.mode.label(), "immersive-ar");
    assert_eq!(plan.features.names().count(), 6);

    // Illegal transition rejected, state untouched.
    assert_eq!(
        plan.advance(XrSessionState::Idle),
        Err(XrSessionPlanError::IllegalTransition {
            from: XrSessionState::Active,
            to: XrSessionState::Idle,
        })
    );
    assert!(plan.is_presenting());
}

#[test]
fn features_bitops_and_names() {
    assert_eq!(XR_FEATURE_COUNT, 8);
    let vr = XrFeatures::room_scale_vr();
    assert!(vr.contains(XrFeatures::CONTROLLERS));
    assert!(vr.contains(XrFeatures::HANDS));
    assert_eq!(vr.names().next(), Some("controllers"));

    let mut custom = XrFeatures::empty();
    custom |= XrFeatures::HIT_TEST | XrFeatures::ANCHORS;
    assert!(custom.contains(XrFeatures::HIT_TEST));
    assert!(!custom.contains(XrFeatures::HANDS));
    assert_eq!(custom.names().count(), 2);
    assert_eq!(custom.intersection(vr), XrFeatures::empty());
    assert_eq!(custom.without(XrFeatures::ANCHORS).names().count(), 1);
}

#[test]
fn feature_negotiation_rejects_impossible_plans() {
    // Inline sessions can't host world-probing features.
    let inline = XrSessionPlan::inline().with_features(XrFeatures::HIT_TEST | XrFeatures::ANCHORS);
    assert_eq!(
        inline.validate(),
        Err(XrSessionPlanError::FeatureRequiresImmersive)
    );

    // VR can't request passthrough.
    let vr = XrSessionPlan::immersive_vr().with_features(
        XrFeatures::room_scale_vr() | XrFeatures::PASSTHROUGH | XrFeatures::CONTROLLERS,
    );
    assert_eq!(
        vr.validate(),
        Err(XrSessionPlanError::PassthroughRequiresAr)
    );
}

#[test]
fn controllers_bay_tracks_poses_and_prediction() {
    let mut bay = XrControllerBay::new();
    assert_eq!(bay.tracked(), 0);

    let pose = XrPose::new(
        Vec3::new(0.4, 1.2, -0.3),
        Quat::from_axis_angle(Vec3::Y, -0.7),
    );
    bay.update(XrHand::Right, pose, XrTrackingStatus::Tracked);
    assert_eq!(bay.tracked(), 1);
    assert!(bay.get(XrHand::Right).is_tracking());

    // Grip pose derived deterministically from the pointer pose.
    let grip = bay.get(XrHand::Right).pose(XrSpace::Grip);
    assert!((grip.position.y + 0.05 - pose.position.y).abs() < EPS);
    assert!(grip.orientation.w.is_finite());

    // Velocity-based prediction travels along +X (dt clamps at 0.25 s).
    let bay_velocity = bay.get_mut(XrHand::Right);
    bay_velocity.linear_velocity = Vec3::new(2.0, 0.0, 0.0);
    let predicted = bay.get(XrHand::Right).predicted_pose(0.25);
    assert!((predicted.position.x - (pose.position.x + 0.5)).abs() < EPS);

    bay.reset();
    assert_eq!(bay.tracked(), 0);
    assert!((bay.get(XrHand::Right).linear_velocity.x).abs() < 1e-7);
    assert_eq!(bay.get(XrHand::Right).hand, XrHand::Right);
}

#[test]
fn hand_frames_track_joints_and_pinch() {
    let mut hand = XrHandFrame::idle(XrHand::Left);
    assert_eq!(hand.tracked_joints(), 0);
    assert!(!hand.joints[scenix_xr::JOINT_THUMB_TIP].tracked);

    hand.track_joint(
        scenix_xr::JOINT_THUMB_TIP,
        XrPose::from_position(Vec3::new(0.30, 1.00, -0.35)),
        0.02,
    );
    hand.track_joint(
        scenix_xr::JOINT_INDEX_TIP,
        XrPose::from_position(Vec3::new(0.38, 1.00, -0.35)),
        0.02,
    );
    let wrist = hand.joint(scenix_xr::JOINT_WRIST);
    assert!(wrist.is_none()); // wrist still untracked

    hand.track_joint(
        scenix_xr::JOINT_WRIST,
        XrPose::from_position(Vec3::ZERO),
        0.05,
    );
    assert!(hand.joint(scenix_xr::JOINT_WRIST).is_some());

    let distance = hand.thumb_index_distance().expect("both tips tracked");
    assert!((distance - 0.08).abs() < EPS);

    hand.pinch_strength = 0.9;
    hand.grip_strength = 0.2;
    assert!(hand.is_pinching());
    assert!(!hand.is_gripping());

    let positions = hand.tracked_positions();
    assert_eq!(positions.len(), 3);
    // Sorted for deterministic consumption.
    assert!(positions.windows(2).all(|pair| pair[0].x <= pair[1].x));
}

#[test]
fn frame_plan_advances_and_scales_viewports() {
    assert_eq!(XR_MAX_VIEWPORT_SCALE, 2.0);

    let viewport = XrViewport::new(1024, 1024).with_scale(3.0);
    assert!((viewport.scale - XR_MAX_VIEWPORT_SCALE).abs() < EPS);
    assert_eq!(viewport.physical_width(), 2048);
    assert_eq!(viewport.physical_height(), 2048);
    assert!((viewport.aspect() - 1.0).abs() < EPS);
    assert_eq!(viewport.stereo_pixel_count(), 2048 * 2048 * 2);

    let head = Transform::new(Vec3::new(0.0, 1.6, 0.0), Quat::IDENTITY, Vec3::ONE);
    let eye = XrEyeProjection::symmetric(core::f32::consts::FRAC_PI_2, 1.0, 0.1, 50.0);
    let stats = XrFrameStats {
        frame_index: 0,
        predicted_display: Duration::ZERO,
        cpu_prep_us: 400,
        submit_us: 600,
        dt_seconds: 1.0 / 72.0,
    };
    let frame = XrFramePlan::default().with_eye(eye).with_stats(stats);
    assert_eq!(frame.stats.frame_index, 0);
    assert!((frame.stats.estimated_fps() - 72.0).abs() < EPS);
    assert_eq!(frame.stats.total_cpu_us(), 1_000);

    let next = frame.next_frame(head, 1.0 / 72.0);
    assert_eq!(next.stats.frame_index, 1);
    let delta = next.stats.predicted_display.as_secs_f32() - 1.0 / 72.0;
    assert!(delta.abs() < 0.001, "display time advanced by one tick");
    let rig = next.camera_rig();
    assert!(!rig.left_view_projection().get(3, 3).is_nan());
}

#[test]
fn anchors_store_prunes_and_evicts() {
    let mut store = XrAnchorStore::with_capacity(4);
    assert!(store.insert(XrAnchor::new(
        XrAnchorId::new(3),
        XrPose::from_position(Vec3::ONE),
        10
    )));
    store.insert(XrAnchor::new(
        XrAnchorId::new(1),
        XrPose::from_position(Vec3::Y),
        10,
    ));
    store.insert(XrAnchor::new(
        XrAnchorId::new(2),
        XrPose::from_position(Vec3::Z),
        10,
    ));
    assert_eq!(store.len(), 3);
    assert!(store.contains(XrAnchorId::new(2)));
    assert!(!store.contains(XrAnchorId::new(4)));

    // Dead anchors expire just past the max age.
    let pruned = store.prune(11 + XR_ANCHOR_MAX_AGE);
    assert_eq!(pruned.len(), 3, "all slots lapse at once");

    // Eviction fills from lowest confidence deterministically.
    let mut tight = XrAnchorStore::with_capacity(2);
    tight.insert(XrAnchor::new(XrAnchorId::new(5), XrPose::default(), 1));
    tight.insert(XrAnchor::new(XrAnchorId::new(6), XrPose::default(), 1));
    let mut stale = XrAnchor::new(XrAnchorId::new(7), XrPose::default(), 1);
    stale.confidence = 0.1;
    assert!(tight.insert(stale));
    assert_eq!(tight.len(), 2);
    assert!(
        !tight.contains(XrAnchorId::new(5)) || !tight.contains(XrAnchorId::new(6)),
        "lowest-confidence slot hardly evicted"
    );

    // Live anchors stay id-ordered.
    assert!(
        tight
            .anchors()
            .windows(2)
            .all(|pair| pair[0].id.get() < pair[1].id.get())
    );

    // Best selection prefers confidence then id.
    if let Some(best) = tight.best() {
        assert_eq!(
            best.id,
            if tight.contains(XrAnchorId::new(5)) {
                XrAnchorId::new(5)
            } else {
                XrAnchorId::new(6)
            }
        );
    }

    // Anchor updates clamp confidence.
    let mut rich = XrAnchor::new(XrAnchorId::new(9), XrPose::default(), 0);
    rich.update(XrPose::from_position(Vec3::Z), 4.0, 5);
    assert!((rich.confidence - 1.0).abs() < EPS);
    let packed = rich.pack();
    assert_eq!(packed.0.len(), 7);
    assert_eq!(packed.1, 1.0);
}

#[test]
fn planes_reconcile_and_pick_floor() {
    let floor = XrPlane::new(2, XrPlaneKind::HorizontalUp, XrPose::default(), 8.0, 6.0);
    let wall = XrPlane::new(1, XrPlaneKind::Vertical, XrPose::default(), 4.0, 3.0);
    let ceiling = XrPlane::new(3, XrPlaneKind::HorizontalDown, XrPose::default(), 8.0, 6.0);

    let mut store = XrPlaneStore::new();
    let has_floor = store.reconcile(vec![wall, floor, ceiling]);
    assert!(has_floor, "floor plane detected");

    // Fixed order by id, checked before kind.
    let ids: Vec<u64> = store.planes().iter().map(|plane| plane.id).collect();
    assert_eq!(ids, vec![1, 2, 3]);

    let largest = store.largest().expect("planes present");
    assert_eq!(largest.id, 2, "floor and ceiling tied on area, id wins");
    assert!((largest.area() - 48.0).abs() < EPS);

    assert_eq!(floor.kind, XrPlaneKind::HorizontalUp);
    assert_eq!(ceiling.normal(), Vec3::new(0.0, -1.0, 0.0));
    // Plane ordering after reconcile starts with the vertical wall.
    assert_eq!(
        store.planes().first().copied().map(|p| p.kind),
        Some(XrPlaneKind::Vertical)
    );

    store.clear();
    assert!(store.planes().is_empty());
}

#[test]
fn planes_detect_floor_classification() {
    let floor = XrPlane::new(9, XrPlaneKind::HorizontalUp, XrPose::default(), 1.0, 1.0);
    assert!(floor.kind.is_floor());
    assert!(!XrPlaneKind::Vertical.is_floor());
    let ceiling = XrPlane::new(3, XrPlaneKind::HorizontalDown, XrPose::default(), 1.0, 1.0);
    assert_eq!(ceiling.normal(), Vec3::new(0.0, -1.0, 0.0));
}

#[test]
fn session_mode_semantics() {
    assert!(!XrSessionMode::Inline.is_immersive());
    assert!(XrSessionMode::Inline.is_passthrough());
    assert!(XrSessionMode::ImmersiveVr.is_immersive());
    assert!(!XrSessionMode::ImmersiveVr.is_passthrough());
    assert!(!XrReferenceSpace::Viewer.has_floor());
    assert!(XrReferenceSpace::Stage.has_floor());
    assert!(!XrSpace::TargetRay.is_grip());
}

#[test]
fn light_estimates_feed_scene_lights() {
    let ambient = XrAmbientEstimate::ambient(Vec3::new(0.8, 0.7, 0.6), 0.75)
        .with_direction(Vec3::new(0.0, -1.0, 0.0));
    assert_eq!(ambient.mode, XrLightEstimationMode::AmbientWithDirection);
    let rgb = ambient.linear_color();
    assert!(
        (rgb.x - 0.6).abs() < EPS,
        "color * intensity flows to AmbientLight"
    );
    assert!(ambient.luminance() > 0.0);

    // No scene light change when the runtime goes silent.
    let later = XrAmbientEstimate::ambient(Vec3::ONE, 0.4);
    assert_eq!(later.mode, XrLightEstimationMode::AmbientOnly);
    assert_eq!(XrLightEstimationMode::Off.label(), "off");
}
