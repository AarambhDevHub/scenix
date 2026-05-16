use scenix::{Aabb, Color, KeyCode, KeyboardState, Mat4, NodeId, Quat, Ray3, Transform, Vec3};

#[test]
fn facade_exports_foundation_api() {
    let id = NodeId::new(7);
    assert_eq!(id.get(), 7);

    let color = Color::from_hex(0x33_66_99);
    assert_eq!(color.to_hex_rgba(), 0x33_66_99_FF);

    let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))
        .rotate_by(Quat::from_axis_angle(Vec3::Y, 0.5));
    let matrix = transform.to_mat4();
    assert_eq!(matrix.mul_vec3(Vec3::ZERO), transform.translation);

    let ray = Ray3::new(Vec3::new(0.0, 0.0, 5.0), Vec3::NEG_Z);
    let aabb = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    assert!(ray.intersect_aabb(aabb).is_some());

    let mut keyboard = KeyboardState::new();
    keyboard.on_key_down(KeyCode::Space);
    assert!(keyboard.is_pressed(KeyCode::Space));

    #[cfg(feature = "scene")]
    {
        let mut scene = scenix::SceneGraph::new();
        let root = scene.add(scenix::SceneNode::group("root"));
        let child = scene
            .add_child(root, scenix::SceneNode::new("child"))
            .unwrap();
        scene.update_world_transforms();
        assert_eq!(scene.parent(child), Some(root));
        assert!(scene.world_matrix(child).is_some());
    }

    assert_eq!(Mat4::IDENTITY.to_cols_array()[0], 1.0);
}
