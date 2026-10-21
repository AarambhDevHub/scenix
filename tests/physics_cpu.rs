//! Distributed `scenix-physics` v1.9.0 CPU validation suite.
//!
//! Run from the repository root:
//!
//! ```sh
//! cargo test -p scenix-physics --test physics_cpu
//! ```

use scenix_core::PhysicsBodyId;
use scenix_math::{Quat, Ray3, Transform, Vec3};
use scenix_physics::{
    CONTACT_RESOLVE_PERCENT, CONTACT_SLOP, CharacterInput, CharacterState, Collider, ColliderGroup,
    ColliderShape, GRAVITY_DEFAULT, MAX_SUBSTEP_DT, MAX_SUBSTEPS, MIN_FRICTION, PHYSICS_VERSION,
    PhysicsBody, PhysicsTransformSync, PhysicsWorld, REST_CONTACT_VELOCITY, RigidBodyKind,
    WIREFRAME_SEGMENTS, WORLD_MAX_BODIES, WorldEvent, debug_wireframe, debug_wireframe_world,
    wireframe_box, wireframe_capsule, wireframe_sphere,
};

const DT: f32 = 1.0 / 60.0;

fn touches(
    a: PhysicsBodyId,
    b: PhysicsBodyId,
    first: PhysicsBodyId,
    second: PhysicsBodyId,
) -> bool {
    (a == first && b == second) || (a == second && b == first)
}

#[test]
fn solids_settle_and_statics_survive() {
    let mut world = PhysicsWorld::earth();
    let ground = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let ball = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 3.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
        ColliderShape::sphere(0.5),
    );

    for _ in 0..240 {
        world.step(DT);
    }

    let settled = world.body(ball).expect("ball body");
    assert!(
        (settled.transform.translation.y - 1.0).abs() < 0.15,
        "ball rests on the ground, got {:?}",
        settled.transform.translation
    );
    assert!(
        settled.linear_velocity.length() < 0.1,
        "resting bodies stay quiet"
    );
    assert!(
        world.are_touching(ball, ground),
        "contact keeps the body up"
    );
    assert_eq!(world.len(), 2);
    assert_eq!(
        world.body(ground).expect("ground").kind,
        RigidBodyKind::Static
    );
}

#[test]
fn solver_is_deterministic() {
    let staged = || {
        let mut world = PhysicsWorld::earth();
        let _floor = world.add_solid(
            PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
            ColliderShape::boxy(20.0, 0.5, 20.0),
        );
        let _settler = world.add_solid(
            PhysicsBody::new(Transform::new(
                Vec3::new(0.6, 2.4, -0.2),
                Quat::IDENTITY,
                Vec3::ONE,
            ))
            .with_kind(RigidBodyKind::Dynamic)
            .with_mass(1.4)
            .with_surface(0.3, 0.4),
            ColliderShape::sphere(0.5),
        );
        let _drifter = world.add_solid(
            PhysicsBody::new(Transform::new(
                Vec3::new(-0.6, 3.2, 0.2),
                Quat::IDENTITY,
                Vec3::ONE,
            ))
            .with_kind(RigidBodyKind::Dynamic)
            .with_mass(0.8)
            .with_surface(0.1, 0.6),
            ColliderShape::capsule(0.3, 0.4),
        );
        world
    };

    let (mut a, mut b) = (staged(), staged());
    for _ in 0..120 {
        a.step(DT);
        b.step(DT);
    }

    assert_eq!(a.len(), b.len());
    for index in 0..a.len() {
        let (ba, bb) = (a.body_at(index).unwrap(), b.body_at(index).unwrap());
        assert_eq!(ba.transform, bb.transform, "transform drift at {index}");
        assert_eq!(
            ba.linear_velocity, bb.linear_velocity,
            "velocity drift at {index}"
        );
    }
    assert_eq!(a.contacts(), b.contacts());
    assert_eq!(a.body_pair_count(), b.body_pair_count());
}

#[test]
fn restitution_lofts_the_ball() {
    let mut world = PhysicsWorld::earth();
    let _ground = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let ball = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 3.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic)
        .with_surface(0.85, 0.02),
        ColliderShape::sphere(0.5),
    );

    let mut saw_floor = false;
    let mut rebound = 0.0f32;
    for _ in 0..180 {
        world.step(DT);
        let y = world.body(ball).unwrap().transform.translation.y;
        if y < 1.3 {
            saw_floor = true;
        }
        if saw_floor {
            rebound = rebound.max(y);
        }
    }
    assert!(saw_floor, "the ball reached the ground");
    assert!(rebound > 1.5, "e=0.85 lofted the ball to {rebound} m");
}

