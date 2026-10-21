//! Physics-to-scene synchronization in deterministic CPU code.
//!
//! Drops a ball over a static floor, steps to rest, snapshots the transform
//! sync queue the scene would consume, and produces the collider debug
//! wireframe for the renderer's debug pass. Run with:
//!
//! `cargo run -p scenix --example physics_sync --features physics,helpers`

use scenix::{
    ColliderDebugHelper, ColliderShape, PhysicsBody, PhysicsTransformSync, PhysicsWorld, Quat,
    RigidBodyKind, Transform, Vec3,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = PhysicsWorld::earth();

    // Playground floor plus one loose ball, one meter up, drifting left.
    let floor = world.add_solid(
        PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
        ColliderShape::boxy(8.0, 0.5, 8.0),
    );
    let ball = world.add_body(
        PhysicsBody::new(Transform::new(
            Vec3::new(0.3, 2.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
        ))
        .with_kind(RigidBodyKind::Dynamic)
        .with_mass(1.0)
        .with_surface(0.3, 0.4),
    );
    let _ball_collider = world.attach_collider(ball, ColliderShape::sphere(0.5));

    // Scene bindings: body ids know which node keys they drive.
    let mut sync = PhysicsTransformSync::new();
    assert!(sync.bind(floor, 0x2), "floor node key 0x2");
    assert!(sync.bind(ball, 0x1), "ball node key 0x1");

    // One second at 60 Hz; the ball settles on the floor plane.
    for _ in 0..60 {
        world.step(1.0 / 60.0);
    }
    let events = world.take_events();
    println!("events: {} contact transitions", events.len());

    let queue = sync.collect(&world);
    assert_eq!(queue.len(), 1, "statics stay off the write list");
    let entry = &queue.entries()[0];
    assert_eq!(entry.body, ball);
    println!(
        "sync: ball node 0x{:x} now at ({:.3}, {:.3}, {:.3})",
        entry.layer,
        entry.transform.translation.x,
        entry.transform.translation.y,
        entry.transform.translation.z,
    );
    assert!(
        entry.transform.translation.y < 1.0,
        "the ball came to rest near the floor"
    );

    // Debug wireframe for every collider, sensors colored apart.
    let wireframe = ColliderDebugHelper::new().world_to_geometry(&world);
    println!("debug wireframe: {} segments", wireframe.segment_count());
    assert!(wireframe.segment_count() > 0, "both colliders wireframe");
    Ok(())
}
