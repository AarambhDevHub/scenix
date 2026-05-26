//! GPU post-processing stack for scenix.
//!
//! `scenix-post` owns full-screen wgpu passes and does not depend on the
//! renderer crate. Renderers can feed it a source texture view and final target
//! view while avoiding a Cargo dependency cycle.

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use scenix_core::{ScenixError, ValidationError};
use scenix_math::Vec2;

/// Per-frame post-processing context.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PostContext {
    /// Monotonic frame index.
    pub frame_index: u64,
    /// Target resolution in pixels.
    pub resolution: Vec2,
    /// Output color format.
    pub color_format: wgpu::TextureFormat,
}

/// CPU counters for a post-processing dispatch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PostStats {
    /// Number of enabled post passes submitted.
    pub passes: u32,
    /// Number of scratch targets resized during this dispatch.
    pub resized_targets: u32,
}

/// Bloom highlight extraction and additive glow configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BloomConfig {
    /// Luminance threshold where bloom begins.
    pub threshold: f32,
    /// Bloom contribution multiplier.
    pub intensity: f32,
    /// Approximate blur radius in pixels.
    pub radius: f32,
}

impl BloomConfig {
    /// Returns a clamped configuration.
    pub fn normalized(self) -> Self {
        Self {
            threshold: self.threshold.clamp(0.0, 16.0),
            intensity: self.intensity.clamp(0.0, 16.0),
            radius: self.radius.clamp(0.0, 64.0),
        }
    }
}

impl Default for BloomConfig {
    #[inline]
    fn default() -> Self {
        Self {
            threshold: 1.0,
            intensity: 0.35,
            radius: 4.0,
        }
    }
}

/// Screen-space ambient occlusion post configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaoConfig {
    /// Sampling radius in view-space units.
    pub radius: f32,
    /// Occlusion strength.
    pub intensity: f32,
    /// Self-occlusion bias.
    pub bias: f32,
}

impl SsaoConfig {
    /// Returns a clamped configuration.
    pub fn normalized(self) -> Self {
        Self {
            radius: self.radius.clamp(0.0, 16.0),
            intensity: self.intensity.clamp(0.0, 4.0),
            bias: self.bias.clamp(0.0, 1.0),
        }
    }
}

impl Default for SsaoConfig {
    #[inline]
    fn default() -> Self {
        Self {
            radius: 0.5,
            intensity: 1.0,
            bias: 0.025,
        }
    }
}

/// Tone mapping operator.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ToneMapper {
    /// Leaves color unchanged.
    None,
    /// Reinhard tone mapping.
    Reinhard,
    /// ACES-inspired filmic curve.
    #[default]
    Aces,
    /// Exponential exposure curve.
    Exposure(f32),
}

/// Fast approximate anti-aliasing configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FxaaConfig {
    /// Minimum contrast that triggers smoothing.
    pub contrast_threshold: f32,
    /// Relative contrast threshold.
    pub relative_threshold: f32,
}

impl FxaaConfig {
    /// Returns a clamped configuration.
    pub fn normalized(self) -> Self {
        Self {
            contrast_threshold: self.contrast_threshold.clamp(0.0, 1.0),
            relative_threshold: self.relative_threshold.clamp(0.0, 1.0),
        }
    }
}

impl Default for FxaaConfig {
    #[inline]
    fn default() -> Self {
        Self {
            contrast_threshold: 0.0312,
            relative_threshold: 0.125,
        }
    }
}

/// Temporal anti-aliasing blend configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaaConfig {
    /// History feedback in `0..=1`.
    pub feedback: f32,
    /// Jitter amount in pixels.
    pub jitter: f32,
}

impl TaaConfig {
    /// Returns a clamped configuration.
    pub fn normalized(self) -> Self {
        Self {
            feedback: self.feedback.clamp(0.0, 1.0),
            jitter: self.jitter.clamp(0.0, 2.0),
        }
    }
}

impl Default for TaaConfig {
    #[inline]
    fn default() -> Self {
        Self {
            feedback: 0.9,
            jitter: 0.5,
        }
    }
}

