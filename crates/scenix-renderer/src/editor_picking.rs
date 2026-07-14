use scenix_camera::PerspectiveCamera;
use scenix_core::{GpuError, MeshId, NodeId, ScenixError, ValidationError};
use scenix_math::{Mat4, Vec3};
use scenix_scene::{NodeKind, SceneGraph};

use crate::gbuffer::TextureTarget;
use crate::{GpuScene, PackedVertex};

const READBACK_ROW: u64 = 256;
const READBACK_SIZE: u64 = READBACK_ROW * 3;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PickFrameUniform {
    view_projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PickObjectUniform {
    world: [[f32; 4]; 4],
    id_bytes: [u32; 4],
}

#[derive(Clone, Copy, Debug)]
struct EditorDraw {
    node_id: NodeId,
    mesh_id: MeshId,
    world: Mat4,
}

pub(crate) struct EditorRenderRequest<'a> {
    pub scene: &'a SceneGraph,
    pub camera: &'a PerspectiveCamera,
    pub layers: u32,
    pub selectable_only: bool,
    pub scissor: Option<(u32, u32)>,
}

/// One on-demand editor-picking request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EditorPickRequest {
    /// Physical target pixel X coordinate.
    pub x: u32,
    /// Physical target pixel Y coordinate.
    pub y: u32,
    /// Scene layer mask.
    pub layers: u32,
    /// Whether scene editor selection policy must be honored.
    pub selectable_only: bool,
}

impl EditorPickRequest {
    /// Creates a pick request testing all selectable layers.
    pub const fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            layers: u32::MAX,
            selectable_only: true,
        }
    }
}

/// Decoded object, depth, normal, and reconstructed position at one pixel.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EditorPickResult {
    /// Picked scene node, or `None` for cleared background.
    pub node_id: Option<NodeId>,
    /// WebGPU depth in `0..=1`.
    pub depth: f32,
    /// Decoded world-space normal.
    pub normal: Vec3,
    /// Reconstructed world-space position for a hit.
    pub world_position: Option<Vec3>,
}

/// Allocation and usage counters for optional editor buffers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EditorBufferStats {
    /// Whether GPU editor buffers are allocated.
    pub allocated: bool,
    /// Current width.
    pub width: u32,
    /// Current height.
    pub height: u32,
    /// Approximate texture/readback/uniform bytes.
    pub memory_bytes: u64,
    /// Number of submitted editor-picking passes.
    pub pick_requests: u64,
}

/// Reusable on-demand ID, normal, depth, uniform, and readback resources.
#[derive(Debug)]
pub struct EditorBuffers {
    id: TextureTarget,
    normal: TextureTarget,
    depth: TextureTarget,
    depth_read: TextureTarget,
    pipeline: wgpu::RenderPipeline,
    frame_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    object_layout: wgpu::BindGroupLayout,
    object_buffer: wgpu::Buffer,
    object_bind_group: wgpu::BindGroup,
    object_stride: u64,
    object_capacity: usize,
    object_bytes: Vec<u8>,
    readback: wgpu::Buffer,
    id_map: Vec<NodeId>,
    draws: Vec<EditorDraw>,
}

