use scenix::{
    Color, MaterialId, MeshId, PbrMaterial, PerspectiveCamera, RenderTargetDescriptor, Renderer,
    RendererConfig, SceneGraph, SceneNode, TextureId, Vec3, box_geometry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let mut renderer = Renderer::headless(RendererConfig::new(128, 128)).await?;
        let target_id = TextureId::new(99);
        renderer.create_render_target(
            target_id,
            RenderTargetDescriptor::color(128, 128, scenix::wgpu::TextureFormat::Rgba8UnormSrgb),
        )?;

        let mesh_id = MeshId::new(1);
        let material_id = MaterialId::new(1);
        renderer.register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))?;
        renderer.register_pbr_material(
            material_id,
            &PbrMaterial::new()
                .albedo(Color::from_hex(0x4EA1FF))
                .metallic_roughness(0.0, 0.5),
        )?;

        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("capture cube", mesh_id, material_id));
        scene.update_world_transforms();

        let camera = PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 4.0))
            .target(Vec3::ZERO);
        renderer.render_to_texture(target_id, &scene, &camera)?;
        let pixel = renderer.read_texture_pixel(target_id, 0, 0)?;
        println!("captured pixel rgba={pixel:?}");

        Ok(())
    })
}