/// SMAA quality preset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SmaaQuality {
    /// Low quality.
    Low,
    /// Balanced quality.
    #[default]
    Medium,
    /// High quality.
    High,
    /// Ultra quality.
    Ultra,
}

/// SMAA configuration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmaaConfig {
    /// Quality preset.
    pub quality: SmaaQuality,
}

/// Depth-of-field post configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DofConfig {
    /// Focus distance in scene units.
    pub focus_distance: f32,
    /// Aperture strength.
    pub aperture: f32,
    /// Maximum blur radius in pixels.
    pub max_blur_radius: f32,
}

impl DofConfig {
    /// Returns a clamped configuration.
    pub fn normalized(self) -> Self {
        Self {
            focus_distance: self.focus_distance.max(0.001),
            aperture: self.aperture.clamp(0.0, 32.0),
            max_blur_radius: self.max_blur_radius.clamp(0.0, 64.0),
        }
    }
}

impl Default for DofConfig {
    #[inline]
    fn default() -> Self {
        Self {
            focus_distance: 10.0,
            aperture: 1.4,
            max_blur_radius: 8.0,
        }
    }
}

/// Fog blend post configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FogPostConfig {
    /// Fog color in linear RGB.
    pub color: [f32; 3],
    /// Fog density.
    pub density: f32,
}

impl FogPostConfig {
    /// Returns a clamped configuration.
    pub fn normalized(self) -> Self {
        Self {
            color: [
                self.color[0].clamp(0.0, 1.0),
                self.color[1].clamp(0.0, 1.0),
                self.color[2].clamp(0.0, 1.0),
            ],
            density: self.density.clamp(0.0, 1.0),
        }
    }
}

impl Default for FogPostConfig {
    #[inline]
    fn default() -> Self {
        Self {
            color: [0.5, 0.6, 0.7],
            density: 0.05,
        }
    }
}

/// Edge outline post configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutlineConfig {
    /// Outline color in linear RGBA.
    pub color: [f32; 4],
    /// Luminance edge threshold.
    pub threshold: f32,
    /// Edge sampling distance in pixels.
    pub thickness: f32,
}

impl OutlineConfig {
    /// Returns a clamped configuration.
    pub fn normalized(self) -> Self {
        Self {
            color: [
                self.color[0].clamp(0.0, 1.0),
                self.color[1].clamp(0.0, 1.0),
                self.color[2].clamp(0.0, 1.0),
                self.color[3].clamp(0.0, 1.0),
            ],
            threshold: self.threshold.clamp(0.0, 1.0),
            thickness: self.thickness.clamp(0.0, 16.0),
        }
    }
}

impl Default for OutlineConfig {
    #[inline]
    fn default() -> Self {
        Self {
            color: [0.0, 0.0, 0.0, 1.0],
            threshold: 0.1,
            thickness: 1.0,
        }
    }
}

/// Camera motion blur post configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MotionBlurConfig {
    /// Blur strength.
    pub strength: f32,
    /// Number of conceptual samples. The v0.7 shader maps this to a compact fixed pattern.
    pub sample_count: u32,
}

impl MotionBlurConfig {
    /// Returns a clamped configuration.
    pub fn normalized(self) -> Self {
        Self {
            strength: self.strength.clamp(0.0, 1.0),
            sample_count: self.sample_count.clamp(1, 32),
        }
    }
}

impl Default for MotionBlurConfig {
    #[inline]
    fn default() -> Self {
        Self {
            strength: 0.08,
            sample_count: 8,
        }
    }
}

/// One post-processing effect in stack order.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PostEffect {
    /// Bloom pass.
    Bloom(BloomConfig),
    /// SSAO approximation pass.
    Ssao(SsaoConfig),
    /// Tonemapping pass.
    Tonemap(ToneMapper),
    /// FXAA pass.
    Fxaa(FxaaConfig),
    /// TAA blend pass.
    Taa(TaaConfig),
    /// SMAA pass.
    Smaa(SmaaConfig),
    /// Depth-of-field pass.
    Dof(DofConfig),
    /// Fog blend pass.
    Fog(FogPostConfig),
    /// Outline edge pass.
    Outline(OutlineConfig),
    /// Motion blur pass.
    MotionBlur(MotionBlurConfig),
}

