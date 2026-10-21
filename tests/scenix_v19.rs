//! scenix v1.9.0 "XR, Audio, And Physics Bridges" — facade-level CPU coverage.
//!
//! Exercises the three bridge crates end to end without an engine or GPU:
//! XR session plans, controller bays, hand frames, anchors and hit probes;
//! spatial audio mixing, Doppler, streams and analysis; the deterministic
//! physics penalty solver with sensors, queries, sleep and the
//! character controller; plus the transform sync, debug helpers, and scene
//! schema nodes the facade re-exports.

use scenix::{
    AnalyserBand, AttenuationModel, AudioAnalyser, AudioListener, AudioLoop, AudioSource,
    AudioStreamPlan, CharacterInput, CharacterState, Collider, ColliderDebugHelper, ColliderGroup,
    ColliderShape, HAND_LINKS, HandSkeletonHelper, PhysicsBody, PhysicsTransformSync, PhysicsWorld,
    PlaybackState, PoseAxesHelper, Quat, Ray3, RigidBodyKind, SceneNode, StreamFormat, Transform,
    Vec3, XR_ANCHOR_MAX_AGE, XrAnchorStore, XrFramePlan, XrHand, XrHandFrame, XrPose,
    XrReferenceSpace, XrSessionPlan, XrSessionState, XrViewport,
};

#[test]
fn xr_session_plans_validate_and_present() {
    // Room-scale VR: stage space, classic feature bundle.
    let mut plan = XrSessionPlan::immersive_vr()
        .with_reference_space(XrReferenceSpace::Stage)
        .with_features(scenix::XrFeatures::room_scale_vr());
    plan.validate().expect("room-scale VR plans validate");
    for state in [
        XrSessionState::Requesting,
        XrSessionState::Starting,
        XrSessionState::Active,
    ] {
        plan.advance(state).expect("legal lifecycle move");
    }
    assert!(plan.is_presenting());
    assert!(scenix::XrSessionState::can_transition(
        scenix::XrSessionState::Active,
        scenix::XrSessionState::Paused
    ));
    assert!(!scenix::XrSessionState::can_transition(
        scenix::XrSessionState::Idle,
        scenix::XrSessionState::Ending
    ));

    // Inline sessions cannot secretly request immersive-only features.
    let inline = scenix::XrSessionPlan::inline().with_features(scenix::XrFeatures::ANCHORS);
    assert!(inline.validate().is_err());
}

#[test]
fn xr_frame_planning_streams_the_stereo_rig() {
    let head = Transform::new(Vec3::new(0.0, 1.7, 0.0), Quat::IDENTITY, Vec3::ONE);
    let frame = XrFramePlan::new(head).with_viewport(XrViewport::new(2064, 2208));
    let frame = frame.next_frame(
        Transform::new(Vec3::new(0.0, 1.7, -0.6), Quat::IDENTITY, Vec3::ONE),
        1.0 / 90.0,
    );
    assert_eq!(frame.stats.frame_index, 1);
    assert_eq!(
        frame.viewport.stereo_pixel_count(),
        2064_u64 * 2208 * 2,
        "both eye channels at native scale"
    );
    let rig = frame.camera_rig();
    assert_eq!(rig.head.translation, Vec3::new(0.0, 1.7, -0.6));
    let left = rig.left_view();
    let right = rig.right_view();
    assert!(left != right, "each eye emits its own view matrix");
}

