//! CPU-side XR session planning for scenix.
//!
//! This crate is the `v1.9.0` XR bridge: session plans that negotiate
//! Immersive AR/VR features against WebXR and OpenXR runtimes, tracked
//! controller poses, hand skeletons, hit-test probes, spatial anchors and
//! detection planes, and estimated lighting that host applications can
//! apply to scene lights. It is **GPU-free** — no renderer code lives
//! here; stereo eye views come from `scenix-camera`'s `XrCameraRig`.
//!
//! # Quick start
//!
//! ```rust
//! use scenix_xr::{XrFeatures, XrReferenceSpace, XrSessionPlan, XrSessionState};
//!
//! let mut plan = XrSessionPlan::immersive_ar()
//!     .with_reference_space(XrReferenceSpace::BoundedFloor)
//!     .with_features(XrFeatures::mobile_ar());
//! assert!(plan.features.contains(XrFeatures::HIT_TEST | XrFeatures::ANCHORS));
//! for state in [
//!     XrSessionState::Requesting,
//!     XrSessionState::Starting,
//!     XrSessionState::Active,
//! ] {
//!     plan.advance(state).expect("legal lifecycle transition");
//! }
//! assert!(plan.is_presenting());
//! ```
//!
//! # House rules
//!
//! * Session planning is deterministic CPU state: the same sequence of
//!   inputs produces the same session state and the same feature snapshot.
//! * Validation is total: every builder clamps or rejects invalid
//!   channel/feature combinations, so applications cannot construct a plan
//!   that an XR runtime would reject at request time.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod anchors;
mod controllers;
mod frames;
mod hands;
mod hit_test;
mod lighting;
mod session;

pub use anchors::{XR_ANCHOR_MAX_AGE, XrAnchor, XrAnchorStore, XrPlane, XrPlaneKind, XrPlaneStore};
pub use controllers::{XrController, XrControllerBay, XrPose, XrSpace};
pub use frames::{XR_MAX_VIEWPORT_SCALE, XrFramePlan, XrFrameStats, XrViewport};
pub use hands::{
    JOINT_INDEX_TIP, JOINT_THUMB_TIP, JOINT_WRIST, XR_HAND_JOINT_COUNT, XrHandFrame, XrJointPose,
};
pub use hit_test::{XR_HIT_MAX_RESULTS, XrHit, XrHitSurface, XrHitTestProbe};
pub use lighting::{XrAmbientEstimate, XrLightEstimate, XrLightEstimationMode};
pub use session::{
    FEATURE_NAMES, XR_FEATURE_COUNT, XrFeatureIter, XrFeatures, XrFoveation, XrReferenceSpace,
    XrSessionMode, XrSessionPlan, XrSessionPlanError, XrSessionState,
};

/// Crate version reported for XR tooling and the wasm inspector.
pub const XR_VERSION: &str = "1.10.0";

/// Convenience alias pairing a controller with its current input snapshot.
///
/// This re-uses `scenix-input`'s vocabulary without re-exporting it: when
/// both crates are in scope, `XrHand` resolves to one definition.
pub type XrControllerSnapshot = scenix_input::XrControllerInput;
