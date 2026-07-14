use std::hint::black_box;
use std::time::Instant;

use scenix_camera::{ArcballController, FirstPersonController};
use scenix_input::{GamepadAxis, GamepadId, InputState, KeyCode, PointerButton};
use scenix_math::{Vec2, Vec3};

fn bench(name: &str, iterations: usize, mut f: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    println!("{name}: {iterations} iterations in {:?}", start.elapsed());
}

fn main() {
    let mut orbit_input = InputState::default();
    orbit_input.on_pointer_down(PointerButton::Left);
    orbit_input.on_pointer_move(Vec2::new(3.0, -2.0));
    orbit_input.on_scroll(0.01);
    let mut arcball = ArcballController::new(Vec3::ZERO, 5.0);
    bench("arcball_input_update", 1_000_000, || {
        black_box(arcball.update_from_input(black_box(&orbit_input), 1.0 / 60.0));
    });

    let mut movement_input = InputState::default();
    movement_input.on_key_down(KeyCode::KeyW);
    movement_input.set_gamepad_connected(GamepadId(0), true);
    movement_input.set_gamepad_axis(GamepadId(0), GamepadAxis::LeftX, 0.5);
    let mut first_person = FirstPersonController::default();
    bench("first_person_input_update", 1_000_000, || {
        black_box(first_person.update_from_input(black_box(&movement_input), 1.0 / 60.0));
    });
}