#[test]
fn friction_stops_sliding_boxes() {
    let mut world = PhysicsWorld::earth();
    let _ground = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let crate_id = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(-1.0, 1.05, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic)
        .with_surface(0.0, 0.92),
        ColliderShape::boxy(0.5, 0.5, 0.5),
    );
    world.body_mut(crate_id).expect("body").linear_velocity = Vec3::new(5.0, 0.0, 0.0);

    for _ in 0..90 {
        world.step(DT);
    }
    let body = world.body(crate_id).expect("body");
    assert!(
        body.linear_velocity.length() < 1.0,
        "μ=0.92 drained the 5 m/s slide, got {:?}",
        body.linear_velocity
    );
}

#[test]
fn contact_normals_face_a_to_b() {
    let mut world = PhysicsWorld::earth();
    let _ground = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let _ball = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 3.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
        ColliderShape::sphere(0.5),
    );

    for _ in 0..240 {
        world.step(DT);
    }

    let contacts = world.contacts();
    assert!(
        !contacts.is_empty(),
        "a resting body still publishes contacts"
    );
    for contact in contacts {
        assert!(contact.penetration >= 0.0);
        assert!(
            contact.a.get() < contact.b.get(),
            "a is always the lower id"
        );
        if contact.normal.y.abs() > 0.5 {
            assert!(contact.normal.y > 0.0, "normal points toward the higher id");
        }
    }
}

#[test]
fn group_masks_gate_pairs() {
    let crew = ColliderGroup::all()
        .with_membership(0b0001)
        .with_filter(0b0001);
    let guest = ColliderGroup::all()
        .with_membership(0b0010)
        .with_filter(0b0010);
    let bridge = ColliderGroup::all()
        .with_membership(0b0011)
        .with_filter(u32::MAX);

    assert!(
        crew.interacts_with(&crew),
        "a faction always touches itself"
    );
    assert!(!crew.interacts_with(&guest), "disjoint crews stay apart");
    assert!(bridge.interacts_with(&crew), "the bridge hears the crew");
    assert!(bridge.interacts_with(&guest), "the bridge hears the guest");
}

