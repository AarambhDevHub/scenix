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

    #[cfg(feature = "mesh")]
    {
        let geometry = scenix::sphere_geometry(1.0, 8, 4);
        let mesh = scenix::Mesh::new(geometry, scenix::MaterialId::new(1));
        assert!(!mesh.geometry.positions.is_empty());
    }

    #[cfg(feature = "material")]
    {
        let material = scenix::PbrMaterial::new()
            .alpha_mode(facade_alpha_mode_mask())
            .double_sided(true);
        assert!(scenix::Material::double_sided(&material));
        assert_eq!(scenix::Material::alpha_cutoff(&material), Some(0.5));
    }

    #[cfg(feature = "light")]
    {
        let light = scenix::DirectionalLight::new(Vec3::new(0.0, -2.0, 0.0), Color::WHITE, 2.0)
            .shadow(facade_shadow_config());
        assert!(light.shadow.unwrap().validate().is_ok());
        assert_eq!(light.direction.length(), 1.0);
    }

    #[cfg(feature = "camera")]
    {
        let camera = scenix::PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
            .position(Vec3::new(0.0, 0.0, 5.0))
            .target(Vec3::ZERO);
        assert!(camera.frustum().contains_point(Vec3::ZERO));
    }

    #[cfg(feature = "texture")]
    {
        let texture = scenix::Texture2D::new(
            1,
            1,
            scenix::TextureFormat::Rgba8Unorm,
            vec![255, 255, 255, 255],
        )
        .unwrap();
        assert_eq!(texture.base_level_len().unwrap(), 4);
    }

    #[cfg(feature = "renderer")]
    {
        let config = scenix::RendererConfig::new(64, 64);
        assert!(config.validate().is_ok());
        let mode = scenix::RenderTargetMode::Headless;
        assert!(matches!(mode, scenix::RenderTargetMode::Headless));
    }

    #[cfg(feature = "loader")]
    {
        let loader = scenix::GltfLoader::new();
        assert!(loader.options().decode_images);
    }

    #[cfg(feature = "post")]
    {
        let stack = scenix::PostStack::new()
            .with_bloom(scenix::BloomConfig::default())
            .with_tonemap(scenix::ToneMapper::Aces);
        assert_eq!(stack.len(), 2);
    }

    assert_eq!(Mat4::IDENTITY.to_cols_array()[0], 1.0);
}

#[cfg(feature = "material")]
fn facade_alpha_mode_mask() -> scenix::AlphaMode {
    scenix::AlphaMode::Mask(0.5)
}

#[cfg(feature = "light")]
fn facade_shadow_config() -> scenix::ShadowConfig {
    scenix::ShadowConfig::new(1024, 0.1, 100.0)
}