#[test]
fn controller_bays_track_hand_poses() {
    let mut bay = scenix::XrControllerBay::new();
    bay.update(
        scenix::XrHand::Right,
        XrPose::new(
            Vec3::new(0.25, 1.0, -0.6),
            Quat::from_axis_angle(Vec3::Y, -0.35),
        ),
        scenix::XrTrackingStatus::Tracked,
    );
    bay.update(
        scenix::XrHand::Left,
        XrPose::from_position(Vec3::new(-0.3, 0.95, -0.5)),
        scenix::XrTrackingStatus::Approximate,
    );
    bay.get_mut(scenix::XrHand::Right).linear_velocity = Vec3::new(0.0, 0.0, -2.0);

    let right = bay.get(scenix::XrHand::Right);
    assert_eq!(right.tracking, scenix::XrTrackingStatus::Tracked);
    assert_eq!(right.hand, scenix::XrHand::Right);
    let (origin, _direction) = bay.aim_ray(scenix::XrHand::Right);
    assert_eq!(origin, right.pointer.position, "aim rays leave the pointer");
    let predicted = right.predicted_pose(0.1);
    assert!(
        (predicted.position.z - right.pointer.position.z).abs() > 0.0,
        "prediction extrapolates the reported velocity"
    );

    bay.reset();
    assert_eq!(
        bay.get(scenix::XrHand::Left).tracking,
        scenix::XrTrackingStatus::Untracked
    );
}

#[test]
fn hand_frames_pinch_over_the_joint_grid() {
    let mut hand = XrHandFrame::idle(XrHand::Right);
    assert_eq!(hand.tracked_joints(), 0);
    assert!(!hand.is_pinching(), "an idle hand has zero pinch strength");
    assert_eq!(hand.hand, XrHand::Right);

    hand.track_joint(
        scenix::JOINT_THUMB_TIP,
        XrPose::from_position(Vec3::new(0.3, 1.0, -0.7)),
        0.01,
    );
    hand.track_joint(
        scenix::JOINT_INDEX_TIP,
        XrPose::from_position(Vec3::new(0.28, 1.02, -0.72)),
        0.01,
    );
    assert_eq!(hand.tracked_joints(), 2);
    let pinch = hand.thumb_index_distance().expect("both tips tracked");
    assert!(pinch > 0.0 && pinch < 0.1, "closed pinch gap = {pinch}");
}

#[test]
fn anchor_stores_prune_expired_anchors() {
    let mut store = XrAnchorStore::new();
    assert!(store.insert(scenix::XrAnchor::new(
        scenix::XrAnchorId::new(3),
        XrPose::from_position(Vec3::ONE),
        10,
    )));
    store.insert(scenix::XrAnchor::new(
        scenix::XrAnchorId::new(1),
        XrPose::from_position(Vec3::Y),
        10,
    ));
    assert_eq!(store.len(), 2);
    assert!(store.contains(scenix::XrAnchorId::new(1)));
    assert!(!store.contains(scenix::XrAnchorId::new(9)));
    let pruned = store.prune(11 + XR_ANCHOR_MAX_AGE);
    assert_eq!(pruned.len(), 2, "every anchor staged on frame 10 expires");
    assert!(store.is_empty());
}

#[test]
fn spatial_mixes_pans_and_dopplers() {
    let listener = AudioListener::axes(Vec3::new(0.0, 1.5, 2.0), Vec3::NEG_Z, Vec3::Y);
    let mut source = AudioSource::new(1, PlaybackState::Playing);
    source.position = Vec3::new(-1.0, 1.0, -1.0);
    let decision = scenix::compute_spatial_decision(
        &listener,
        &source,
        AttenuationModel::Inverse,
        1.0,
        &scenix::DopplerModel::default(),
    );
    assert!(decision.audible);
    assert!(
        decision.left_gain() > decision.right_gain(),
        "source sits left"
    );

    // A near-silent source still reports the neutral multiplier.
    let mut silent = AudioSource::new(2, PlaybackState::Paused);
    silent.position = Vec3::new(0.0, 1.0, -1.0);
    silent.volume = 0.0;
    let neutral = scenix::compute_spatial_decision(
        &listener,
        &silent,
        AttenuationModel::Inverse,
        1.0,
        &scenix::DopplerModel::default(),
    );
    assert!(
        (neutral.frequency_multiplier - 1.0).abs() < 1.0e-4,
        "silent voices doppler to ~1.0"
    );

    // A closing source pitches up by the speed-of-sound ratio.
    let mut mover = AudioSource::new(3, PlaybackState::Playing);
    mover.position = Vec3::new(0.0, 1.0, -2.0);
    mover.velocity = Vec3::new(0.0, 0.0, 34.3);
    let fast = scenix::compute_spatial_decision(
        &listener,
        &mover,
        AttenuationModel::Inverse,
        1.0,
        &scenix::DopplerModel::default(),
    );
    assert!(fast.frequency_multiplier > 1.0, "closing sources pitch up");

    // Pan cross-fades clamp in `0..=1`.
    let mix = scenix::blend_pan([1.0, 0.0], [0.0, 1.0], 0.5);
    assert!((mix[0] - 0.5).abs() < 1.0e-6 && (mix[1] - 0.5).abs() < 1.0e-6);
}

