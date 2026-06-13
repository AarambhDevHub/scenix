use scenix_math::Vec2;
use scenix_post::{BloomConfig, FxaaConfig, PostContext, PostStack, PostTarget, ToneMapper};

fn test_error(error: impl core::fmt::Debug) -> String {
    format!("{error:?}")
}

fn run_gpu_tests() -> bool {
    std::env::var("SCENIX_RUN_GPU_TESTS").as_deref() == Ok("1")
}

#[test]
fn gpu_post_stack_smoke_produces_non_black_target() -> Result<(), String> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let (device, queue) = device().await?;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let source =
            PostTarget::new(&device, "post.test.source", 4, 4, format).map_err(test_error)?;
        let output =
            PostTarget::new(&device, "post.test.output", 4, 4, format).map_err(test_error)?;

        clear_target(&device, &queue, source.view());

        let mut stack = PostStack::new()
            .with_bloom(BloomConfig::default())
            .with_tonemap(ToneMapper::Reinhard)
            .with_fxaa(FxaaConfig::default());
        let stats = stack
            .apply_to_view(
                &device,
                &queue,
                source.view(),
                output.view(),
                PostContext {
                    frame_index: 0,
                    resolution: Vec2::new(4.0, 4.0),
                    color_format: format,
                },
            )
            .map_err(test_error)?;
        assert_eq!(stats.passes, 3);

        let pixel = read_pixel(&device, &queue, output.texture())?;
        assert!(pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0);
        Ok(())
    })
}

async fn device() -> Result<(wgpu::Device, wgpu::Queue), String> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::all();
    let instance = wgpu::Instance::new(descriptor);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .map_err(test_error)?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("scenix.post.test.device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        })
        .await
        .map_err(test_error)?;
    Ok((device, queue))
}

fn clear_target(device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("scenix.post.test.clear.encoder"),
    });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scenix.post.test.clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.25,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    queue.submit(Some(encoder.finish()));
}

fn read_pixel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Result<[u8; 4], String> {
    let padded_bytes_per_row = 256_u32;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scenix.post.test.readback"),
        size: padded_bytes_per_row as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("scenix.post.test.readback.encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(test_error)?;
    receiver.recv().map_err(test_error)?.map_err(test_error)?;
    let mapped = slice.get_mapped_range();
    let pixel = [mapped[0], mapped[1], mapped[2], mapped[3]];
    drop(mapped);
    buffer.unmap();
    Ok(pixel)
}
