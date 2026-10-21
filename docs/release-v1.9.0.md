# Scenix v1.9.0 — XR, Audio, And Physics Bridges

Version 1.9 brings simulation-adjacent runtime systems onto the scenix
vocabulary: three optional, `no_std`-capable CPU crates (`scenix-xr`,
`scenix-audio`, `scenix-physics`) that own deterministic CPU rules as the
source of truth, plus CPU-bridge scene nodes, camera stereo math, a WebXR
fallback plan, debug helpers, and facade integration. No engine, OpenXR
runtime, or audio SDK is involved — the host replays the same agreements in
tests, demos, or real runtimes. The renderer pipeline itself is untouched.

## Data Flow

```text
 XrSessionPlan (validate, lifecycle)        AudioListener (ortho axes)
        |                                          |
        v                                          |
 Requesting → Starting → Active                    |
        |                                          v
        +--> XrControllerBay (pointer/grip)   compute_spatial_decision
        |       |  velocity prediction              |  (pan + attenuation
        |       v                                   |   + cone + Doppler)
        |   XrControllerInput (scenix-input)        v
        |                                     SpatialDecision → SpatialMix
        +--> XrHandFrame (26-slot profile)          |   → compute_spatial_gain
        +--> anchors / planes / hit tests     AudioStreamPlan (buffer health)
        +--> XrLightEstimate                  AudioAnalyser (RMS/peak/bands)
        |
        ---- renderer untouched ----          PhysicsWorld::earth (step dt)
        |                                           |
        v                                           v
 XrFramePlan::camera_rig(head)                substeps → bodies, contacts
        |                                           |
        +--> XrCameraRig → left/right VP      cast_ray / overlap_sphere
        +--> wasm xr fallback (emulated rig)        |
        +--> PoseAxesHelper / HandSkeletonHelper    v
                                              PhysicsTransformSync (sorted
                                               body → node-key writes)
                                              ColliderDebugHelper wireframes
```

The CPU crates never touch `wgpu`: session agreements, spatial mix decisions,
and rigid-body states are built and verified on the host before anything
reaches a frame. Bridge scene nodes (`NodeKind::XrAnchor`, `AudioSource`,
`PhysicsBody`) ride the scene schema for visibility and keys, while the
renderer's culling pass matches them in a no-op arm — they submit no draws.

## New Public Surfaces

- `scenix-xr` (new crate): `XrSessionPlan` (`immersive_vr`/`immersive_ar`/
  `inline`, `with_reference_space`/`with_features`/`with_foveation`,
  `validate` with `XrSessionPlanError`, `advance` lifecycle moves, `is_
  presenting`, `camera_rig`), `XrSessionState::can_transition`,
  `XrFeatures` bitsets (`CONTROLLERS`, `HANDS`, `HIT_TEST`, `ANCHORS`,
  `PLANES`, `DEPTH_SENSING`, `LIGHT_ESTIMATION`, `PASSTHROUGH`, with
  `room_scale_vr`/`mobile_ar` bundles and `FEATURE_NAMES`),
  `XrControllerBay`/`XrController` (both-hand pointer/grip tracking with
  velocity-based prediction), `XrHandFrame`/`XrJointPose` over the 26-slot
  hand profile (`JOINT_WRIST`, `JOINT_THUMB_TIP`, `JOINT_INDEX_TIP`,
  `thumb_index_distance`), `XrAnchorStore` (age-pruned), `XrPlaneStore`
  (`reconcile` diffs), `XrHitTestProbe` (floor/wall/window casts),
  `XrLightEstimate`, `XrFramePlan`/`XrFrameStats`/`XrViewport`
  (`XR_MAX_VIEWPORT_SCALE` tensor scaling).