impl PostEffect {
    /// Returns a stable numeric kind used by the WGSL shader.
    #[inline]
    pub const fn kind_id(&self) -> u32 {
        match self {
            Self::Bloom(_) => 1,
            Self::Ssao(_) => 2,
            Self::Tonemap(_) => 3,
            Self::Fxaa(_) => 4,
            Self::Taa(_) => 5,
            Self::Smaa(_) => 6,
            Self::Dof(_) => 7,
            Self::Fog(_) => 8,
            Self::Outline(_) => 9,
            Self::MotionBlur(_) => 10,
        }
    }

    fn params(&self) -> [f32; 8] {
        match *self {
            Self::Bloom(config) => {
                let config = config.normalized();
                [
                    config.threshold,
                    config.intensity,
                    config.radius,
                    0.0,
                    self.kind_id() as f32,
                    0.0,
                    0.0,
                    0.0,
                ]
            }
            Self::Ssao(config) => {
                let config = config.normalized();
                [
                    config.radius,
                    config.intensity,
                    config.bias,
                    0.0,
                    self.kind_id() as f32,
                    0.0,
                    0.0,
                    0.0,
                ]
            }
            Self::Tonemap(mapper) => {
                let (mode, exposure) = match mapper {
                    ToneMapper::None => (0.0, 1.0),
                    ToneMapper::Reinhard => (1.0, 1.0),
                    ToneMapper::Aces => (2.0, 1.0),
                    ToneMapper::Exposure(exposure) => (3.0, exposure.max(0.0)),
                };
                [
                    mode,
                    exposure,
                    0.0,
                    0.0,
                    self.kind_id() as f32,
                    0.0,
                    0.0,
                    0.0,
                ]
            }
            Self::Fxaa(config) => {
                let config = config.normalized();
                [
                    config.contrast_threshold,
                    config.relative_threshold,
                    0.0,
                    0.0,
                    self.kind_id() as f32,
                    0.0,
                    0.0,
                    0.0,
                ]
            }
            Self::Taa(config) => {
                let config = config.normalized();
                [
                    config.feedback,
                    config.jitter,
                    0.0,
                    0.0,
                    self.kind_id() as f32,
                    0.0,
                    0.0,
                    0.0,
                ]
            }
            Self::Smaa(config) => {
                let quality = match config.quality {
                    SmaaQuality::Low => 0.25,
                    SmaaQuality::Medium => 0.5,
                    SmaaQuality::High => 0.75,
                    SmaaQuality::Ultra => 1.0,
                };
                [quality, 0.0, 0.0, 0.0, self.kind_id() as f32, 0.0, 0.0, 0.0]
            }
            Self::Dof(config) => {
                let config = config.normalized();
                [
                    config.focus_distance,
                    config.aperture,
                    config.max_blur_radius,
                    0.0,
                    self.kind_id() as f32,
                    0.0,
                    0.0,
                    0.0,
                ]
            }
            Self::Fog(config) => {
                let config = config.normalized();
                [
                    config.color[0],
                    config.color[1],
                    config.color[2],
                    config.density,
                    self.kind_id() as f32,
                    0.0,
                    0.0,
                    0.0,
                ]
            }
            Self::Outline(config) => {
                let config = config.normalized();
                [
                    config.color[0],
                    config.color[1],
                    config.threshold,
                    config.thickness,
                    self.kind_id() as f32,
                    config.color[2],
                    config.color[3],
                    0.0,
                ]
            }
            Self::MotionBlur(config) => {
                let config = config.normalized();
                [
                    config.strength,
                    config.sample_count as f32,
                    0.0,
                    0.0,
                    self.kind_id() as f32,
                    0.0,
                    0.0,
                    0.0,
                ]
            }
        }
    }
}

