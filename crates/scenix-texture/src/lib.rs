#![cfg_attr(not(feature = "std"), no_std)]

//! CPU-side texture data and sampling metadata for scenix.
//!
//! This crate stores raw texture bytes, sampler settings, simple atlas packing,
//! and CPU mipmap generation. It does not decode image files and does not
//! allocate GPU resources.

extern crate alloc;

pub mod atlas;
pub mod format;
pub mod mipmap;
pub mod sampler;
pub mod texture;
pub mod video;

pub use atlas::{AtlasEntry, AtlasRect, TextureAtlas, UvRect};
pub use format::TextureFormat;
pub use sampler::{AddressMode, CompareFunction, FilterMode, Sampler};
pub use texture::{Texture2D, Texture3D, TextureCube};
pub use video::VideoTexture;