- `scenix-audio` (new crate): `AudioListener::axes`,
  `AudioSource`/`AudioChannel`/`AudioLoop`/`PlaybackState`,
  `compute_spatial_decision` (one `SpatialDecision` folding
  `AttenuationModel::{None, Linear, Inverse, InverseSquare}` pan weights,
  `SpeakerLayout`s, `DirectionalCone` filtering, and `DopplerModel::shift`
  with wave-travel direction and clamped multipliers), `compute_spatial_
  gain`, `blend_pan`, `AudioStreamPlan`
  (`accept_chunk`/`promote_pending`/`consume_chunk`, underrun accounting,
  `buffered_milliseconds`), `AudioAnalyser` (`analyse_mono`/`analyse_stereo`
  → `AnalyserFrame` with `rms`/`peak`/`band` over
  `AnalyserBand::{Bass, Mid, Treble}`), and the bounds
  (`SPEED_OF_SOUND`, `STREAM_MAX_CHUNKS`, `AUDIO_LISTENER_MAX_SOURCES`,
  `ANALYSER_MIN_FFT_SIZE`, `ANALYSER_MAX_FFT_SIZE`).
- `scenix-physics` (new crate): `PhysicsWorld::earth()` with real substeps,
  `add_body`/`add_solid`/`attach_collider` (shapes, groups, sensors,
  densities), deterministic contacts (restitution + Coulomb friction +
  Baumgarte positional push — `CONTACT_RESOLVE_PERCENT` 0.2, `CONTACT_SLOP`
  5 mm), `WorldEvent` contact/sleep streams with a→b normals, spatial
  queries (`cast_ray`, `cast_ray_excluding`, `overlap_sphere`,
  `SpatialQueryHit`), `CharacterState::move_and_slide` (kinematic capsule,
  slope limits, step offsets, jump launch, probe-based ground snapping
  guarded by `PHYSICS_GROUND_REATTACH_SPEED`), `PhysicsTransformSync`/
  `SyncQueue` (sorted body → node-key writes), and `debug_wireframe`/
  `debug_wireframe_world` collider debug geometry.
- `scenix-core`/`scenix-scene`: the `XrSessionId`/`XrAnchorId`/
  `AudioSourceId`/`AudioListenerId`/`PhysicsWorldId`/`PhysicsBodyId`/
  `ColliderId` newtypes and `NodeKind::{XrAnchor, AudioSource, PhysicsBody}`
  riders with inspector labels.
- `scenix-camera`: the stereo surface `XrFov` (four-angle eye frusta),
  `XrEyeProjection` (WebGPU-convention projections plus a `symmetric`
  helper), and `XrCameraRig` (head transform + eye separation → left/right
  view and view-projection matrices).
- `scenix-input`: the shared XR vocabulary `XrHand`, `XrControllerInput`
  (WebXR `xr-standard` trigger/squeeze/thumbstick/trackpad/A-B with the
  `XR_PRESS_THRESHOLD` press gate), and `XrTrackingStatus`.
- `scenix-wasm` (behind `xr`): `webxr_fallback_plan(granted)` →
  `WebXrFallbackPlan::{RuntimeGranted, EmulatedDesktop}` with const
  `immersive`/`emulated_controller` profiles, `emulated_headset_rig`
  (standing desk stereo), and `emulated_controller_pose`.
- `scenix-helpers`: `PoseAxesHelper` (oriented triad + `-Z` forward stick),
  `HandSkeletonHelper` (26-slot wiring through `HAND_LINKS`), and behind
  the `physics` feature `ColliderDebugHelper` (solid-cyan vs sensor-orange
  wireframes via `world_to_geometry`).
- `scenix-math`: additive `floor`/`ceil`/`round` shims so `no_std` substeps
  and viewport math round identically on every target.
- `scenix` facade: the `xr`, `audio`, `physics`, and `bridges` features
  (`bridges` enables all three). Bridge re-exports are curated lists because
  their module namespaces (`xr`, `audio`, `physics`, `world`, `sync`) would
  otherwise shadow the scene/input glob re-exports.

## First Bridge Setup