/// Renderer-owned post-processing texture target.
pub struct PostTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl PostTarget {
    /// Allocates a texture target suitable for post-processing.
    pub fn new(
        device: &wgpu::Device,
        label: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, ScenixError> {
        if width == 0 || height == 0 {
            return Err(ScenixError::Validation(ValidationError::OutOfRange));
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Ok(Self {
            texture,
            view,
            width,
            height,
            format,
        })
    }

    /// Returns the texture view.
    #[inline]
    pub const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the backing texture.
    #[inline]
    pub const fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Returns the target width.
    #[inline]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the target height.
    #[inline]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the texture format.
    #[inline]
    pub const fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
}

/// Ordered post-processing stack with cached GPU resources.
pub struct PostStack {
    effects: Vec<PostEffect>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,
    uniform_buffer: Option<wgpu::Buffer>,
    pipelines: Vec<(wgpu::TextureFormat, Arc<wgpu::RenderPipeline>)>,
    scratch: [Option<PostTarget>; 2],
}

impl PostStack {
    /// Creates an empty post stack.
    #[inline]
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            bind_group_layout: None,
            sampler: None,
            uniform_buffer: None,
            pipelines: Vec::new(),
            scratch: [None, None],
        }
    }

    /// Returns the ordered effects.
    #[inline]
    pub fn effects(&self) -> &[PostEffect] {
        &self.effects
    }

    /// Returns the number of effects in the stack.
    #[inline]
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Returns whether the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Adds a bloom pass.
    pub fn with_bloom(mut self, config: BloomConfig) -> Self {
        self.effects.push(PostEffect::Bloom(config.normalized()));
        self
    }

    /// Adds an SSAO pass.
    pub fn with_ssao(mut self, config: SsaoConfig) -> Self {
        self.effects.push(PostEffect::Ssao(config.normalized()));
        self
    }

    /// Adds a tone mapping pass.
    pub fn with_tonemap(mut self, mapper: ToneMapper) -> Self {
        self.effects.push(PostEffect::Tonemap(mapper));
        self
    }

    /// Adds an FXAA pass.
    pub fn with_fxaa(mut self, config: FxaaConfig) -> Self {
        self.effects.push(PostEffect::Fxaa(config.normalized()));
        self
    }

    /// Adds a TAA pass.
    pub fn with_taa(mut self, config: TaaConfig) -> Self {
        self.effects.push(PostEffect::Taa(config.normalized()));
        self
    }

    /// Adds an SMAA pass.
    pub fn with_smaa(mut self, config: SmaaConfig) -> Self {
        self.effects.push(PostEffect::Smaa(config));
        self
    }

    /// Adds a depth-of-field pass.
    pub fn with_dof(mut self, config: DofConfig) -> Self {
        self.effects.push(PostEffect::Dof(config.normalized()));
        self
    }

    /// Adds a fog blend pass.
    pub fn with_fog(mut self, config: FogPostConfig) -> Self {
        self.effects.push(PostEffect::Fog(config.normalized()));
        self
    }

    /// Adds an outline pass.
    pub fn with_outline(mut self, config: OutlineConfig) -> Self {
        self.effects.push(PostEffect::Outline(config.normalized()));
        self
    }

    /// Adds a motion blur pass.
    pub fn with_motion_blur(mut self, config: MotionBlurConfig) -> Self {
        self.effects
            .push(PostEffect::MotionBlur(config.normalized()));
        self
    }

    /// Removes an effect by index.
    pub fn remove(&mut self, index: usize) -> Option<PostEffect> {
        if index < self.effects.len() {
            Some(self.effects.remove(index))
        } else {
            None
        }
    }

