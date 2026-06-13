use scenix::{
    Color, MaterialId, MeshId, PerspectiveCamera, Renderer, RendererConfig, Sampler, SceneGraph,
    SceneNode, Texture2D, TextureFormat, TextureId, ToonMaterial, Vec3, sphere_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(320, 240)).await?;
        let mesh_id = MeshId::new(1);
        let material_id = MaterialId::new(1);
        let gradient_id = TextureId::new(1);
        let mut material = ToonMaterial::new().steps(4).outline(0.03, Color::BLACK);
        material.color = Color::from_hex(0xFFCC66);
        material.gradient_map = Some(gradient_id);

        renderer.register_mesh(mesh_id, &sphere_geometry(1.0, 32, 16))?;
        renderer.register_texture2d(
            gradient_id,
            &Texture2D::new(
                4,
                1,
                TextureFormat::Rgba8UnormSrgb,
                vec![
                    35, 30, 24, 255, 110, 88, 52, 255, 215, 160, 68, 255, 255, 226, 144, 255,
                ],
            )?,
            Sampler::new(),
        )?;
        renderer.register_toon_material(material_id, &material)?;

        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("toon sphere", mesh_id, material_id));
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(55.0, 320.0 / 240.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 3.2))
            .target(Vec3::ZERO);
        let stats = renderer.render(&scene, &camera)?;
        println!(
            "toon shading rendered {} visible mesh",
            stats.visible_meshes
        );

        Ok(())
    })
}
