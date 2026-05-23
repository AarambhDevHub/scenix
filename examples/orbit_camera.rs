use scenix::{OrbitController, PerspectiveCamera, Vec2, Vec3};

fn main() {
    let mut camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 500.0);
    let mut orbit = OrbitController::new(Vec3::ZERO, 8.0);

    orbit.min_distance = 2.0;
    orbit.max_distance = 20.0;
    orbit.on_drag(Vec2::new(24.0, -12.0), 1.0 / 60.0);
    orbit.on_scroll(-0.5, 1.0 / 60.0);
    orbit.apply_to_perspective(&mut camera);

    assert!(camera.position.distance(orbit.target) >= orbit.min_distance);
    assert!(camera.frustum().contains_point(Vec3::ZERO));
}
