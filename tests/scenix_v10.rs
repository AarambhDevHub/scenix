#[test]
fn stable_default_facade_exports_raycasting_and_helpers() {
    let grid = scenix::GridHelper::new(2.0, 2).to_geometry();
    assert!(grid.validate().is_ok());

    let axes = scenix::AxesHelper::new(1.0).to_geometry();
    assert_eq!(axes.positions.len(), 6);

    let camera = scenix::PerspectiveCamera::new(60.0, 1.0, 0.1, 100.0)
        .position(scenix::Vec3::new(0.0, 0.0, 4.0))
        .target(scenix::Vec3::ZERO);
    let ray = scenix::Raycaster::from_camera_ndc(&camera, scenix::Vec2::ZERO);
    assert!(ray.direction.z < -0.9);
}

#[cfg(feature = "renderer")]
#[test]
fn stable_facade_exports_renderer_material_registration_api() {
    let mut gpu_scene = scenix::GpuScene::new();
    gpu_scene
        .register_physical_material(scenix::MaterialId::new(1), &scenix::PhysicalMaterial::new())
        .unwrap();
    gpu_scene
        .register_toon_material(scenix::MaterialId::new(2), &scenix::ToonMaterial::new())
        .unwrap();
    gpu_scene
        .register_wireframe_material(
            scenix::MaterialId::new(3),
            &scenix::WireframeMaterial::new(),
        )
        .unwrap();
    gpu_scene
        .register_normal_material(scenix::MaterialId::new(4), &scenix::NormalMaterial::new())
        .unwrap();

    assert_eq!(gpu_scene.material_count(), 4);
}

#[cfg(feature = "wasm")]
#[test]
fn stable_facade_exports_wasm_demo_helpers() {
    assert_eq!(scenix::clamp_canvas_size(0, 0), (1, 1));
    assert_eq!(
        scenix::pointer_button_from_dom(0),
        Some(scenix::PointerButton::Left)
    );
    assert_eq!(
        scenix::BrowserBackendPreference::Auto,
        scenix::BrowserBackendPreference::Auto
    );
    assert_eq!(
        scenix::BrowserBackendKind::WebGl,
        scenix::BrowserBackendKind::WebGl
    );
}