#[test]
fn insertion_honors_custom_groups() {
    let mut world = PhysicsWorld::earth();
    let left = world.add_body(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.3, 2.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
    );
    let _left_sensor = world.insert_collider(
        Collider::new(left, ColliderShape::sphere(0.5))
            .with_group(ColliderGroup::all().with_membership(1).with_filter(1)),
    );
    let right = world.add_body(
        PhysicsBody::new(Transform::new(
            Vec3::new(-0.3, 2.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
    );
    let _right_sensor = world.insert_collider(
        Collider::new(right, ColliderShape::sphere(0.5))
            .with_group(ColliderGroup::all().with_membership(2).with_filter(2)),
    );

    // Disjoint groups never push each other: both sink side by side and the
    // 0.6 m separation survives the transient overlap.
    for _ in 0..30 {
        world.step(DT);
    }
    let gap = world.body(left).unwrap().transform.translation
        - world.body(right).unwrap().transform.translation;
    assert!(
        (gap.length() - 0.6).abs() < 1.0e-3,
        "gap = {}",
        gap.length()
    );
}

#[test]
fn sensors_flag_entries_without_resolving() {
    let mut world = PhysicsWorld::earth();
    let _ground = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let ball = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 4.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
        ColliderShape::sphere(0.5),
    );
    let watcher = world.add_body(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 2.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::KinematicPosition),
    );
    let sensor = world
        .insert_collider(Collider::new(watcher, ColliderShape::boxy(0.5, 0.5, 0.5)).as_sensor());

    let mut entered = 0;
    for _ in 0..240 {
        world.step(DT);
        for event in world.take_events() {
            if let WorldEvent::SensorEntered { sensor: id, other } = event {
                assert_eq!(id, sensor);
                assert_eq!(other, ball);
                entered += 1;
            }
        }
    }
    assert!(entered >= 1, "the floating sensor flagged the passing ball");

    let rested = world.body(ball).unwrap().transform.translation;
    assert!(
        (rested.y - 1.0).abs() < 0.15,
        "sensor hits never push, table-top rest at {rested:?}"
    );
}

#[test]
fn spatial_queries_hit_and_overlap() {
    let mut world = PhysicsWorld::earth();
    let wall = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(6.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(0.5, 3.0, 3.0),
    );
    let block = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(2.5, 0.5, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(0.5, 0.5, 0.5),
    );

    let wall_ray = Ray3::new(Vec3::new(0.0, 1.5, 0.0), Vec3::X);
    let hit = world.cast_ray(wall_ray, 100.0).expect("a wall strike");
    assert_eq!(hit.body, wall);
    assert!(
        (hit.distance - 5.5).abs() < 0.05,
        "wall face distance = {}",
        hit.distance
    );
    assert!(
        hit.normal.x < -0.9,
        "the face normal points back at the ray"
    );

    let block_ray = Ray3::new(Vec3::new(2.5, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
    let dropped = world.cast_ray(block_ray, 100.0).expect("a block strike");
    assert_eq!(dropped.body, block);
    assert!(
        (dropped.distance - 4.0).abs() < 0.05,
        "block top distance = {}",
        dropped.distance
    );
    assert!(dropped.normal.y > 0.9);

    let miss = world.cast_ray(Ray3::new(Vec3::new(0.0, 5.0, 0.0), Vec3::Z), 3.0);
    assert!(miss.is_none(), "side ray leaves the scene untouched");

    let sweep = world.overlap_sphere(Vec3::new(4.0, 1.0, 0.0), 2.5);
    assert!(sweep.contains(&wall));
    assert!(sweep.contains(&block));
    let empty = world.overlap_sphere(Vec3::new(-50.0, -50.0, 0.0), 1.0);
    assert!(empty.is_empty());
}

#[test]
fn pair_events_book_end_on_separation() {
    let mut world = PhysicsWorld::earth();
    let first = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic)
        .with_surface(0.9, 0.02),
        ColliderShape::sphere(0.35),
    );
    let second = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.5, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic)
        .with_surface(0.9, 0.02),
        ColliderShape::sphere(0.35),
    );
    world.body_mut(first).expect("body").linear_velocity = Vec3::new(1.5, 0.0, 0.0);
    world.body_mut(second).expect("body").linear_velocity = Vec3::new(-1.5, 0.0, 0.0);

    let (mut started, mut ended) = (false, false);
    for _ in 0..240 {
        world.step(DT);
        for event in world.take_events() {
            match event {
                WorldEvent::ContactStarted { a, b } => {
                    if touches(a, b, first, second) {
                        started = true;
                    }
                }
                WorldEvent::ContactEnded { a, b } if touches(a, b, first, second) => {
                    ended = true;
                }
                _ => {}
            }
        }
    }
    assert!(started, "the overlapping pair announced contact");
    assert!(ended, "a rebounding pair separates and ends contact");
}

#[test]
fn sleeping_bodies_rest_and_wake_on_impulse() {
    let mut world = PhysicsWorld::earth();
    let _ground = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let ball = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 3.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
        ColliderShape::sphere(0.5),
    );

    let mut slept = false;
    for _ in 0..300 {
        world.step(DT);
        for event in world.take_events() {
            if let WorldEvent::BodySlept { body } = event {
                slept |= body == ball;
            }
        }
    }
    assert!(slept, "a settled ball crosses the sleep threshold");
    assert!(world.body(ball).unwrap().sleep.asleep);

    world
        .body_mut(ball)
        .expect("body")
        .apply_impulse(Vec3::new(0.0, 3.0, 0.0));
    let woken = world.body(ball).expect("body");
    assert!(!woken.sleep.asleep, "impulses re-light the body");
    assert!(woken.linear_velocity.y > 2.0);
}

