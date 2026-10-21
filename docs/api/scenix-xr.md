# `scenix-xr`

## Role

Deterministic CPU source of truth for XR runtime agreements: session plans
and lifecycle, controller bays, 26-slot hand frames, anchors, planes, hit
testing, light estimation, and per-frame stereo stats (v1.9.0).

## Dependency Weight

Lightweight `no_std` on `scenix-core`, `scenix-math`, `scenix-input`, and
`scenix-camera`.

## Install

```toml
[dependencies]
scenix-xr = "1"
```

## Key Public API

`XrSessionPlan`, `XrSessionState`, `XrSessionPlanError`, `XrFeatures`,
`XrReferenceSpace`, `XrSessionMode`, `XrFoveation`, `FEATURE_NAMES`,
`XrController`, `XrControllerBay`, `XrHandFrame`, `XrJointPose`,
`XR_HAND_JOINT_COUNT`, `JOINT_WRIST`, `JOINT_THUMB_TIP`, `JOINT_INDEX_TIP`,
`XrAnchor`, `XrAnchorStore`, `XR_ANCHOR_MAX_AGE`, `XrPlane`, `XrPlaneStore`,
`XrHitTestProbe`, `XrLightEstimate`, `XrFramePlan`, `XrFrameStats`,
`XrViewport`, `XR_MAX_VIEWPORT_SCALE`, `XR_VERSION`, `XrPose`, `XrSpace`

## Common Use

```rust
use scenix_xr::{
    XrFeatures, XrReferenceSpace, XrSessionPlan, XrSessionState,
};
let mut plan = XrSessionPlan::immersive_vr()
    .with_reference_space(XrReferenceSpace::Stage)
    .with_features(XrFeatures::room_scale_vr());
plan.validate()?;
for state in [XrSessionState::Requesting, XrSessionState::Starting, XrSessionState::Active] {
    plan.advance(state)?;
}
# fn main() -> Result<(), Box<dyn std::error::Error>> { let mut plan = XrSessionPlan::immersive_vr().with_reference_space(XrReferenceSpace::Stage).with_features(XrFeatures::room_scale_vr()); plan.validate()?; for state in [XrSessionState::Requesting, XrSessionState::Starting, XrSessionState::Active] { plan.advance(state)?; } Ok(()) }
```

## Notes

v1.9.0 adds this crate. No engine or SDK dependency: the crate models what a
runtime would grant, so hosts replay identical session agreements in tests.
`scenix-input` owns the shared `XrHand`/`XrTrackingStatus`/
`XrControllerInput` vocabulary (the facade reaches it through its input
glob), and `scenix-wasm`'s `xr` feature consumes the plan types for its
browser fallback. `XrFramePlan::camera_rig` hands the head transform to
`scenix-camera`'s `XrCameraRig` for stereo view-projections.

## Related Docs

- [Crate dependency map](../reference/crate-dependency-map.md)
- [Scenix v1.9.0 release](../release-v1.9.0.md)