impl EditorBuffers {
    /// Allocates editor buffers for a target size.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let frame_layout = uniform_layout(device, "scenix.editor.frame.layout", false);
        let object_layout = uniform_layout(device, "scenix.editor.object.layout", true);
        let frame_buffer = uniform_buffer(
            device,
            "scenix.editor.frame.uniform",
            core::mem::size_of::<PickFrameUniform>() as u64,
        );
        let frame_bind_group = uniform_bind_group(
            device,
            "scenix.editor.frame.bind_group",
            &frame_layout,
            &frame_buffer,
            core::mem::size_of::<PickFrameUniform>() as u64,
        );
        let object_stride = aligned_uniform_size::<PickObjectUniform>();
        let object_capacity = 256;
        let object_buffer = uniform_buffer(
            device,
            "scenix.editor.object.uniform",
            object_stride * object_capacity as u64,
        );
        let object_bind_group = uniform_bind_group(
            device,
            "scenix.editor.object.bind_group",
            &object_layout,
            &object_buffer,
            core::mem::size_of::<PickObjectUniform>() as u64,
        );
        let pipeline = create_pipeline(device, &frame_layout, &object_layout);
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scenix.editor.readback"),
            size: READBACK_SIZE,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let (id, normal, depth, depth_read) = create_targets(device, width, height);
        Self {
            id,
            normal,
            depth,
            depth_read,
            pipeline,
            frame_buffer,
            frame_bind_group,
            object_layout,
            object_buffer,
            object_bind_group,
            object_stride,
            object_capacity,
            object_bytes: Vec::new(),
            readback,
            id_map: vec![NodeId::default()],
            draws: Vec::new(),
        }
    }

    /// Reallocates only textures when target dimensions change.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.id.width() != width || self.id.height() != height {
            (self.id, self.normal, self.depth, self.depth_read) =
                create_targets(device, width, height);
        }
    }

    /// Object-ID texture view.
    pub const fn id_view(&self) -> &wgpu::TextureView {
        self.id.view()
    }

    /// Encoded normal texture view.
    pub const fn normal_view(&self) -> &wgpu::TextureView {
        self.normal.view()
    }

    /// Depth texture view.
    pub const fn depth_view(&self) -> &wgpu::TextureView {
        self.depth.view()
    }

    /// Current width.
    pub const fn width(&self) -> u32 {
        self.id.width()
    }

    /// Current height.
    pub const fn height(&self) -> u32 {
        self.id.height()
    }

    pub(crate) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gpu_scene: &GpuScene,
        request: EditorRenderRequest<'_>,
    ) -> Result<u32, ScenixError> {
        self.draws.clear();
        self.id_map.clear();
        self.id_map.push(NodeId::default());
        for node_id in request.scene.iter_depth_first() {
            let Some(node) = request.scene.get(node_id) else {
                continue;
            };
            if !node.visible || node.layer & request.layers == 0 {
                continue;
            }
            if request.selectable_only && !request.scene.is_selectable(node_id) {
                continue;
            }
            let NodeKind::Mesh { mesh_id, .. } = node.kind else {
                continue;
            };
            if gpu_scene.mesh(mesh_id).is_none() {
                continue;
            }
            self.id_map.push(node_id);
            self.draws.push(EditorDraw {
                node_id,
                mesh_id,
                world: request
                    .scene
                    .world_matrix(node_id)
                    .unwrap_or(Mat4::IDENTITY),
            });
        }
        if self.draws.len() > u32::MAX as usize - 1 {
            return Err(ScenixError::Validation(ValidationError::OutOfRange));
        }
        self.ensure_object_capacity(device, self.draws.len().max(1));
        let required = self.object_stride as usize * self.draws.len().max(1);
        self.object_bytes.resize(required, 0);
        self.object_bytes.fill(0);
        for (index, draw) in self.draws.iter().enumerate() {
            debug_assert_eq!(self.id_map[index + 1], draw.node_id);
            let id = (index + 1) as u32;
            let object = PickObjectUniform {
                world: mat4_uniform(draw.world),
                id_bytes: [
                    id & 0xff,
                    (id >> 8) & 0xff,
                    (id >> 16) & 0xff,
                    (id >> 24) & 0xff,
                ],
            };
            let offset = self.object_stride as usize * index;
            self.object_bytes[offset..offset + core::mem::size_of::<PickObjectUniform>()]
                .copy_from_slice(bytemuck::bytes_of(&object));
        }
        let frame = PickFrameUniform {
            view_projection: mat4_uniform(request.camera.view_projection()),
        };
        queue.write_buffer(&self.frame_buffer, 0, bytemuck::bytes_of(&frame));
        if !self.draws.is_empty() {
            queue.write_buffer(&self.object_buffer, 0, &self.object_bytes);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("scenix.editor.pick.encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scenix.editor.pick.pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.id.view(),
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.normal.view(),
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.depth_read.view(),
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some((x, y)) = request.scissor {
                pass.set_scissor_rect(x, y, 1, 1);
            }
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.frame_bind_group, &[]);
            for (index, draw) in self.draws.iter().enumerate() {
                let Some(mesh) = gpu_scene.mesh(draw.mesh_id) else {
                    continue;
                };
                pass.set_bind_group(
                    1,
                    &self.object_bind_group,
                    &[(index as u64 * self.object_stride) as u32],
                );
                pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                pass.set_index_buffer(
                    mesh.index_buffer().slice(..),
                    mesh.packed().index_format.to_wgpu(),
                );
                pass.draw_indexed(0..mesh.packed().index_count, 0, 0..1);
            }
        }
        queue.submit(Some(encoder.finish()));
        Ok(self.draws.len() as u32)
    }

    pub(crate) fn read_pixel(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &PerspectiveCamera,
        x: u32,
        y: u32,
    ) -> Result<EditorPickResult, ScenixError> {
        if x >= self.width() || y >= self.height() {
            return Err(ScenixError::Validation(ValidationError::OutOfRange));
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("scenix.editor.readback.encoder"),
        });
        copy_pixel(
            &mut encoder,
            self.id.texture(),
            &self.readback,
            x,
            y,
            0,
            wgpu::TextureAspect::All,
        );
        copy_pixel(
            &mut encoder,
            self.normal.texture(),
            &self.readback,
            x,
            y,
            READBACK_ROW,
            wgpu::TextureAspect::All,
        );
        copy_pixel(
            &mut encoder,
            self.depth_read.texture(),
            &self.readback,
            x,
            y,
            READBACK_ROW * 2,
            wgpu::TextureAspect::All,
        );
        queue.submit(Some(encoder.finish()));

        let slice = self.readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|_| ScenixError::Gpu(GpuError::Upload))?;
        receiver
            .recv()
            .map_err(|_| ScenixError::Gpu(GpuError::Upload))?
            .map_err(|_| ScenixError::Gpu(GpuError::Upload))?;
        let mapped = slice.get_mapped_range();
        let id = u32::from_le_bytes([mapped[0], mapped[1], mapped[2], mapped[3]]);
        let normal_offset = READBACK_ROW as usize;
        let normal = Vec3::new(
            mapped[normal_offset] as f32 / 255.0 * 2.0 - 1.0,
            mapped[normal_offset + 1] as f32 / 255.0 * 2.0 - 1.0,
            mapped[normal_offset + 2] as f32 / 255.0 * 2.0 - 1.0,
        )
        .normalize();
        let depth_offset = (READBACK_ROW * 2) as usize;
        let depth = f32::from_le_bytes([
            mapped[depth_offset],
            mapped[depth_offset + 1],
            mapped[depth_offset + 2],
            mapped[depth_offset + 3],
        ]);
        drop(mapped);
        self.readback.unmap();
        let node_id = self
            .id_map
            .get(id as usize)
            .copied()
            .filter(|id| !id.is_null());
        let world_position = node_id.and_then(|_| {
            let inverse = camera.view_projection().inverse()?;
            let ndc = Vec3::new(
                (x as f32 + 0.5) / self.width() as f32 * 2.0 - 1.0,
                1.0 - (y as f32 + 0.5) / self.height() as f32 * 2.0,
                depth,
            );
            Some(inverse.mul_vec3(ndc))
        });
        Ok(EditorPickResult {
            node_id,
            depth,
            normal: if node_id.is_some() {
                normal
            } else {
                Vec3::ZERO
            },
            world_position,
        })
    }

    /// Approximate memory owned by the editor buffers.
    pub fn memory_bytes(&self) -> u64 {
        let pixels = self.width() as u64 * self.height() as u64;
        pixels * 16 + READBACK_SIZE + self.object_stride * self.object_capacity as u64
    }

    fn ensure_object_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.object_capacity {
            return;
        }
        self.object_capacity = needed.next_power_of_two();
        self.object_buffer = uniform_buffer(
            device,
            "scenix.editor.object.uniform",
            self.object_stride * self.object_capacity as u64,
        );
        self.object_bind_group = uniform_bind_group(
            device,
            "scenix.editor.object.bind_group",
            &self.object_layout,
            &self.object_buffer,
            core::mem::size_of::<PickObjectUniform>() as u64,
        );
    }
}