#[test]
fn character_walks_jumps_lands_and_respawns() {
    let mut world = PhysicsWorld::earth();
    let _ground = world.add_solid(
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

    let idle = CharacterInput::idle(DT);
    for _ in 0..90 {
        state.move_and_slide(&mut world, &idle);
        world.step(DT);
    }
    assert!(state.ground.landed, "the floor probe catches the character");

    // Walking writes the wish velocity and slides the capsule.
    let walk = CharacterInput::move_towards(Vec3::X, 3.0, DT);
    let stride = state.move_and_slide(&mut world, &walk);
    assert!(
        stride.moved.length() > 0.0,
        "the wish displacement is reported"
    );
    let start = world.body(player).unwrap().transform.translation;
    for _ in 0..30 {
        state.move_and_slide(&mut world, &walk);
        world.step(DT);
    }
    let walked = world.body(player).unwrap().transform.translation.x - start.x;
    assert!(
        (walked - 1.5).abs() < 0.6,
        "walked {walked} m in 30 frames of 3 m/s"
    );

    // Jumping launches vertically.
    let jump = CharacterInput {
        wish_direction: Vec3::X,
        speed: 0.0,
        jump: true,
        dt: DT,
    };
    state.move_and_slide(&mut world, &jump);
    assert!(!state.ground.landed, "jumping lifts the ground flag");
    let launch = world.body(player).expect("body");
    assert!(
        (launch.linear_velocity.y - 5.0).abs() < 1.0e-4,
        "the default jump speed is applied"
    );
    let launch_y = launch.transform.translation.y;
    world.step(DT);
    assert!(
        world.body(player).unwrap().transform.translation.y > launch_y,
        "the physics step carries the jump"
    );

    // The capsule clears the floor and eventually lands again.
    for _ in 0..120 {
        state.move_and_slide(&mut world, &idle);
        world.step(DT);
    }
    assert!(state.ground.landed, "the character lands again");

    // Respawning freezes all momentum.
    state.respawn(
        Transform::new(Vec3::new(4.0, 1.4, 0.0), Quat::IDENTITY, Vec3::ONE),
        &mut world,
    );
    let body = world.body(player).expect("body");
    assert_eq!(body.transform.translation.x, 4.0);
    assert!(
        body.linear_velocity.length() < 1.0e-6,
        "teleport pads settle the controller"
    );
}

#[test]
fn sync_bindings_drive_writebacks() {
    let mut world = PhysicsWorld::earth();
    let falling = world.add_body(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 3.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
    );
    let _falling_collider = world.attach_collider(falling, ColliderShape::sphere(0.5));
    let anchor =
        world.add_body(PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static));
    let _anchor_collider = world.attach_collider(anchor, ColliderShape::boxy(10.0, 0.5, 10.0));

    let mut sync = PhysicsTransformSync::new();
    assert!(sync.bind(falling, 0x02));
    assert!(sync.bind(anchor, 0xA1));
    assert!(
        !sync.bind(falling, 0x03),
        "bindings are one body to one node"
    );

    let queue = sync.collect(&world);
    assert_eq!(queue.len(), 1, "static anchors are skipped");
    let entry = queue.sorted()[0];
    assert_eq!(entry.body, falling);
    assert_eq!(entry.layer, 2);
    assert_eq!(entry.transform, world.body(falling).unwrap().transform);

    assert!(sync.unbind(anchor));
    assert_eq!(sync.len(), 1);
    assert!(!sync.unbind(anchor), "stale unbinds report false");
}

#[test]
fn collider_shapes_carry_packed_keys_and_bounds() {
    // Shape keys tag the variant, not the parameters.
    assert_eq!(ColliderShape::sphere(0.5).as_packed_key(), 1);
    assert_eq!(ColliderShape::sphere(2.0).as_packed_key(), 1);
    assert_eq!(ColliderShape::boxy(1.0, 1.0, 1.0).as_packed_key(), 2);
    assert_eq!(ColliderShape::capsule(0.35, 0.55).as_packed_key(), 3);
    assert_eq!(ColliderShape::Plane.as_packed_key(), 4);

    let capsule = ColliderShape::capsule(0.35, 0.55);
    let local = capsule.local_aabb();
    let half = local.half_extents();
    assert!((half.x - 0.35).abs() < 1.0e-5);
    assert!((half.y - 0.9).abs() < 1.0e-5, "caps reach past the shaft");
    assert!((half.z - 0.35).abs() < 1.0e-5);
    assert!((capsule.bounding_radius() - 0.9).abs() < 1.0e-5);

    let placed = Transform::new(
        Vec3::new(2.0, 4.0, 6.0),
        Quat::IDENTITY,
        Vec3::new(1.0, 2.0, 1.0),
    );
    let world_aabb = capsule.world_aabb(&placed);
    assert_eq!(world_aabb.center(), Vec3::new(2.0, 4.0, 6.0));
    let world_half = world_aabb.half_extents();
    assert!(
        (world_half.y - 1.8).abs() < 1.0e-5,
        "per-axis scale applies"
    );
    assert!((world_half.x - 0.35).abs() < 1.0e-5);

    let key = ColliderShape::boxy(0.4, 0.2, 0.4).as_packed_key();
    assert_eq!(key, ColliderShape::boxy(5.0, 1.0, 2.0).as_packed_key());
}

