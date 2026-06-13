//! wgpu renderer, GPU scene upload, passes, and frame orchestration for scenix.
//!
//! This crate is intentionally the first GPU-dependent layer in scenix. CPU-side
//! crates keep owning authoring data; this crate owns upload, render-target
//! allocation, render-pass scheduling, and pipeline caching.

pub mod config;
pub mod environment;
pub mod frame;
pub mod gbuffer;
pub mod gpu_scene;
pub mod material;
pub mod pass;
pub mod pipeline_cache;
pub mod renderer;
mod shadow;

pub use config::{RenderTargetMode, RendererConfig};
pub use environment::EnvironmentMap;
pub use frame::{FrameContext, FrameStats, PipelineCacheStats, RendererDiagnostics, ResourceStats};
pub use gbuffer::{GBuffer, RenderTargetDescriptor, RenderTargetKind};
pub use gpu_scene::{
    DrawSubmission, GpuIndexFormat, GpuMesh, GpuScene, GpuTexture, PackedGeometry, PackedVertex,
    RendererLight, RendererMaterial, TextureStore, to_wgpu_address_mode, to_wgpu_compare,
    to_wgpu_filter_mode, to_wgpu_texture_format,
};
pub use material::{GpuMaterial, MaterialUniform};
pub use pass::culling::{CullingStats, collect_visible_draws};
pub use pass::sort::{sort_opaque_front_to_back, sort_transparent_back_to_front};
pub use pipeline_cache::{PipelineCache, RenderPassKind, RendererPipelineKey};
pub use renderer::Renderer;
pub use shadow::ShadowMapAtlas;

pub use wgpu;
