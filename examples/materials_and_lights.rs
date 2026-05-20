use scenix::{
    AmbientLight, Color, DirectionalLight, LightId, Material, MaterialId, Mesh, MeshId,
    PbrMaterial, SceneGraph, SceneNode, ShadowConfig, Vec3, box_geometry,
};

fn main() {
    let material_id = MaterialId::new(1);
    let mesh_id = MeshId::new(1);
    let light_id = LightId::new(1);

    let material = PbrMaterial::new()
        .albedo(Color::from_hex(0xCC_88_44).to_linear())
        .metallic_roughness(0.0, 0.55);
    let mesh = Mesh::new(box_geometry(1.0, 1.0, 1.0, 1, 1, 1), material_id);

    let sun = DirectionalLight::new(Vec3::new(-1.0, -2.0, -1.0), Color::WHITE, 3.0)
        .shadow(ShadowConfig::default());
    let ambient = AmbientLight::new(Color::WHITE, 0.15);

    let mut scene = SceneGraph::new();
    let _mesh_node = scene.add(SceneNode::mesh("cube", mesh_id, material_id));
    let _light_node = scene.add(SceneNode::light("sun", light_id));

    assert!(!mesh.geometry.positions.is_empty());
    assert!(!material.is_transparent());
    assert!(sun.shadow.unwrap().validate().is_ok());
    assert_eq!(ambient.intensity, 0.15);
}
