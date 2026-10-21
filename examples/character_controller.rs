//! Deterministic first-person character control on the CPU bridge.
//!
//! Grounds a kinematic capsule, walks it at a fixed wish speed, jumps, and
//! lands back on the floor while reporting per-frame step data. Run with:
//!
//! `cargo run -p scenix --example character_controller --features physics`

use scenix::{
    CharacterInput, CharacterState, ColliderShape, PhysicsBody, PhysicsWorld, Quat, RigidBodyKind,
    Transform, Vec3,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const DT: f32 = 1.0 / 60.0;

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

    // Settle onto the floor.
    let idle = CharacterInput::idle(DT);
    for _ in 0..90 {
        state.move_and_slide(&mut world, &idle);
        world.step(DT);
    }
    assert!(state.ground.landed, "the floor probe catches the character");

    // Walk +X at 3 m/s for half a second.
    let walk = CharacterInput::move_towards(Vec3::X, 3.0, DT);
    let start = world.body(player).unwrap().transform.translation;
    for _ in 0..30 {
        let stride = state.move_and_slide(&mut world, &walk);
        world.step(DT);
        if stride.moved.length() > 0.0 {
            println!(
                "step: moved {:.3} m, grounded={}",
                stride.moved.length(),
                state.ground.landed
            );
        }
    }
    let walked = world.body(player).unwrap().transform.translation.x - start.x;
    println!(
        "walked {:.2} m in 30 frames at 3 m/s (started at x={:.3})",
        walked, start.x
    );
    assert!(
        (walked - 1.5).abs() < 0.6,
        "walking matches the wish speed over dt"
    );

    // Jump with forward keep-drift, then land back on the floor.
    let before = world.body(player).unwrap().transform.translation.y;
    let jump = CharacterInput {
        wish_direction: Vec3::X,
        speed: 0.0,
        jump: true,
        dt: DT,
    };
    state.move_and_slide(&mut world, &jump);
    world.step(DT);
    assert!(!state.ground.landed, "the jump lifts the ground flag");
    for _ in 0..90 {
        state.move_and_slide(&mut world, &idle);
        world.step(DT);
    }
    let after = world.body(player).unwrap().transform.translation.y;
    println!(
        "jump: launch vy=5.0 m/s, back on the floor at y={:.3} (started {:.3})",
        after, before
    );
    assert!(state.ground.landed, "the character lands again");
    Ok(())
}
