use scenix_camera::{
    CubeCamera, CubeFace, FlyController, Frustum, OrbitController, OrthographicCamera,
    PerspectiveCamera, Visibility,
};
use scenix_input::{KeyCode, KeyboardState, PointerState};
use scenix_math::{Aabb, Mat4, Vec2, Vec3};

fn close(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-4, "{a} != {b}");
}

#[test]
fn perspective_camera_builds_view_projection_and_center_ray() {
    let camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 100.0)
        .position(Vec3::new(0.0, 0.0, 5.0))
        .target(Vec3::ZERO);

    let view_origin = camera.view_matrix().mul_vec3(Vec3::ZERO);
    close(view_origin.z, -5.0);

    let ray = camera.screen_to_ray(Vec2::ZERO);
    close(ray.direction.x, 0.0);
    close(ray.direction.y, 0.0);
    assert!(ray.direction.z < -0.98);

    assert_ne!(camera.view_projection(), Mat4::IDENTITY);
}

#[test]
fn orthographic_camera_builds_parallel_center_ray() {
    let camera = OrthographicCamera::new(-2.0, 2.0, -1.0, 1.0, 0.1, 10.0)
        .position(Vec3::new(0.0, 0.0, 5.0))
        .target(Vec3::ZERO);
    let ray = camera.screen_to_ray(Vec2::ZERO);

    close(ray.origin.x, 0.0);
    close(ray.origin.y, 0.0);
    assert!(ray.direction.z < -0.98);
    assert!(camera.frustum().contains_point(Vec3::ZERO));
}

#[test]
fn frustum_culls_points_spheres_and_aabbs() {
    let camera = PerspectiveCamera::new(70.0, 1.0, 0.1, 20.0)
        .position(Vec3::new(0.0, 0.0, 5.0))
        .target(Vec3::ZERO);
    let frustum = camera.frustum();

    assert!(frustum.contains_point(Vec3::ZERO));
    assert!(!frustum.contains_point(Vec3::new(100.0, 0.0, 0.0)));
    assert_eq!(frustum.contains_sphere(Vec3::ZERO, 1.0), Visibility::Inside);
    assert_eq!(
        frustum.contains_sphere(Vec3::new(100.0, 0.0, 0.0), 1.0),
        Visibility::Outside
    );

    let inside = Aabb::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
    let outside = Aabb::new(
        Vec3::new(100.0, 100.0, 100.0),
        Vec3::new(101.0, 101.0, 101.0),
    );
    assert_eq!(frustum.contains_aabb(&inside), Visibility::Inside);
    assert!(!frustum.intersects_aabb(&outside));

    let identity_frustum = Frustum::from_view_projection(Mat4::IDENTITY);
    assert!(identity_frustum.contains_point(Vec3::ZERO));
}

#[test]
fn cube_camera_generates_six_face_matrices() {
    let camera = CubeCamera::new(Vec3::new(1.0, 2.0, 3.0), 0.1, 50.0);
    let matrices = camera.view_projections();

    assert_eq!(CubeFace::all().len(), 6);
    assert_ne!(matrices[0], matrices[1]);
    assert_ne!(camera.view_matrix(CubeFace::PositiveX), Mat4::IDENTITY);
}

#[test]
fn orbit_controller_clamps_polar_angle_and_distance() {
    let mut orbit = OrbitController::new(Vec3::ZERO, 50.0);
    orbit.min_distance = 2.0;
    orbit.max_distance = 10.0;
    orbit.min_polar_angle = 0.25;
    orbit.max_polar_angle = 1.0;
    orbit.phi = -10.0;
    orbit.update(0.016);

    close(orbit.distance, 10.0);
    close(orbit.phi, 0.25);

    orbit.on_drag(Vec2::new(10.0, -1000.0), 0.016);
    close(orbit.phi, 1.0);
}

#[test]
fn fly_controller_moves_with_keyboard_and_clamps_pitch() {
    let mut keyboard = KeyboardState::new();
    keyboard.on_key_down(KeyCode::KeyW);
    keyboard.on_key_down(KeyCode::ShiftLeft);

    let mut pointer = PointerState::new();
    pointer.delta = Vec2::new(0.0, -100_000.0);

    let mut fly = FlyController::new(Vec3::ZERO);
    fly.speed = 1.0;
    fly.fast_multiplier = 2.0;
    let transform = fly.update(keyboard, pointer, 1.0);

    assert!(transform.translation.length() > 0.0);
    close(fly.pitch, fly.pitch_limit);
}
