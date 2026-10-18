//! Node-material pipeline and uniform support (scenix v1.6.0), behind the
//! `nodes` feature.
//!
//! Node materials compile from WGSL produced by `scenix-nodes` and reuse the
//! built-in bind-group layout tuple (frame, object, material, lights) plus
//! the shared [`crate::gpu_scene::PackedVertex`] buffer layout. Per-draw CPU state lands in the
//! same material uniform buffer used by built-in shaders, tagged with shader
//! kind `7.0` so preview fallbacks can detect node artifacts.

use crate::MaterialUniform;
use crate::gpu_scene::NodeMaterialSlot;

/// Packs one node draw's material uniform.
pub(crate) fn node_material_uniform(slot: &NodeMaterialSlot) -> MaterialUniform {
    let alpha_cutoff = if slot.alpha_cutoff >= 0.0 {
        Some(slot.alpha_cutoff)
    } else {
        None
    };
    MaterialUniform::new(
        slot.base_color,
        slot.emissive,
        slot.metallic,
        slot.roughness,
        alpha_cutoff,
        7.0,
        slot.feature_bits,
    )
}

/// Builds the render pipeline for one generated WGSL node module.
///
/// Mirrors the built-in draw pipeline: depth32/less-equal depth, alpha
/// blending, shared vertex layout, and the frame/object/material/light
/// bind-group layouts. Draw-time double-sided state selects culling through
/// the encoded feature bits inside the WGSL module's behavior.
pub(crate) fn create_node_pipeline(
    device: &wgpu::Device,
    uniforms: &crate::renderer::UniformResources,
    wgsl_source: &str,
    color_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("scenix.node_material.shader"),
        source: wgpu::ShaderSource::Wgsl(wgsl_source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("scenix.node_material.layout"),
        bind_group_layouts: &[
            Some(&uniforms.frame_layout),
            Some(&uniforms.object_layout),
            Some(&uniforms.material_layout),
            Some(&uniforms.light_layout),
        ],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("scenix.node_material.pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[crate::PackedVertex::layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}
