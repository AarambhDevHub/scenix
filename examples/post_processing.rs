use scenix::{
    BloomConfig, Color, FxaaConfig, LightId, MaterialId, MeshId, PerspectiveCamera, PostStack,
    Renderer, RendererConfig, SceneGraph, SceneNode, ScenixError, SsaoConfig, TaaConfig,
    ToneMapper, Vec3, box_geometry,
};

fn main() -> Result<(), ScenixError> {
    pollster::block_on(run())
}

async fn run() -> Result<(), ScenixError> {
    let mut renderer = Renderer::headless(RendererConfig::new(256, 256)).await?;
    renderer.set_post_stack(Some(
        PostStack::new()
            .with_ssao(SsaoConfig::default())
            .with_bloom(BloomConfig::default())
            .with_tonemap(ToneMapper::Aces)
            .with_fxaa(FxaaConfig::default())
            .with_taa(TaaConfig::default()),
    ));

    let mesh_id = MeshId::new(1);
    let material_id = MaterialId::new(1);
    renderer.register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;
    renderer.register_pbr_material(
        material_id,
        &scenix::PbrMaterial::new()
            .albedo(Color::from_rgb(0.15, 0.45, 0.9))
            .metallic_roughness(0.0, 0.55),
    )?;
    renderer.register_ambient_light(
        LightId::new(1),
        scenix::AmbientLight::new(Color::WHITE, 0.3),
    )?;

    let mut scene = SceneGraph::new();
    scene.add(SceneNode::mesh("post-cube", mesh_id, material_id));
    scene.update_world_transforms();

    let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 0.0, 4.0))
        .target(Vec3::ZERO);
    let stats = renderer.render(&scene, &camera)?;
    println!(
        "post stack rendered {} visible mesh with {} effects",
        stats.visible_meshes,
        renderer.post_stack().map_or(0, PostStack::len)
    );

    Ok(())
}
