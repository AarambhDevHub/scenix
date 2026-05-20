#![cfg_attr(not(feature = "std"), no_std)]

//! GPU-free material descriptions for scenix.
//!
//! This crate defines renderer-agnostic materials and compact pipeline keys.
//! It intentionally has no `wgpu` dependency; GPU upload and bind-group logic
//! live in the renderer milestone.

extern crate alloc;

pub mod depth;
pub mod lambert;
pub mod line;
pub mod normal;
pub mod pbr;
pub mod physical;
pub mod points;
pub mod shader;
pub mod toon;
mod traits;
pub mod unlit;
pub mod wireframe;

pub use depth::DepthMaterial;
pub use lambert::LambertMaterial;
pub use line::LineMaterial;
pub use normal::NormalMaterial;
pub use pbr::PbrMaterial;
pub use physical::PhysicalMaterial;
pub use points::PointsMaterial;
pub use shader::ShaderMaterial;
pub use toon::ToonMaterial;
pub use traits::{
    AlphaMode, FEATURE_ALBEDO_TEXTURE, FEATURE_CLEARCOAT, FEATURE_CUSTOM_TEXTURES, FEATURE_DASHED,
    FEATURE_DOUBLE_SIDED, FEATURE_EMISSIVE_TEXTURE, FEATURE_FLAT_SHADING, FEATURE_GRADIENT_TEXTURE,
    FEATURE_IRIDESCENCE, FEATURE_METALLIC_ROUGHNESS_TEXTURE, FEATURE_NORMAL_TEXTURE,
    FEATURE_OCCLUSION_TEXTURE, FEATURE_OUTLINE, FEATURE_SHEEN, FEATURE_SIZE_ATTENUATION,
    FEATURE_TRANSMISSION, FEATURE_VERTEX_COLORS, FEATURE_WIREFRAME, FEATURE_WORLD_SPACE, Material,
    PipelineAlphaMode, PipelineKey, ShaderKind,
};
pub use unlit::UnlitMaterial;
pub use wireframe::WireframeMaterial;
