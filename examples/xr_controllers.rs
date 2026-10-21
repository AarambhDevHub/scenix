//! Controller bay bookkeeping in deterministic CPU code.
//!
//! Updates both controllers with tracked poses, reads pointer rays and
//! predicted poses through the bay, then confirms the handedness checks the
//! facade re-exports. Run with:
//!
//! `cargo run -p scenix --example xr_controllers --features xr`

use scenix::{Quat, Vec3, XrControllerBay, XrHand, XrPose, XrTrackingStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bay = XrControllerBay::new();

    // Right controller points down-range; the left one is only approximately
    // tracked (jittery optical conditions).
    bay.update(
        XrHand::Right,
        XrPose::new(
            Vec3::new(0.25, 1.0, -0.6),
            Quat::from_axis_angle(Vec3::Y, -0.35),
        ),
        XrTrackingStatus::Tracked,
    );
    bay.update(
        XrHand::Left,
        XrPose::new(
            Vec3::new(-0.3, 0.95, -0.5),
            Quat::from_axis_angle(Vec3::Y, 0.5),
        ),
        XrTrackingStatus::Approximate,
    );

    // Give the right hand a 2 m/s push down-range; prediction extrapolates
    // pointer motion from the reported velocity.
    bay.get_mut(XrHand::Right).linear_velocity = Vec3::new(0.0, 0.0, -2.0);

    let right = bay.get(XrHand::Right);
    let left = bay.get(XrHand::Left);
    assert_eq!(right.tracking, XrTrackingStatus::Tracked);
    assert_eq!(left.tracking, XrTrackingStatus::Approximate);
    assert_eq!(right.hand, XrHand::Right);

    // Aim rays and 100 ms arm kinematics prediction.
    let (origin, direction) = bay.aim_ray(XrHand::Right);
    let predicted = right.predicted_pose(0.1);
    let moved = (predicted.position.z - right.pointer.position.z).abs();
    println!(
        "right aim from ({:.2}, {:.2}, {:.2}) toward ({:.2}, {:.2}, {:.2}); \
         100 ms prediction drifts z by {:.3} m",
        origin.x, origin.y, origin.z, direction.x, direction.y, direction.z, moved,
    );
    assert!(moved > 0.0, "prediction must extrapolate velocity");

    // Resetting drops back to the idle bay: untracked hands, origin poses.
    bay.reset();
    assert_eq!(
        bay.get(XrHand::Left).tracking,
        XrTrackingStatus::Untracked,
        "reset empties the bay"
    );
    Ok(())
}