#[test]
fn wireframes_describe_shapes_exactly() {
    let identity = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
    let plane = debug_wireframe(&ColliderShape::Plane, &identity);
    assert_eq!(plane.len(), 4, "the plane wireframe is one square tile");

    let sphere = wireframe_sphere(Vec3::ZERO, 1.0);
    assert_eq!(sphere.len(), 3 * WIREFRAME_SEGMENTS);
    for ring_start in [0usize, WIREFRAME_SEGMENTS, 2 * WIREFRAME_SEGMENTS] {
        let ring_end = ring_start + WIREFRAME_SEGMENTS - 1;
        for offset in 1..WIREFRAME_SEGMENTS {
            assert_eq!(
                sphere[ring_start + offset].0,
                sphere[ring_start + offset - 1].1,
                "ring chains head to tail"
            );
        }
        // Closure: the last segment lands on the first vertex.
        let open = (sphere[ring_end].1 - sphere[ring_start].0).length();
        assert!(open < 1.0e-5, "ring closes with gap {open}");
    }
    for (a, b) in &sphere {
        assert!((a.length() - 1.0).abs() < 1.0e-5);
        assert!((b.length() - 1.0).abs() < 1.0e-5);
    }

    let box_segs = wireframe_box(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(box_segs.len(), 12);
    for (a, b) in box_segs {
        let diff = a - b;
        let changed = [diff.x, diff.y, diff.z]
            .iter()
            .filter(|axis| axis.abs() > 1.0e-5)
            .count();
        assert_eq!(changed, 1, "box edges are axis aligned");
    }

    let up = Transform::new(Vec3::new(0.0, 2.0, 0.0), Quat::IDENTITY, Vec3::ONE);
    let capsule = wireframe_capsule(&up, 0.55, 0.35);
    assert_eq!(
        capsule.len(),
        2 * WIREFRAME_SEGMENTS + 4,
        "rings plus rails"
    );
    for (a, b) in capsule.iter().take(WIREFRAME_SEGMENTS) {
        assert!((a.y - b.y).abs() < 1.0e-5, "bottom ring is flat");
        assert!((a.y - 1.45).abs() < 1.0e-5);
    }
    for (a, b) in capsule.iter().rev().take(4) {
        assert!((a.y - b.y).abs() > 1.0, "rails run along the axis");
    }
}

#[test]
fn debug_wireframe_world_tags_every_collider() {
    let mut world = PhysicsWorld::earth();
    let _ground = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(20.0, 0.5, 20.0),
    );
    let _loom = world.add_solid(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.0, 1.6, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic),
        ColliderShape::capsule(0.35, 0.55),
    );

    let tagged = debug_wireframe_world(&world);
    assert_eq!(tagged.len(), world.colliders().len(), "one entry per body");
    assert_eq!(tagged.len(), 2);
    assert_ne!(tagged[0].0, tagged[1].0, "collider ids are distinct");
    for (_, segs) in tagged {
        assert!(!segs.is_empty());
    }
}

#[test]
fn solver_constants_are_pinned() {
    assert_eq!(CONTACT_RESOLVE_PERCENT, 0.2);
    assert_eq!(CONTACT_SLOP, 0.005);
    assert_eq!(REST_CONTACT_VELOCITY, 0.4);
    assert_eq!(MIN_FRICTION, 0.02);
    assert_eq!(MAX_SUBSTEP_DT, 1.0 / 120.0);
    assert_eq!(MAX_SUBSTEPS, 4);
    assert_eq!(WORLD_MAX_BODIES, 4_096);
    assert_eq!(GRAVITY_DEFAULT, 9.81);
    assert_eq!(PHYSICS_VERSION, "1.9.1");
}
