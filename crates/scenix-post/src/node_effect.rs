//! Full-screen node effect programs (scenix v1.6.0), behind the `nodes`
//! feature.
//!
//! A [`NodeEffectProgram`] owns one generated WGSL module produced by a
//! `scenix-nodes` post graph and executes it against the scenix post-pass
//! contract: group 0 binding 0 is the source texture, binding 1 the linear
//! sampler, and binding 2 the `[f32; 8]` parameter block whose slots align
//! with [`scenix_nodes::slots`].

use scenix_nodes::{PostNodeEffect, WgslOptions, generate_post_wgsl};

/// Uniform block declared by generated post node modules (`PostParams`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct NodePostUniform {
    values: [f32; 8],
}

/// Executor for one validated post node effect.
///
/// The program lazily builds its pipeline per target color format and reuses
/// the sampler, bind-group layout, and uniform buffer across frames.
#[derive(Debug)]
pub struct NodeEffectProgram {
    label: Option<String>,
    wgsl_source: String,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,
    uniform_buffer: Option<wgpu::Buffer>,
    pipelines: Vec<(wgpu::TextureFormat, wgpu::RenderPipeline)>,
}

impl NodeEffectProgram {
    /// Creates a program from raw WGSL with an optional debug label.
    ///
    /// Use [`NodeEffectProgram::from_effect`] for scenix-nodes generated
    /// modules.
    #[inline]
    pub fn with_source(name: impl Into<String>, wgsl_source: impl Into<String>) -> Self {
        Self {
            label: Some(name.into()),
            wgsl_source: wgsl_source.into(),
            bind_group_layout: None,
            sampler: None,
            uniform_buffer: None,
            pipelines: Vec::new(),
        }
    }

    /// Creates and generates the WGSL module for one validated effect.
    ///
    /// # Errors
    /// Returns the first [`scenix_nodes::NodeDiagnostic`] produced by graph
    /// resolution.
    pub fn from_effect(effect: &PostNodeEffect) -> Result<Self, scenix_nodes::NodeDiagnostic> {
        let program = generate_post_wgsl(effect, &WgslOptions::default())?;
        let label = program.label().map(str::to_owned);
        Ok(Self {
            label,
            wgsl_source: program.into_source(),
            bind_group_layout: None,
            sampler: None,
            uniform_buffer: None,
            pipelines: Vec::new(),
        })
    }

    /// Returns the program label.
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the WGSL module backing the program.
    #[inline]
    pub fn source(&self) -> &str {
        &self.wgsl_source
    }

    fn ensure_common_resources(&mut self, device: &wgpu::Device) {
        if self.bind_group_layout.is_none() {
            self.bind_group_layout = Some(device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("scenix.node_effect.layout"),
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
                label: Some("scenix.node_effect.sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            }));
        }
        if self.uniform_buffer.is_none() {
            self.uniform_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("scenix.node_effect.uniforms"),
                size: std::mem::size_of::<NodePostUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
    }

    /// Returns the pipeline for one target format, generating it on first use.
    fn pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> &wgpu::RenderPipeline {
        if let Some(index) = self
            .pipelines
            .iter()
            .position(|(pipeline_format, _)| *pipeline_format == format)
        {
            return &self.pipelines[index].1;
        }

        self.ensure_common_resources(device);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scenix.node_effect.shader"),
            source: wgpu::ShaderSource::Wgsl(self.wgsl_source.clone().into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scenix.node_effect.pipeline_layout"),
            bind_group_layouts: &[Some(self.bind_group_layout.as_ref().unwrap())],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scenix.node_effect.pipeline"),
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
        });
        self.pipelines.push((format, pipeline));
        let index = self.pipelines.len() - 1;
        &self.pipelines[index].1
    }

    /// Returns the lazily built bind-group layout.
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.bind_group_layout.as_ref().unwrap()
    }

    /// Renders the node effect from `source` into `destination`.
    ///
    /// `params` fills the `[f32; 8]` post parameter block; slots follow the
    /// bundled preset conventions documented in [`scenix_nodes::slots`].
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        destination: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        params: &[f32; 8],
    ) {
        self.ensure_common_resources(device);
        queue.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&NodePostUniform { values: *params }),
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scenix.node_effect.bind_group"),
            layout: self.bind_group_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
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
        let pipeline = self.pipeline(device, format);
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scenix.node_effect.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: destination,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use scenix_nodes::NodeDiagnosticKind::StageUnsupported;
    use scenix_nodes::{PostNodeEffect, ShaderGraph, ShaderNodeKind, UniformInput};

    use crate::NodeEffectProgram;

    #[test]
    fn preset_graphs_generate_programs() {
        let effect = PostNodeEffect::vignette();
        let program = NodeEffectProgram::from_effect(&effect);
        assert!(program.is_ok());
    }

    #[test]
    fn from_effect_rejects_stage_incompatible_nodes() {
        // Uniforms do not exist in the post stage.
        let mut graph = ShaderGraph::new();
        let time = graph.push(ShaderNodeKind::Uniform(UniformInput::Time));
        graph.add_fragment_output(time, None);
        let effect = PostNodeEffect::new("uniform-in-post", graph);
        assert!(matches!(
            NodeEffectProgram::from_effect(&effect),
            Err(diagnostic) if matches!(diagnostic.kind, StageUnsupported { .. })
        ));
    }

    #[test]
    fn with_source_records_label_and_module() {
        let program = NodeEffectProgram::with_source("flip", "// wgsl");
        assert_eq!(program.label(), Some("flip"));
        assert_eq!(program.source(), "// wgsl");
    }
}