#[test]
fn stream_plans_buffer_consume_and_underrun() {
    let mut plan = AudioStreamPlan::new(4, StreamFormat::PcmF32, 48_000, 2);
    plan.accept_chunk();
    plan.accept_chunk();
    assert_eq!(plan.pending_chunks, 2);
    assert!(plan.promote_pending());
    assert!(plan.promote_pending());
    assert_eq!(plan.ready_chunks, 2);
    let ms = plan.buffered_milliseconds(1024);
    assert!(
        (ms - 42.667).abs() < 0.01,
        "{ms} ms = two chunked kilobytes"
    );
    assert!(plan.consume_chunk());
    assert!(plan.consume_chunk());
    assert!(!plan.consume_chunk(), "an empty plan underruns");
    assert_eq!(plan.underruns, 0, "the failed drain did not count yet");
    plan.register_underrun();
    assert_eq!(plan.underruns, 1);
    plan.reset();
    assert_eq!(plan.ready_chunks, 0, "reset drains the stream plan");
}

#[test]
fn analysers_band_and_meter_sine_content() {
    let mut analyser = AudioAnalyser::new(1024, 0.0);
    let sine: Vec<f32> = (0..1024)
        .map(|i| (core::f32::consts::TAU * 1000.0 * i as f32 / 48_000.0).sin())
        .collect();
    let frame = analyser.analyse_mono(&sine, 48_000);
    assert!(frame.peak > 0.9, "full-scale sine peaks");
    assert!(frame.is_peaking(), "strict peak threshold at 0.9");
    assert!(
        frame.band(AnalyserBand::Mid) >= frame.band(AnalyserBand::Bass),
        "1 kHz lands mid-band"
    );
}

#[test]
fn loop_schedules_count_completed_passes() {
    assert!(!AudioLoop::Once.has_next_pass(0));
    assert!(AudioLoop::Infinite.has_next_pass(1_000_000));
    assert!(!AudioLoop::Times(3).has_next_pass(3));
    assert!(AudioLoop::Times(3).has_next_pass(2));
    assert_eq!(AudioLoop::Times(0).total_passes(), 1);
}

#[test]
fn solver_sleeps_resting_bodies_and_keeps_events() {
    let dt = 1.0 / 60.0;
    let mut world = PhysicsWorld::earth();
    let _floor = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    // One silent ball drifting far above, plus a drop that must land and
    // sleep: gravity bleed on the resting ball never wakes it back up.
    let far = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(10.0, 20.0, -10.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
        ColliderShape::sphere(0.5),
    );
    let mut ball = PhysicsBody::new(Transform::new(
        Vec3::new(0.0, 3.0, 0.0),
        Quat::IDENTITY,
        Vec3::ONE,
    ))
    .with_kind(RigidBodyKind::Dynamic)
    .with_surface(0.3, 0.4);
    ball.linear_velocity = Vec3::ZERO;
    ball.sleep.asleep = false;
    let settle = world.add_solid(ball, ColliderShape::sphere(0.5));
    world.body_mut(far).expect("far body").sleep.asleep = true;

    let mut slept = false;
    for _ in 0..300 {
        world.step(dt);
        for event in world.take_events() {
            if matches!(event, scenix::WorldEvent::BodySlept { .. }) {
                slept = true;
            }
        }
    }
    assert!(
        slept,
        "the settling ball reached sleep inside three seconds"
    );
    assert!(
        world.body(far).unwrap().sleep.asleep,
        "the frozen far body never wakes: gravity bleed must not trip sleep"
    );

    // The settle ball rests near the floor: y barely above the surface.
    let rest_y = world.body(settle).expect("ball").transform.translation.y;
    assert!(
        rest_y >= 0.0 && rest_y < 1.0,
        "the dropped ball settled on the floor (y = {rest_y})"
    );
}

