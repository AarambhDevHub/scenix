use std::hint::black_box;
use std::time::Instant;

use scenix_math::Vec2;
use scenix_post::{
    BloomConfig, FxaaConfig, PostContext, PostStack, PostTarget, SsaoConfig, TaaConfig, ToneMapper,
};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    println!("{name}: {iterations} iterations in {:?}", start.elapsed());
}

fn main() {
    let stack = PostStack::new()
        .with_ssao(SsaoConfig::default())
        .with_bloom(BloomConfig::default())
        .with_tonemap(ToneMapper::Aces)
        .with_fxaa(FxaaConfig::default())
        .with_taa(TaaConfig::default());
    black_box(stack.len());

    if std::env::var("SCENIX_RUN_GPU_BENCHES").as_deref() != Ok("1") {
        println!("set SCENIX_RUN_GPU_BENCHES=1 to run post GPU benches");
        return;
    }

    pollster::block_on(async {
        let (device, queue) = device().await.expect("wgpu device");
        run_gpu_bench(&device, &queue, 1280, 720, "post_stack_720p");
        run_gpu_bench(&device, &queue, 1920, 1080, "post_stack_1080p");
    });
}

fn run_gpu_bench(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, name: &str) {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let source = PostTarget::new(device, "scenix.post.bench.source", width, height, format)
        .expect("source target");
    let output = PostTarget::new(device, "scenix.post.bench.output", width, height, format)
        .expect("output target");
    let mut stack = PostStack::new()
        .with_ssao(SsaoConfig::default())
        .with_bloom(BloomConfig::default())
        .with_tonemap(ToneMapper::Aces)
        .with_fxaa(FxaaConfig::default())
        .with_taa(TaaConfig::default());

    bench(name, 60, || {
        black_box(
            stack
                .apply_to_view(
                    device,
                    queue,
                    source.view(),
                    output.view(),
                    PostContext {
                        frame_index: 0,
                        resolution: Vec2::new(width as f32, height as f32),
                        color_format: format,
                    },
                )
                .unwrap(),
        );
    });
}

async fn device() -> Result<(wgpu::Device, wgpu::Queue), Box<dyn std::error::Error>> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("scenix.post.bench.device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        })
        .await?;
    Ok((device, queue))
}
