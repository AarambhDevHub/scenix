//! Particle GPU tests (scenix v1.7.0).
//!
//! Everything is gated behind `SCENIX_RUN_GPU_TESTS=1` so CI lap-shared
//! machines and developer laptops skip wgpu work unless it is explicitly
//! requested (lavapipe and SwiftShader both provide compute support).

use scenix_camera::PerspectiveCamera;
use scenix_core::{Color, ParticleSystemId, ScenixError};
use scenix_material::ParticleMaterial;
use scenix_math::Vec3;
use scenix_particles::{ParticleEmitter, ParticleSystem};
use scenix_renderer::{ParticleComputeParams, Renderer, RendererConfig};
use scenix_scene::SceneGraph;

fn run_gpu_tests() -> bool {
    std::env::var("SCENIX_RUN_GPU_TESTS").as_deref() == Ok("1")
}

fn marker_material() -> ParticleMaterial {
    ParticleMaterial::new().with_color(Color::from_rgb(0.9, 0.5, 0.1))
}

/// Seeds a pool where 16 slots are alive and 16 latent (zero lifetime),
/// giving the respawn pass room to claim dead lanes.
fn burst_system() -> ParticleSystem {
    let emitter = ParticleEmitter::new()
        .with_rate(0.0)
        .with_lifetime(1.0, 1.5)
        .with_size(0.3)
        .with_speed(1.0, 2.0);
    let mut system = ParticleSystem::new(emitter, 32, 99);
    system.spawn_burst(16);
    system
}

fn camera() -> PerspectiveCamera {
    PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 2.0, 8.0))
        .target(Vec3::ZERO)
}

#[test]
fn billboard_pipeline_draws_the_cpu_batch() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(64, 64)).await?;
        let system_id = ParticleSystemId::new(1);
        let material = marker_material();

        renderer.register_particles(system_id, &material)?;
        let mut system = burst_system();
        let batch = system.update(1.0 / 60.0);
        renderer.update_particles(system_id, batch, &material)?;

        let stats = renderer.render(&SceneGraph::new(), &camera())?;
        assert_eq!(stats.particle_instances, batch.len() as u32);

        assert!(renderer.unregister_particles(system_id));
        Ok(())
    })
}

#[test]
fn compute_simulation_integrates_gravity_and_respawns() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(64, 64)).await?;
        let system_id = ParticleSystemId::new(2);
        let material = marker_material();

        // Ten slots born with a 0.1 s lifetime so the first GPU step kills
        // the whole pool; the second step lets the ring respawn a budget.
        let emitter = ParticleEmitter::new()
            .with_rate(0.0)
            .with_lifetime(0.1, 0.1)
            .with_size(0.3)
            .with_speed(1.0, 2.0);
        let mut source = ParticleSystem::new(emitter, 10, 99);
        source.spawn_burst(10);

        // dt = 0.1 matches the lifetime exactly: integration must age them
        // to death without rounding ambiguity in the shader comparison.
        let params = ParticleComputeParams {
            gravity_dt: [0.0, -9.8, 0.0, 0.1],
            ..ParticleComputeParams::default()
        };
        renderer.register_particle_compute(system_id, &material, &source, &params)?;

        let mut pool = Vec::new();
        renderer.read_particle_pool(system_id, &mut pool)?;
        assert_eq!(pool.len(), 10, "pool mirrors the CPU pool");
        assert!(pool.iter().all(|item| item.vel_life[3] > 0.0));

        // Step 1: no respawn budget — everything must integrate to death.
        renderer.simulate_particles_compute(system_id, &params, 0)?;
        renderer.read_particle_pool(system_id, &mut pool)?;
        let dead = pool
            .iter()
            .filter(|item| item.pos_age[3] >= item.vel_life[3])
            .count();
        assert_eq!(dead, 10, "dt 0.1 must age every 0.1 s lifetime to death");
        for item in &pool {
            assert!(
                (item.pos_age[3] - 0.1).abs() < 1e-4,
                "age must grow by dt, saw {}",
                item.pos_age[3]
            );
        }

        // Step 2: a budget of 6 respawns exactly six dead slots from the
        // ring window.
        renderer.simulate_particles_compute(system_id, &params, 6)?;
        renderer.read_particle_pool(system_id, &mut pool)?;
        let alive = pool
            .iter()
            .filter(|item| item.pos_age[3] < item.vel_life[3])
            .count();
        assert_eq!(alive, 6, "the respawn budget must claim six slots");
        for item in &pool {
            if item.pos_age[3] < item.vel_life[3] {
                assert_eq!(item.pos_age[3], 0.0, "respawned slots restart at 0");
                assert!(item.vel_life[3] >= 0.5 && item.vel_life[3] <= 1.5);
                assert!(item.vel_life[1] > 0.0, "respawns move along direction");
            }
        }
        let cursor = renderer.read_particle_ring(system_id)?;
        assert_eq!(cursor, 6, "the ring cursor lands on the claim boundary");

        // A CPU update converts the slot back to the draw path.
        renderer.update_particles(system_id, source.update(1.0 / 60.0), &material)?;
        assert!(renderer.unregister_particles(system_id));
        Ok(())
    })
}
