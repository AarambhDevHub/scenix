//! Immersive VR viewer planning entirely in deterministic CPU code.
//!
//! Validates a session plan, walks the runtime lifecycle to `Active`, then
//! advances two frame ticks and builds the stereo rig the renderer would
//! stream. Run with:
//!
//! `cargo run -p scenix --example xr_viewer --features xr,camera`

use scenix::{
    Quat, Transform, Vec3, XrEyeProjection, XrFeatures, XrFramePlan, XrReferenceSpace,
    XrSessionPlan, XrSessionState, XrViewport,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Room-scale VR: floor-anchored stage, classic feature set.
    let mut plan = XrSessionPlan::immersive_vr()
        .with_reference_space(XrReferenceSpace::Stage)
        .with_features(XrFeatures::room_scale_vr());
    plan.validate()?;
    for state in [
        XrSessionState::Requesting,
        XrSessionState::Starting,
        XrSessionState::Active,
    ] {
        plan.advance(state)?;
    }
    assert!(plan.is_presenting(), "the viewer is presenting");

    // Headset swapchain mirrored through a 90-degree symmetric eye layout.
    let eye = XrEyeProjection::symmetric(1.5708, 1.0, 0.05, 50.0);
    let head = Transform::new(Vec3::new(0.0, 1.7, 0.0), Quat::IDENTITY, Vec3::ONE);
    let mut frame = XrFramePlan::new(head)
        .with_eye(eye)
        .with_viewport(XrViewport::new(2064, 2208));

    // Two 90 Hz ticks: the head drifts half a meter down the view axis.
    frame = frame.next_frame(
        Transform::new(Vec3::new(0.0, 1.7, -0.6), Quat::IDENTITY, Vec3::ONE),
        1.0 / 90.0,
    );
    frame = frame.next_frame(head, 1.0 / 90.0);
    assert_eq!(frame.stats.frame_index, 2, "two frames planned");
    assert_eq!(
        frame.viewport.stereo_pixel_count(),
        2064_u64 * 2208 * 2,
        "both eye channels stream at native scale"
    );

    // The rig emits the exact two view-projections the renderer consumes.
    let rig = frame.camera_rig();
    let left = rig.left_view();
    let right = rig.right_view();
    println!(
        "xr viewer: mode={:?} space={:?} frames={} left_col0={:?} right_col0={:?}",
        plan.mode, plan.reference_space, frame.stats.frame_index, left.cols[0], right.cols[0],
    );
    Ok(())
}