```rust
use scenix::{
    AttenuationModel, AudioListener, AudioStreamPlan, AudioSource, CharacterInput,
    CharacterState, ColliderShape, DopplerModel, PhysicsBody, PhysicsWorld, PlaybackState, Quat,
    RigidBodyKind, StreamFormat, Transform, Vec3, XrControllerBay, XrFeatures, XrHand, XrPose,
    XrReferenceSpace, XrSessionPlan, XrSessionState, XrTrackingStatus,
};

// 1. XR: validate and present a room-scale VR plan.
let mut xr = XrSessionPlan::immersive_vr()
    .with_reference_space(XrReferenceSpace::Stage)
    .with_features(XrFeatures::room_scale_vr());
xr.validate()?;
for state in [
    XrSessionState::Requesting,
    XrSessionState::Starting,
    XrSessionState::Active,
] {
    xr.advance(state)?;
}

// Controller bays track pointer/grip poses; prediction extrapolates them.
let mut bay = XrControllerBay::new();
bay.update(
    XrHand::Right,
    XrPose::from_position(Vec3::new(0.25, 1.0, -0.6)),
    XrTrackingStatus::Tracked,
);

// 2. Audio: face a listener down-range and let the spatial mix decide.
let listener = AudioListener::axes(Vec3::new(0.0, 1.5, 2.0), Vec3::NEG_Z, Vec3::Y);
let mut source = AudioSource::new(1, PlaybackState::Playing);
source.position = Vec3::new(-1.0, 1.0, -1.0);
let decision = scenix::compute_spatial_decision(
    &listener,
    &source,
    AttenuationModel::Inverse,
    1.0,
    &DopplerModel::default(),
);
assert!(decision.audible);

// 3. Physics: walk a capsule across a floor.
let mut world = PhysicsWorld::earth();
world.add_solid(
    PhysicsBody::new(Transform::IDENTITY).with_kind(RigidBodyKind::Static),
    ColliderShape::boxy(20.0, 0.5, 20.0),
);
let player = world.add_body(
    PhysicsBody::new(Transform::new(Vec3::new(0.0, 1.4, 0.0), Quat::IDENTITY, Vec3::ONE))
        .with_kind(RigidBodyKind::KinematicPosition),
);
world.attach_collider(player, ColliderShape::capsule(0.35, 0.55));
let mut controller = CharacterState::new(player, 0.35, 0.55);
controller.move_and_slide(&mut world, &CharacterInput::move_towards(Vec3::X, 3.0, 1.0 / 60.0));

// 4. Streamed audio health stays deterministic for music-reactive scenes.
let mut plan = AudioStreamPlan::new(4, StreamFormat::PcmF32, 48_000, 2);
plan.accept_chunk();
assert!(plan.promote_pending());
```

Feature gates: `xr`, `audio`, and `physics` are independent facade features;
`bridges` enables all three at once; `scenix-wasm`'s `xr` feature carries the
browser fallback; `scenix-helpers`' `physics` feature carries the collider
wireframe helper.

## Migration

No v1.8 API changed. `NodeKind` gained the additive `XrAnchor`, `AudioSource`,
and `PhysicsBody` variants, which scene traversal and editor code already
match exhaustively in their own fallback arms. Default facade features are
unchanged, so existing apps compile identically. The bridge re-exports are
curated lists (not globs) because their module namespaces collide with the
scene/input globs — import the same names as before, they simply route
through the typed lists. `scenix-math`'s rounding shims are additive.

## Validation

The release is covered by the `scenix-xr`/`scenix-audio`/`scenix-physics`
crate suites (`tests/xr_cpu.rs`, `tests/audio_cpu.rs`, `tests/physics_cpu.rs`:
session validation and the total lifecycle table, controller bays and
kinematic prediction, 26-slot hand frames, anchor pruning, plane
reconciliation, hit tests, viewport math; spatial decisions with cone
filtering and clamped Doppler, stream buffering health, analyser bands,
no_std lanes; determinism against body ordering, restitution/friction
equilibrium, sensors and group filtering, ray/overlap queries, sleeping,
character walk/jump/land, and respawn), the facade `scenix_v19` suite, and
the `xr_viewer`/`xr_controllers`/`audio_positions`/`physics_sync`/
`character_controller` examples. CI runs the facade feature lanes
(`--no-default-features --features xr|audio|physics|bridges` with and
without `scene`/`camera`/`helpers`), no-default CPU builds, new-crate
`--all-features` tests, rustdoc warnings-as-errors, and packaging.

Related reading:

- [scenix-xr crate reference](api/scenix-xr.md)
- [scenix-audio crate reference](api/scenix-audio.md)
- [scenix-physics crate reference](api/scenix-physics.md)
- [Examples](examples/README.md)
- [GitHub release notes](../.github/release-notes/1.9.0.md)
