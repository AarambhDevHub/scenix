use scenix::{
    AmbientLight, Color, DirectionalLight, EnvironmentMap, LightId, MaterialId, MeshId,
    PbrMaterial, PerspectiveCamera, Renderer, RendererConfig, Sampler, SceneGraph, SceneNode,
    Texture2D, TextureCube, TextureFormat, TextureId, Vec3, sphere_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(320, 240)).await?;
        let mesh_id = MeshId::new(1);
        let material_id = MaterialId::new(1);
        let albedo_id = TextureId::new(1);
        let environment_id = TextureId::new(2);

        renderer.register_mesh(mesh_id, &sphere_geometry(1.0, 32, 16))?;
        renderer.register_texture2d(
            albedo_id,
            &Texture2D::new(1, 1, TextureFormat::Rgba8UnormSrgb, vec![242, 178, 89, 255])?,
            Sampler::new(),
        )?;
        let env_face = vec![96, 126, 180, 255];
        renderer.register_texture_cube(
            environment_id,
            &TextureCube::new(
                1,
                TextureFormat::Rgba8UnormSrgb,
                [
                    env_face.clone(),
                    env_face.clone(),
                    env_face.clone(),
                    env_face.clone(),
                    env_face.clone(),
                    env_face,
                ],
            )?,
            Sampler::new(),
        )?;
        renderer.set_environment_map(EnvironmentMap::new(environment_id).intensity(0.35))?;
        let mut material = PbrMaterial::new()
            .albedo(Color::WHITE)
            .metallic_roughness(0.35, 0.32);
        material.albedo_texture = Some(albedo_id);
        renderer.register_pbr_material(material_id, &material)?;
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
