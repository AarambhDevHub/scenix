use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use scenix_material::{PipelineAlphaMode, PipelineKey, ShaderKind};

/// Render pass family for pipeline selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RenderPassKind {
    /// Opaque G-buffer geometry pass.
    Geometry,
    /// Deferred lighting resolve pass.
    Lighting,
    /// Forward transparent/unlit pass.
    Forward,
    /// Depth-only shadow pass.
    Shadow,
}

/// Full renderer pipeline cache key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RendererPipelineKey {
    /// Material-level pipeline key.
    pub material: PipelineKey,
    /// Pass family.
    pub pass: RenderPassKind,
    /// Color target format.
    pub color_format: wgpu::TextureFormat,
    /// MSAA sample count.
    pub sample_count: u32,
    /// Whether depth testing is enabled.
    pub depth: bool,
}

impl RendererPipelineKey {
    /// Creates a renderer pipeline key.
    #[inline]
    pub const fn new(
        material: PipelineKey,
        pass: RenderPassKind,
        color_format: wgpu::TextureFormat,
        sample_count: u32,
        depth: bool,
    ) -> Self {
        Self {
            material,
            pass,
            color_format,
            sample_count,
            depth,
        }
    }
}

/// Lazy render-pipeline cache keyed by renderer pipeline state.
#[derive(Default)]
pub struct PipelineCache {
    pipelines: HashMap<RendererPipelineKey, Arc<wgpu::RenderPipeline>>,
}

impl PipelineCache {
    /// Creates an empty pipeline cache.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of cached pipelines.
    #[inline]
    pub fn len(&self) -> usize {
        self.pipelines.len()
    }

    /// Returns whether the cache is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pipelines.is_empty()
    }

    /// Gets an existing pipeline or lazily creates one.
    pub fn get_or_create(
        &mut self,
        device: &wgpu::Device,
        key: RendererPipelineKey,
    ) -> Arc<wgpu::RenderPipeline> {
        if let Some(pipeline) = self.pipelines.get(&key) {
            return Arc::clone(pipeline);
        }

        let pipeline = Arc::new(create_pipeline(device, key));
        self.pipelines.insert(key, Arc::clone(&pipeline));
        pipeline
    }
}

fn create_pipeline(device: &wgpu::Device, key: RendererPipelineKey) -> wgpu::RenderPipeline {
    let (vertex_source, fragment_source) = shader_sources(key);
    let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("scenix.pipeline.vertex"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(vertex_source)),
    });
    let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("scenix.pipeline.fragment"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(fragment_source)),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("scenix.pipeline.layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    let blend = match key.material.alpha {
        PipelineAlphaMode::Blend => Some(wgpu::BlendState::ALPHA_BLENDING),
        PipelineAlphaMode::Opaque | PipelineAlphaMode::Mask => Some(wgpu::BlendState::REPLACE),
    };

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("scenix.pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &vertex,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: key.sample_count,
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: key.color_format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn shader_sources(key: RendererPipelineKey) -> (&'static str, &'static str) {
    match key.pass {
        RenderPassKind::Shadow => (
            include_str!("shaders/shadow_depth.vert.wgsl"),
            include_str!("shaders/pbr.frag.wgsl"),
        ),
        RenderPassKind::Lighting => (
            include_str!("shaders/deferred_resolve.wgsl"),
            include_str!("shaders/deferred_resolve.wgsl"),
        ),
        RenderPassKind::Geometry | RenderPassKind::Forward => match key.material.shader {
            ShaderKind::Unlit => (
                include_str!("shaders/pbr.vert.wgsl"),
                include_str!("shaders/unlit.frag.wgsl"),
            ),
            ShaderKind::Pbr | ShaderKind::Lambert => (
                include_str!("shaders/pbr.vert.wgsl"),
                include_str!("shaders/pbr.frag.wgsl"),
            ),
            _ => (
                include_str!("shaders/pbr.vert.wgsl"),
                include_str!("shaders/unlit.frag.wgsl"),
            ),
        },
    }
}