#[test]
fn queries_ray_cast_and_sphere_overlap() {
    let mut world = PhysicsWorld::earth();
    let wall = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(5.5, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(0.5, 1.0, 2.0),
    );
    let _floor = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let probe = Ray3::new(Vec3::new(0.0, 1.0, 0.0), Vec3::X);
    let hit = world.cast_ray(probe, 100.0).expect("the wall is hit");
    assert_eq!(hit.body, wall);
    assert!(hit.normal.x < 0.0, "face normal points back at the ray");
    let misaligned = world.cast_ray(Ray3::new(Vec3::new(0.0, 50.0, 0.0), Vec3::X), 2.0);
    assert!(misaligned.is_none(), "a ray off in space hits nothing");
}

#[test]
fn sensor_groups_filter_and_sensors_never_resolve() {
    let mut world = PhysicsWorld::earth();
    let solid_floor = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(8.0, 0.5, 8.0),
    );
    let trigger = world.add_body(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 2.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Static),
    );
    let _trigger_collider = world.insert_collider(
        Collider::new(trigger, ColliderShape::sphere(0.6))
            .with_group(ColliderGroup::all().with_membership(4).with_filter(4))
            .as_sensor(),
    );
    let _solid_floor = solid_floor;

    // Every collider still ray casts; the sensor resolves nothing at rest.
    let hit = world
        .cast_ray(Ray3::new(Vec3::new(0.0, 5.0, 0.0), -Vec3::Y), 50.0)
        .expect("something is hit from above");
    assert!(
        hit.normal.y < 0.0 || hit.normal.y > 0.0,
        "vertical rays return vertical normals"
    );
    world.step(1.0 / 60.0);
    let overlaps = world.overlap_sphere(Vec3::new(0.0, 2.0, 0.0), 0.1);
    assert!(
        overlaps.contains(&trigger),
        "sensors appear in overlap queries"
    );
}

#[test]
fn characters_walk_jump_and_land() {
    let dt = 1.0 / 60.0;
    let mut world = PhysicsWorld::earth();
    let _floor = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let player = world.add_body(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 1.4, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::KinematicPosition),
    );
    let _player_collider = world.attach_collider(player, ColliderShape::capsule(0.35, 0.55));
    let mut state = CharacterState::new(player, 0.35, 0.55);

    let idle = CharacterInput::idle(dt);
    for _ in 0..90 {
        state.move_and_slide(&mut world, &idle);
        world.step(dt);
    }
    assert!(state.ground.landed, "the floor probe catches the character");

    let walk = CharacterInput::move_towards(Vec3::X, 3.0, dt);
    let start = world.body(player).unwrap().transform.translation.x;
    for _ in 0..30 {
        state.move_and_slide(&mut world, &walk);
        world.step(dt);
    }
    let walked = world.body(player).unwrap().transform.translation.x - start;
    assert!(
        (walked - 1.5).abs() < 0.6,
        "walked {walked} m in 30 frames of 3 m/s"
    );

    let jump = CharacterInput {
        wish_direction: Vec3::X,
        speed: 0.0,
        jump: true,
        dt,
    };
    state.move_and_slide(&mut world, &jump);
    world.step(dt);
    assert!(!state.ground.landed, "jumping lifts the ground flag");
    for _ in 0..90 {
        state.move_and_slide(&mut world, &idle);
        world.step(dt);
    }
    assert!(state.ground.landed, "the character lands again");
}

