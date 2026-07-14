use scenix::{
    EditorPickRequest, MaterialId, MeshId, PerspectiveCamera, Renderer, RendererConfig, SceneGraph,
    SceneNode, UnlitMaterial, Vec3, box_geometry,
};

fn main() {
    pollster::block_on(async {
        let mut renderer = match Renderer::headless(RendererConfig::new(64, 64)).await {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("headless adapter unavailable: {error}");
                return;
            }
        };
        let mesh_id = MeshId::new(1);
        let material_id = MaterialId::new(1);
        renderer
            .register_mesh(mesh_id, &box_geometry(1.0, 1.0, 1.0, 1, 1, 1))
            .unwrap();
        renderer
            .register_unlit_material(material_id, &UnlitMaterial::default())
            .unwrap();
        let mut scene = SceneGraph::new();
        scene.add(SceneNode::mesh("Picked cube", mesh_id, material_id));
        scene.update_world_transforms();
        let camera = PerspectiveCamera::default().position(Vec3::new(0.0, 0.0, 5.0));
        let result = renderer
            .pick(&scene, &camera, EditorPickRequest::new(32, 32))
            .unwrap();
        println!("pick result: {result:?}");
    });
}
