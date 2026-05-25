use scenix::{
    Color, DirectionalLight, LightId, MaterialId, MeshId, PbrMaterial, PerspectiveCamera, Renderer,
    RendererConfig, SceneGraph, SceneNode, ShadowConfig, Transform, Vec3, box_geometry,
    plane_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(320, 240)).await?;
        let cube_mesh = MeshId::new(1);
        let floor_mesh = MeshId::new(2);
        let cube_material = MaterialId::new(1);
        let floor_material = MaterialId::new(2);

        renderer.register_mesh(cube_mesh, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;
        renderer.register_mesh(floor_mesh, &plane_geometry(6.0, 6.0, 1, 1))?;
        renderer.register_pbr_material(
            cube_material,
            &PbrMaterial::new()
                .albedo(Color::from_rgb(0.2, 0.55, 0.85))
                .metallic_roughness(0.0, 0.45),
        )?;
        renderer.register_pbr_material(
            floor_material,
            &PbrMaterial::new()
                .albedo(Color::from_rgb(0.6, 0.62, 0.58))
                .metallic_roughness(0.0, 0.9),
        )?;
        renderer.register_directional_light(
            LightId::new(1),
            DirectionalLight::new(Vec3::new(-0.4, -1.0, -0.25), Color::WHITE, 4.0)
                .shadow(ShadowConfig::new(1024, 0.1, 50.0)),
        )?;

        let mut scene = SceneGraph::new();
        scene.add(
            SceneNode::mesh("cube", cube_mesh, cube_material)
                .transform(Transform::from_translation(Vec3::new(0.0, 0.75, 0.0))),
        );
        scene.add(
            SceneNode::mesh("floor", floor_mesh, floor_material)
                .transform(Transform::from_translation(Vec3::new(0.0, -0.01, 0.0))),
        );
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(55.0, 320.0 / 240.0, 0.1, 100.0)
            .position(Vec3::new(3.0, 2.2, 4.0))
            .target(Vec3::new(0.0, 0.4, 0.0));
        let stats = renderer.render(&scene, &camera)?;
        println!(
            "shadow demo: {} visible meshes, {} light",
            stats.visible_meshes, stats.lights
        );

        Ok(())
    })
}