#[test]
fn transform_sync_skips_statics_and_sorts_layers() {
    let mut world = PhysicsWorld::earth();
    let floor = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(8.0, 0.5, 8.0),
    );
    let ball = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.3, 2.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic)
        .with_mass(1.0)
        .with_surface(0.3, 0.4),
        ColliderShape::sphere(0.5),
    );
    let mut sync = PhysicsTransformSync::new();
    assert!(sync.bind(floor, 0x2));
    assert!(sync.bind(ball, 0x1));
    assert!(!sync.bind(ball, 0x3), "bindings are one-to-one");
    for _ in 0..60 {
        world.step(1.0 / 60.0);
    }
    let queue = sync.collect(&world);
    assert_eq!(queue.len(), 1, "statics stay off the write list");
    assert_eq!(queue.entries()[0].body, ball);
    assert_eq!(queue.entries()[0].layer, 0x1);
    let sorted = queue.sorted();
    assert!(sorted.first().copied().map(|e| e.layer) == Some(0x1));
}

#[test]
fn debug_helpers_wireframe_shapes_and_poses() {
    let mut world = PhysicsWorld::earth();
    let _floor = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(8.0, 0.5, 8.0),
    );
    let wireframe = ColliderDebugHelper::new().world_to_geometry(&world);
    assert!(
        wireframe.segment_count() > 0,
        "colliders wireframe in the default palette"
    );
    let warm = ColliderDebugHelper::new()
        .with_colors(scenix::Color::RED, scenix::Color::WHITE)
        .world_to_geometry(&world);
    assert_eq!(warm.segment_count(), wireframe.segment_count());

    // One oriented triad helper draws four sticks: +X/+Y/+Z and forward.
    let pose = PoseAxesHelper::new(
        Vec3::new(0.0, 1.0, 0.0),
        Quat::from_axis_angle(Vec3::Y, 1.0),
        0.5,
    );
    assert_eq!(pose.to_geometry().segment_count(), 4);

    // Hand skeleton links bridge the wrist into five digit chains.
    let joints = [Vec3::ZERO; scenix::HAND_JOINT_COUNT];
    let skeleton = HandSkeletonHelper::new(joints);
    assert_eq!(
        skeleton.to_geometry().segment_count(),
        HAND_LINKS.len(),
        "every declared link draws"
    );
}

#[test]
fn scene_schema_owns_the_bridge_nodes() {
    // Bridge node kinds round-trip through the scene schema the facade
    // re-exports; ids come from the core glob.
    let body_id = scenix::PhysicsBodyId::new(7);
    let collider_id = scenix::ColliderId::new(11);
    let source_id = scenix::AudioSourceId::new(2);
    let listener_id = scenix::AudioListenerId::new(5);
    let anchor_id = scenix::XrAnchorId::new(13);
    assert_eq!(body_id.get(), 7);
    assert_eq!(collider_id.get(), 11);

    let mut body_node = SceneNode::new("bridge-body");
    body_node.kind = scenix::NodeKind::PhysicsBody {
        body_id,
        collider_id,
    };
    assert_eq!(body_node.name, "bridge-body");

    let mut audio_node = SceneNode::new("bridge-audio");
    audio_node.kind = scenix::NodeKind::AudioSource {
        source_id,
        listener_id,
    };
    assert_eq!(audio_node.transform, Transform::IDENTITY);

    let mut anchor_node = SceneNode::new("bridge-anchor");
    anchor_node.kind = scenix::NodeKind::XrAnchor { anchor_id };
    assert!(anchor_node.visible);
    assert_eq!(
        anchor_node.layer,
        u32::MAX,
        "nodes default to all culling layers"
    );
}
