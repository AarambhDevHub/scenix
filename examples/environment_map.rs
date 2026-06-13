use scenix::{
    Color, EnvironmentMap, LightId, MaterialId, MeshId, PbrMaterial, PerspectiveCamera, Renderer,
    RendererConfig, Sampler, SceneGraph, SceneNode, TextureCube, TextureFormat, TextureId, Vec3,
    sphere_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(320, 240)).await?;
        let mesh_id = MeshId::new(1);
        let material_id = MaterialId::new(1);
        let environment_id = TextureId::new(1);

        let faces = [
            vec![120, 168, 255, 255],
            vec![88, 126, 210, 255],
            vec![180, 210, 255, 255],
            vec![34, 38, 54, 255],
            vec![96, 150, 230, 255],
            vec![70, 96, 156, 255],
        ];
        renderer.register_texture_cube(
            environment_id,
            &TextureCube::new(1, TextureFormat::Rgba8UnormSrgb, faces)?,
            Sampler::new(),
        )?;
        renderer.set_environment_map(
            EnvironmentMap::new(environment_id)
                .intensity(0.45)
                .light_probe(LightId::new(1)),
        )?;

        renderer.register_mesh(mesh_id, &sphere_geometry(1.0, 48, 24))?;
        renderer.register_pbr_material(
            material_id,
            &PbrMaterial::new()
                .albedo(Color::from_hex(0xDDEBFF))
                .metallic_roughness(0.85, 0.18),
        )?;

        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("environment sphere", mesh_id, material_id));
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(50.0, 320.0 / 240.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 3.5))
            .target(Vec3::ZERO);
        let stats = renderer.render(&scene, &camera)?;
        println!(
            "environment map rendered {} draw with {} texture",
            stats.opaque_draws,
            renderer.diagnostics().textures
        );

        Ok(())
    })
}
