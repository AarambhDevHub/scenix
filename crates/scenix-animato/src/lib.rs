#![cfg_attr(not(feature = "std"), no_std)]

//! Animato bridge types for scenix.
//!
//! This crate keeps Animato as the timing/interpolation engine and provides
//! scenix-native adapters for scene nodes, cameras, PBR materials, lights,
//! morph weights, and explicit skeleton pose arrays.
//!
//! Scenix v1.4.0 adds a clip-based animation runtime (`AnimationClip`,
//! `AnimationAction`, `AnimationMixer`) on top of the procedural Animato
//! tween/spring tracks, plus property bindings, loop modes, markers/events,
//! crossfade, additive blending, retargeting, light/morph targets, and
//! deterministic sampling.
//!
//! Animato 1.7.0 is the release target; the scenix bridge uses the stable
//! `std`, `tween`, `spring`, and `serde` feature set.

extern crate alloc;

mod action;
mod binding;
mod camera;
mod clip;
mod driver;
mod events;
#[cfg(feature = "inspector")]
mod inspector;
mod keyframe;
mod light;
mod loop_mode;
mod material;
mod mixer;
mod morph;
mod retarget;
mod scene;
mod skeleton;
mod tracks;
mod values;

pub use action::{ActionHandle, ActionState, AnimationAction, BlendMode};
pub use animato::{Easing, SpringConfig};
pub use binding::{
    BindingKey, BoneProperty, CameraProperty, LightProperty, MaterialProperty, NodeProperty,
    PropertyBinding,
};
pub use camera::{
    CameraAnimationTarget, CameraAnimator, CameraStoreMut, CameraStores, OrthographicBounds,
    OrthographicBoundsTrack,
};
pub use clip::{AnimationClip, AnimationMarker, ClipChannel, ClipTrack};
pub use driver::{DriverStats, ScenixAnimationDriver};
pub use events::{AnimationEvent, MixerTickResult};
pub use keyframe::{
    KeyframeBool, KeyframeColor, KeyframeInterpolation, KeyframeQuat, KeyframeScalar, KeyframeVec3,
};
pub use light::{LightAnimationTarget, LightAnimator, LightStoreMut, LightStores};
pub use loop_mode::{LoopAdvance, LoopMode};
pub use material::{MaterialAnimationTarget, MaterialAnimator, PbrMaterialStoreMut};
pub use mixer::AnimationMixer;
pub use morph::{MorphWeightAnimator, MorphWeightStoreMut};
pub use retarget::{RetargetEntry, RetargetMap};
pub use scene::{NodeAnimationTarget, NodeAnimator};
pub use skeleton::{BoneAnimation, BoneAnimationTarget, SkeletonPose, SkinnedMeshAnimator};
pub use tracks::{BoolTrack, ColorTrack, QuatTrack, ScalarTrack, Vec3Track};
pub use values::{AnimColor, AnimQuat, AnimVec3};