    /// Clears all effects while retaining GPU resources for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    /// Applies the stack from `source` to `output` and submits a GPU command buffer.
    pub fn apply_to_view(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source: &wgpu::TextureView,
        output: &wgpu::TextureView,
        context: PostContext,
    ) -> Result<PostStats, ScenixError> {
        if self.effects.is_empty() {
            return Ok(PostStats::default());
        }
        if context.resolution.x <= 0.0 || context.resolution.y <= 0.0 {
            return Err(ScenixError::Validation(ValidationError::OutOfRange));
        }

        let width = context.resolution.x as u32;
        let height = context.resolution.y as u32;
        let resized_targets =
            self.ensure_scratch_targets(device, width, height, context.color_format)?;
        let pipeline = self.pipeline(device, context.color_format);
        self.ensure_common_resources(device);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("scenix.post.encoder"),
        });
        let mut current_scratch: Option<usize> = None;

        for index in 0..self.effects.len() {
            let last = index + 1 == self.effects.len();
            let destination_scratch = if last { None } else { Some(index % 2) };
            let source_view = if let Some(scratch) = current_scratch {
                self.scratch[scratch].as_ref().unwrap().view()
            } else {
                source
            };
            let destination_view = if let Some(scratch) = destination_scratch {
                self.scratch[scratch].as_ref().unwrap().view()
            } else {
                output
            };

            let params = self.effects[index].params();
            self.render_effect(EffectPass {
                device,
                queue,
                pipeline: &pipeline,
                encoder: &mut encoder,
                source: source_view,
                destination: destination_view,
                params: &params,
            });
            current_scratch = destination_scratch;
        }

        queue.submit(Some(encoder.finish()));
        Ok(PostStats {
            passes: self.effects.len() as u32,
            resized_targets,
        })
    }

    fn ensure_scratch_targets(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<u32, ScenixError> {
        if self.effects.len() <= 1 {
            return Ok(0);
        }

        let mut resized = 0;
        for index in 0..2 {
            let replace = self.scratch[index].as_ref().is_none_or(|target| {
                target.width() != width || target.height() != height || target.format() != format
            });
            if replace {
                self.scratch[index] = Some(PostTarget::new(
                    device,
                    if index == 0 {
                        "scenix.post.scratch.0"
                    } else {
                        "scenix.post.scratch.1"
                    },
                    width,
                    height,
                    format,
                )?);
                resized += 1;
            }
        }
        Ok(resized)
    }

    fn ensure_common_resources(&mut self, device: &wgpu::Device) {
        if self.bind_group_layout.is_none() {
            self.bind_group_layout = Some(device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("scenix.post.bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                },
            ));
        }
        if self.sampler.is_none() {
            self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("scenix.post.sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            }));
        }
        if self.uniform_buffer.is_none() {
            self.uniform_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("scenix.post.uniforms"),
                size: std::mem::size_of::<PostUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
    }

    fn pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Arc<wgpu::RenderPipeline> {
        if let Some((_, pipeline)) = self
            .pipelines
            .iter()
            .find(|(pipeline_format, _)| *pipeline_format == format)
        {
            return Arc::clone(pipeline);
        }

        self.ensure_common_resources(device);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scenix.post.shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scenix.post.pipeline_layout"),
            bind_group_layouts: &[Some(self.bind_group_layout.as_ref().unwrap())],
            immediate_size: 0,
        });
        let pipeline = Arc::new(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("scenix.post.pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                multiview_mask: None,
                cache: None,
            }),
        );
        self.pipelines.push((format, Arc::clone(&pipeline)));
        pipeline
    }

    fn render_effect(&self, pass: EffectPass<'_>) {
        let uniform = PostUniform {
            values: *pass.params,
        };
        pass.queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&uniform),
        );
        let bind_group = pass.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scenix.post.bind_group"),
            layout: self.bind_group_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(pass.source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_ref().unwrap().as_entire_binding(),
                },
            ],
        });
        let mut render_pass = pass.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scenix.post.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: pass.destination,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(pass.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

impl Default for PostStack {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostUniform {
    values: [f32; 8],
}

struct EffectPass<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    pipeline: &'a wgpu::RenderPipeline,
    encoder: &'a mut wgpu::CommandEncoder,
    source: &'a wgpu::TextureView,
    destination: &'a wgpu::TextureView,
    params: &'a [f32; 8],
}
