#![cfg_attr(not(feature = "std"), no_std)]

//! Animato 1.5 bridge types for scenix.
//!
//! This crate keeps Animato as the timing/interpolation engine and provides
//! scenix-native adapters for scene nodes, cameras, PBR materials, and explicit
//! skeleton pose arrays.

extern crate alloc;

mod camera;
mod driver;
mod material;
mod scene;
mod skeleton;
mod tracks;
mod values;

pub use animato::{Easing, SpringConfig};
pub use camera::{
    CameraAnimationTarget, CameraAnimator, CameraStoreMut, CameraStores, OrthographicBounds,
    OrthographicBoundsTrack,
};
pub use driver::{DriverStats, ScenixAnimationDriver};
pub use material::{MaterialAnimationTarget, MaterialAnimator, PbrMaterialStoreMut};
pub use scene::{NodeAnimationTarget, NodeAnimator};
pub use skeleton::{BoneAnimation, BoneAnimationTarget, SkeletonPose, SkinnedMeshAnimator};
pub use tracks::{BoolTrack, ColorTrack, QuatTrack, ScalarTrack, Vec3Track};
pub use values::{AnimColor, AnimQuat, AnimVec3};
