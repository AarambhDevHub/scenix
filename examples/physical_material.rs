use scenix::{
    Color, MaterialId, MeshId, PbrMaterial, PerspectiveCamera, PhysicalMaterial, Renderer,
    RendererConfig, SceneGraph, SceneNode, Vec3, sphere_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(320, 240)).await?;
        let mesh_id = MeshId::new(1);
        let material_id = MaterialId::new(1);

        renderer.register_mesh(mesh_id, &sphere_geometry(1.0, 40, 20))?;
        renderer.register_physical_material(
            material_id,
            &PhysicalMaterial::new()
                .base(
                    PbrMaterial::new()
                        .albedo(Color::from_hex(0x2F80FF))
                        .metallic_roughness(0.65, 0.18),
                )
                .clearcoat(0.8, 0.12)
                .sheen(0.2, Color::from_hex(0x99CCFF), 0.45),
        )?;

        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("clearcoat sphere", mesh_id, material_id));
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(50.0, 320.0 / 240.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 3.4))
            .target(Vec3::ZERO);
        let stats = renderer.render(&scene, &camera)?;
        println!("physical material rendered {} draw", stats.opaque_draws);

        Ok(())
    })
}
