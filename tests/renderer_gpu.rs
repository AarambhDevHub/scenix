use std::sync::Arc;

use scenix_camera::PerspectiveCamera;
use scenix_core::{Color, LightId, MaterialId, MeshId};
use scenix_light::{AmbientLight, DirectionalLight};
use scenix_material::{PbrMaterial, PipelineKey};
use scenix_math::Vec3;
use scenix_mesh::box_geometry;
use scenix_renderer::{
    PipelineCache, RenderPassKind, RenderTargetMode, Renderer, RendererConfig, RendererPipelineKey,
};
use scenix_scene::{SceneGraph, SceneNode};

fn run_gpu_tests() -> bool {
    std::env::var("SCENIX_RUN_GPU_TESTS").as_deref() == Ok("1")
}

async fn cube_renderer()
-> Result<(Renderer, SceneGraph, PerspectiveCamera), Box<dyn std::error::Error>> {
    let mut renderer = Renderer::headless(RendererConfig::new(64, 64)).await?;
    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);

    renderer.register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;
    renderer.register_pbr_material(
        material_id,
        &PbrMaterial::new()
            .albedo(Color::from_rgb(0.8, 0.2, 0.1))
            .metallic_roughness(0.0, 0.7),
    )?;
    renderer.register_ambient_light(
        LightId::new(1),
        AmbientLight::new(Color::from_rgb(0.5, 0.5, 0.5), 0.2),
    )?;
    renderer.register_directional_light(
        LightId::new(2),
        DirectionalLight::new(Vec3::new(-1.0, -2.0, -1.0), Color::WHITE, 1.0),
    )?;

    let mut scene = SceneGraph::new();
    scene.add(SceneNode::mesh("cube", mesh_id, material_id));
    scene.update_world_transforms();

    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 0.0, 4.0))
        .target(Vec3::ZERO);

    Ok((renderer, scene, camera))
}

#[test]
fn pipeline_cache_reuses_matching_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let (renderer, _, _) = cube_renderer().await?;
        let mut cache = PipelineCache::new();
        let key = RendererPipelineKey::new(
            PipelineKey::default(),
            RenderPassKind::Geometry,
            renderer.config().preferred_color_format(),
            renderer.config().sample_count,
            false,
        );

        let first = cache.get_or_create(renderer.device(), key);
        let second = cache.get_or_create(renderer.device(), key);
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(cache.len(), 1);
        Ok(())
    })
}

#[test]
fn headless_render_of_cube_produces_non_black_framebuffer() -> Result<(), Box<dyn std::error::Error>>
{
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let (mut renderer, scene, camera) = cube_renderer().await?;
        assert_eq!(renderer.target_mode(), RenderTargetMode::Headless);

        let stats = renderer.render(&scene, &camera)?;
        assert_eq!(stats.visible_meshes, 1);
        assert_eq!(stats.opaque_draws, 1);

        let pixel = renderer.read_headless_pixel()?;
        assert!(pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0);
        Ok(())
    })
}

#[test]
fn resize_recreates_offscreen_targets() -> Result<(), Box<dyn std::error::Error>> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let (mut renderer, _, _) = cube_renderer().await?;
        renderer.resize(128, 96)?;
        assert_eq!(renderer.config().width, 128);
        assert_eq!(renderer.config().height, 96);
        assert_eq!(renderer.gbuffer().width(), 128);
        assert_eq!(renderer.gbuffer().height(), 96);
        Ok(())
    })
}

#[cfg(feature = "post")]
#[test]
fn renderer_applies_optional_post_stack() -> Result<(), Box<dyn std::error::Error>> {
    if !run_gpu_tests() {
        return Ok(());
    }

    pollster::block_on(async {
        let (mut renderer, scene, camera) = cube_renderer().await?;
        renderer.set_post_stack(Some(
            scenix_post::PostStack::new()
                .with_bloom(scenix_post::BloomConfig::default())
                .with_tonemap(scenix_post::ToneMapper::Aces)
                .with_fxaa(scenix_post::FxaaConfig::default()),
        ));

        let stats = renderer.render(&scene, &camera)?;
        assert_eq!(stats.visible_meshes, 1);
        assert_eq!(renderer.post_stack().unwrap().len(), 3);

        let pixel = renderer.read_headless_pixel()?;
        assert!(pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0);
        Ok(())
    })
}
