use scenix::{
    Color, MaterialId, MeshId, PbrMaterial, PerspectiveCamera, Renderer, RendererConfig,
    SceneGraph, SceneNode, Vec3, box_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(256, 256)).await?;
        let mesh_id = MeshId::new(1);
        let material_id = MaterialId::new(1);

        renderer.register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;
        renderer.register_pbr_material(
            material_id,
            &PbrMaterial::new()
                .albedo(Color::from_rgb(0.8, 0.25, 0.15))
                .metallic_roughness(0.0, 0.65),
        )?;

        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("cube", mesh_id, material_id));
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 4.0))
            .target(Vec3::ZERO);
        let stats = renderer.render(&scene, &camera)?;
        println!(
            "rendered frame {} with {} visible mesh",
            stats.frame_index, stats.visible_meshes
        );

        Ok(())
    })
}