fn create_targets(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (TextureTarget, TextureTarget, TextureTarget, TextureTarget) {
    let color_usage = wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::COPY_SRC;
    let depth_usage = wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::COPY_SRC;
    (
        TextureTarget::new(
            device,
            "scenix.editor.id",
            width,
            height,
            wgpu::TextureFormat::Rgba8Unorm,
            color_usage,
        ),
        TextureTarget::new(
            device,
            "scenix.editor.normal",
            width,
            height,
            wgpu::TextureFormat::Rgba8Unorm,
            color_usage,
        ),
        TextureTarget::new(
            device,
            "scenix.editor.depth",
            width,
            height,
            wgpu::TextureFormat::Depth32Float,
            depth_usage,
        ),
        TextureTarget::new(
            device,
            "scenix.editor.depth_read",
            width,
            height,
            wgpu::TextureFormat::R32Float,
            color_usage,
        ),
    )
}

fn create_pipeline(
    device: &wgpu::Device,
    frame_layout: &wgpu::BindGroupLayout,
    object_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("scenix.editor.pick.shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/editor_pick.wgsl").into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("scenix.editor.pick.pipeline.layout"),
        bind_group_layouts: &[Some(frame_layout), Some(object_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("scenix.editor.pick.pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[PackedVertex::layout()],
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
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn copy_pixel(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    buffer: &wgpu::Buffer,
    x: u32,
    y: u32,
    offset: u64,
    aspect: wgpu::TextureAspect,
) {
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect,
        },
        wgpu::TexelCopyBufferInfo {
            buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset,
                bytes_per_row: Some(READBACK_ROW as u32),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
}

fn uniform_layout(
    device: &wgpu::Device,
    label: &'static str,
    dynamic: bool,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn uniform_buffer(device: &wgpu::Device, label: &'static str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(1),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn uniform_bind_group(
    device: &wgpu::Device,
    label: &'static str,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    binding_size: u64,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: wgpu::BufferSize::new(binding_size),
            }),
        }],
    })
}

fn aligned_uniform_size<T>() -> u64 {
    (core::mem::size_of::<T>() as u64 + 255) & !255
}

fn mat4_uniform(matrix: Mat4) -> [[f32; 4]; 4] {
    [
        matrix.cols[0].to_array(),
        matrix.cols[1].to_array(),
        matrix.cols[2].to_array(),
        matrix.cols[3].to_array(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_pick_request_defaults_to_selectable_all_layers() {
        let request = EditorPickRequest::new(2, 3);
        assert_eq!(request.layers, u32::MAX);
        assert!(request.selectable_only);
    }
}
