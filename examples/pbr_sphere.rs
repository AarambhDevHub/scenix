use scenix::{
    AmbientLight, Color, DirectionalLight, LightId, MaterialId, MeshId, PbrMaterial,
    PerspectiveCamera, Renderer, RendererConfig, SceneGraph, SceneNode, Vec3, sphere_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(320, 240)).await?;
        let mesh_id = MeshId::new(1);
        let material_id = MaterialId::new(1);

        renderer.register_mesh(mesh_id, &sphere_geometry(1.0, 32, 16))?;
        renderer.register_pbr_material(
            material_id,
            &PbrMaterial::new()
                .albedo(Color::from_rgb(0.95, 0.7, 0.35))
                .metallic_roughness(0.35, 0.32),
        )?;
        renderer.register_ambient_light(
            LightId::new(1),
            AmbientLight::new(Color::from_rgb(0.55, 0.62, 0.7), 0.25),
        )?;
        renderer.register_directional_light(
            LightId::new(2),
            DirectionalLight::new(Vec3::new(-0.6, -1.0, -0.4), Color::WHITE, 3.0),
        )?;

        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("sphere", mesh_id, material_id));
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(50.0, 320.0 / 240.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 3.5))
            .target(Vec3::ZERO);
        let stats = renderer.render(&scene, &camera)?;
        println!(
            "pbr sphere: {} opaque draw, {} lights",
            stats.opaque_draws, stats.lights
        );

        Ok(())
    })
}
