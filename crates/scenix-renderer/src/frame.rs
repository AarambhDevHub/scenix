use scenix_math::{Mat4, Vec2, Vec3};

use crate::RenderTargetMode;

/// Per-frame camera and target data uploaded before render passes.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameContext {
    /// Monotonic frame index.
    pub frame_index: u64,
    /// Render target resolution in pixels.
    pub resolution: Vec2,
    /// Camera view matrix.
    pub view: Mat4,
    /// Camera projection matrix.
    pub projection: Mat4,
    /// Projection multiplied by view.
    pub view_projection: Mat4,
    /// Camera position in world space.
    pub camera_position: Vec3,
}

/// CPU-side counters reported after a rendered frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameStats {
    /// Monotonic frame index.
    pub frame_index: u64,
    /// Number of mesh nodes seen during traversal.
    pub scene_meshes: u32,
    /// Number of mesh nodes submitted after culling.
    pub visible_meshes: u32,
    /// Number of mesh nodes rejected by frustum culling.
    pub culled_meshes: u32,
    /// Opaque draw submissions.
    pub opaque_draws: u32,
    /// Transparent draw submissions.
    pub transparent_draws: u32,
    /// Registered lights available to the frame.
    pub lights: u32,
    /// Render target backing used for this frame.
    pub target_mode: Option<RenderTargetMode>,
}

/// Renderer-wide resource and frame diagnostics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RendererDiagnostics {
    /// Monotonic frame index that will be used by the next render call.
    pub frame_index: u64,
    /// Registered mesh count.
    pub meshes: u32,
    /// Registered material count.
    pub materials: u32,
    /// Registered texture count.
    pub textures: u32,
    /// Registered light count.
    pub lights: u32,
    /// Registered render target count.
    pub render_targets: u32,
    /// Approximate GPU texture bytes owned by renderer uploads.
    pub texture_memory_bytes: u64,
    /// Approximate GPU vertex and index buffer bytes.
    pub geometry_memory_bytes: u64,
    /// Approximate uniform-buffer bytes allocated for draw resources.
    pub uniform_memory_bytes: u64,
    /// Cached render pipeline count.
    pub pipeline_cache_entries: u32,
    /// Shadow atlas layers available to the renderer.
    pub shadow_slots: u32,
}

/// Snapshot of renderer-owned resource memory.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceStats {
    /// Approximate mesh buffer bytes.
    pub geometry_bytes: u64,
    /// Approximate texture and render-target bytes.
    pub texture_bytes: u64,
    /// Approximate uniform-buffer bytes.
    pub uniform_bytes: u64,
}

/// Pipeline cache counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PipelineCacheStats {
    /// Number of cached render pipelines.
    pub entries: u32,
}
