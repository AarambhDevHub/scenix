//! scenix v1.7.0 particles — deterministic CPU fountain plus (optionally)
//! the GPU compute path when the `renderer` feature is enabled.
//!
//! Run: `cargo run -p scenix --example particles_basic --features particles`

use scenix::{
    Color, ColorCurve, ColorOverLifetime, Drag, EmissionShape, FlipbookConfig, FlipbookGrid,
    Gravity, ParticleEmitter, ParticleMaterial, ParticleSystem, SizeOverLifetime, Vec3,
};

fn fountain() -> ParticleEmitter {
    ParticleEmitter::new()
        .with_rate(240.0)
        .with_lifetime(0.9, 1.6)
        .with_size(0.2)
        .with_speed(4.0, 7.0)
        .toward(Vec3::new(0.0, 1.0, 0.0))
        .with_shape(EmissionShape::Cone {
            radius: 0.15,
            height: 0.35,
        })
        .with_modifier(Gravity::new(Vec3::new(0.0, -12.0, 0.0)))
        .with_modifier(Drag::new(0.15))
        .with_modifier(ColorOverLifetime::new(ColorCurve::from_to(
            [1.0, 0.85, 0.35, 1.0],
            [0.9, 0.15, 0.0, 0.0],
        )))
        .with_modifier(SizeOverLifetime::from_to(1.15, 0.05))
}

fn main() {
    // Pure CPU: one deterministic system drives the whole demo.
    let system_id = scenix::ParticleSystemId::new(1);
    let mut system = ParticleSystem::new(fountain().at(Vec3::new(0.0, 0.0, 0.0)), 1_024, 2021018);
    system.set_flipbook(Some(FlipbookConfig::full(4, 4, 24.0)));

    // The material carries the GPU draw path: tint + flipbook atlas on the
    // billboard shader (see scenix-material `FeatureBits`).
    let material = ParticleMaterial::new()
        .with_color(Color::from_rgb(1.0, 0.9, 0.6))
        .with_flipbook(Some(scenix::FlipbookParams::full(4, 4, 24.0)));

    // 60 frames of simulation; print a heartbeat every quarter second.
    let dt = 1.0 / 60.0;
    for frame in 0..60 {
        let batch = system.update(dt);
        if frame % 15 == 0 {
            let centroid: [f32; 3] = batch
                .items
                .iter()
                .fold([0.0; 3], |acc, item| {
                    [
                        acc[0] + item.position[0],
                        acc[1] + item.position[1],
                        acc[2] + item.position[2],
                    ]
                })
                .map(|sum| sum / batch.len().max(1) as f32);
            println!(
                "frame {frame:02}: {} alive, spawn total {}, centroid [{:.2}, {:.2}, {:.2}]",
                batch.len(),
                system.total_spawned(),
                centroid[0],
                centroid[1],
                centroid[2]
            );
        }
    }

    // Atlas sanity check: 4×4 cells, frame 5 sits in column 1 row 1.
    let grid = FlipbookGrid::new(4, 4).expect("non-degenerate grid");
    let cell = grid.uv(5);
    println!(
        "flipbook frame 5 uv [{:.2}, {:.2}, {:.2}, {:.2}] of {} frames",
        cell.u0,
        cell.v0,
        cell.u1,
        cell.v1,
        grid.frame_count()
    );

    // Optional GPU tail: the same seed uploads to a compute pool and the
    // renderer simulates until the particles burn out.
    #[cfg(feature = "renderer")]
    render_gpu_tail(system_id, material);

    println!("particles_basic done");
}

/// Drives the wgpu compute simulation headlessly when the `renderer`
/// feature is on (the GPU test suite `tests/particles_gpu.rs` covers the
/// same path with assertions).
#[cfg(feature = "renderer")]
fn render_gpu_tail(system_id: scenix::ParticleSystemId, material: ParticleMaterial) {
    use scenix::{ParticleComputeParams, PerspectiveCamera, Renderer, RendererConfig, SceneGraph};

    let renderer = match pollster::block_on(Renderer::headless(RendererConfig::new(96, 96))) {
        Ok(renderer) => renderer,
        Err(err) => {
            println!("headless renderer unavailable: {err:?}");
            return;
        }
    };
    let mut renderer = renderer;
    let mut source = ParticleSystem::new(fountain().at(Vec3::new(0.0, 0.0, 0.0)), 256, 2021018);
    source.spawn_burst(64);

    let params = ParticleComputeParams::default().with_spawn_budget(16);
    if let Err(err) = renderer.register_particle_compute(system_id, &material, &source, &params) {
        println!("compute registration unavailable: {err:?}");
        return;
    }
    for _ in 0..30 {
        let _ = renderer.simulate_particles_compute(system_id, &params, 16);
    }
    let mut pool = Vec::new();
    match renderer.read_particle_pool(system_id, &mut pool) {
        Ok(()) => {
            let falling = pool.iter().filter(|item| item.vel_life[1] < 0.0).count();
            println!("computed pool: {} slots, {} falling", pool.len(), falling);
        }
        Err(err) => println!("pool readback unavailable: {err:?}"),
    }
    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 4.0, 12.0))
        .target(Vec3::ZERO);
    if let Err(err) = renderer.render(&SceneGraph::new(), &camera) {
        println!("render skipped: {err:?}");
    }
}
