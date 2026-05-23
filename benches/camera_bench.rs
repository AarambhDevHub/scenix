use std::hint::black_box;
use std::time::Instant;

use scenix_camera::{FlyController, OrbitController, PerspectiveCamera};
use scenix_input::{KeyCode, KeyboardState, PointerState};
use scenix_math::{Aabb, Vec2, Vec3};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!("{name}: {iterations} iterations in {elapsed:?}");
}

fn main() {
    let camera = PerspectiveCamera::new(60.0, 16.0 / 9.0, 0.1, 1000.0)
        .position(Vec3::new(0.0, 0.0, 5.0))
        .target(Vec3::ZERO);
    let bounds = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));

    bench("camera_frustum_aabb", 100_000, || {
        let frustum = black_box(camera).frustum();
        black_box(frustum.contains_aabb(black_box(&bounds)));
    });

    bench("perspective_screen_to_ray", 100_000, || {
        black_box(black_box(camera).screen_to_ray(Vec2::ZERO));
    });

    let mut orbit = OrbitController::new(Vec3::ZERO, 8.0);
    bench("orbit_controller_drag", 100_000, || {
        orbit.on_drag(black_box(Vec2::new(1.0, -0.5)), 1.0 / 60.0);
        black_box(orbit.camera_transform());
    });

    let mut keyboard = KeyboardState::new();
    keyboard.on_key_down(KeyCode::KeyW);
    let mut pointer = PointerState::new();
    pointer.delta = Vec2::new(0.5, -0.25);
    let mut fly = FlyController::new(Vec3::ZERO);
    bench("fly_controller_update", 100_000, || {
        black_box(fly.update(black_box(keyboard), black_box(pointer), 1.0 / 60.0));
    });
}
