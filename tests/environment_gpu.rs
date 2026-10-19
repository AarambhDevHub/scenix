use scenix_camera::PerspectiveCamera;
use scenix_core::{Color, MaterialId, ScenixError, SkyId, TerrainId, WaterId};
use scenix_material::{SkyMaterial, TerrainMaterial, WaterMaterial};
use scenix_math::Vec3;
use scenix_renderer::{EnvironmentMaterial, Renderer, RendererConfig};
use scenix_scene::{SceneGraph, SceneNode};
use scenix_terrain::{Heightmap, LodPolicy, TerrainConfig, TerrainGrid};
use scenix_water::WaveField;

fn run_gpu_tests() -> bool {
    std::env::var("SCENIX_RUN_GPU_TESTS").as_deref() == Ok("1")
}

fn terrain_material() -> TerrainMaterial {
    TerrainMaterial::default_palette()
}

async fn environment_renderer() -> Result<(Renderer, SceneGraph, PerspectiveCamera), ScenixError> {
    let mut renderer = Renderer::headless(RendererConfig::new(64, 64)).await?;

    // Terrain chunk geometry through the v1.8 CPU crate.
    let heights = Heightmap::from_fn(16, 16, 1.0, |ix, iz| ((ix + iz) as f32) * 0.125);
    let config = TerrainConfig::new(2, 8, LodPolicy::default_three_band());
    let grid = TerrainGrid::generate(&heights, config);
    let geometry = grid
        .chunk_geometry_with_splat(0, 1, &|x, _z, _h| {
            // Grass palette, with a rock band on the left half.
            let weights = if x < 8.0 {
                [0.1, 0.1, 0.7, 0.1]
            } else {
                [0.55, 0.25, 0.15, 0.05]
            };
            Some(weights)
        })
        .expect("chunk 0 exists");

    renderer.register_environment(
        MaterialId::new(1),
        EnvironmentMaterial::Terrain(&terrain_material(), &geometry),
    )?;
    renderer.register_environment(
        MaterialId::new(2),
        EnvironmentMaterial::Sky(&SkyMaterial::clear_day()),
    )?;
    renderer.register_environment(
        MaterialId::new(3),
        EnvironmentMaterial::Water(
            &WaterMaterial::clear().with_foam(0.45, 6.0),
            &WaveField::calm(),
        ),
    )?;

    let mut scene = SceneGraph::new();
    scene.add(
        SceneNode::sky("dome", SkyId::new(1), MaterialId::new(2)).transform(
            scenix_math::Transform::new(
                Vec3::ZERO,
                scenix_math::Quat::IDENTITY,
                Vec3::new(100.0, 100.0, 100.0),
            ),
        ),
    );
    scene.add(
        SceneNode::terrain("terrain", TerrainId::new(1), MaterialId::new(1))
            .transform(scenix_math::Transform::IDENTITY),
    );
    scene.add(
        SceneNode::water("patch", WaterId::new(1), MaterialId::new(3)).transform(
            scenix_math::Transform::new(
                Vec3::new(0.0, 0.2, 0.0),
                scenix_math::Quat::IDENTITY,
                Vec3::new(8.0, 1.0, 8.0),
            ),
        ),
    );
    scene.update_world_transforms();

    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 200.0)
        .position(Vec3::new(0.0, 3.0, 8.0))
        .target(Vec3::new(0.0, 0.0, 0.0));

    Ok((renderer, scene, camera))
}

#[test]
fn environment_scene_renders_and_counts_environment_draws() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let (mut renderer, scene, camera) = environment_renderer().await?;
        let stats = renderer.render(&scene, &camera)?;
        assert_eq!(stats.scene_environments, 3);
        assert_eq!(stats.visible_environments, 3);
        assert_eq!(renderer.environment_count(), 3);

        // The sky dome paints the backdrop: register a hand-picked flat
        // gradient, render a sky-only scene aimed slightly above the
        // horizon, and the center pixel must be the blue bookend.
        let flat_sky = SkyMaterial::clear_day().with_gradient(
            Color::rgb(0.15, 0.35, 0.85),
            Color::rgb(0.45, 0.65, 1.0),
            Color::rgb(0.05, 0.15, 0.85),
        );
        renderer.register_environment(MaterialId::new(2), EnvironmentMaterial::Sky(&flat_sky))?;
        let mut sky_only = SceneGraph::new();
        // Same 100 m scale as the registered slot geometry implies: every ray
        // from inside the dome must hit its shell.
        sky_only.add(
            SceneNode::sky("dome", SkyId::new(1), MaterialId::new(2)).transform(
                scenix_math::Transform::new(
                    Vec3::ZERO,
                    scenix_math::Quat::IDENTITY,
                    Vec3::new(100.0, 100.0, 100.0),
                ),
            ),
        );
        sky_only.update_world_transforms();
        let sky_camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 200.0)
            .position(Vec3::new(0.0, 3.0, 8.0))
            .target(Vec3::new(0.0, 4.0, 0.0));
        let stats = renderer.render(&sky_only, &sky_camera)?;
        assert_eq!(stats.visible_environments, 1);
        let pixel = renderer.read_headless_pixel()?;
        // The headless target is Bgra8UnormSrgb and read_target_pixel copies
        // bytes verbatim, so mapped bytes are [blue, green, red, alpha].
        let [blue, _green, red, _alpha] = pixel;
        assert!(
            blue > 130 && blue > red + 30,
            "sky-dominant pixel expected, got bgra bytes {pixel:?}"
        );
        renderer.unregister_environment(MaterialId::new(3));
        assert_eq!(renderer.environment_count(), 2);
        Ok(())
    })
}

#[test]
fn environment_nodes_without_slots_reject_render() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(32, 32)).await?;
        let mut scene = SceneGraph::new();
        scene.add(SceneNode::terrain(
            "missing",
            TerrainId::new(1),
            MaterialId::new(9),
        ));
        scene.update_world_transforms();
        let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0);
        let error = renderer
            .render(&scene, &camera)
            .expect_err("no slot registered");
        assert!(matches!(
            error,
            ScenixError::Validation(scenix_core::ValidationError::InvalidId)
        ));
        Ok(())
    })
}

#[test]
fn register_environment_rejects_null_ids() -> Result<(), ScenixError> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(16, 16)).await?;
        let error = renderer
            .register_environment(
                MaterialId::default(),
                EnvironmentMaterial::Sky(&SkyMaterial::clear_day()),
            )
            .expect_err("null id rejected");
        assert!(matches!(
            error,
            ScenixError::Validation(scenix_core::ValidationError::InvalidId)
        ));
        assert_eq!(renderer.environment_count(), 0);
        Ok(())
    })
}
