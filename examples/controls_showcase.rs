use scenix::{
    ArcballController, FirstPersonController, GamepadAxis, GamepadId, InputState, KeyCode,
    PerspectiveCamera, PointerButton, Vec2, Vec3, ViewportMetrics,
};

fn main() {
    let mut input = InputState::new(ViewportMetrics::new(Vec2::new(1280.0, 720.0), 2.0));
    input.on_pointer_down(PointerButton::Left);
    input.on_pointer_move(Vec2::new(42.0, 18.0));
    input.on_scroll(-0.5);

    let mut camera = PerspectiveCamera::default();
    let mut arcball = ArcballController::new(Vec3::ZERO, 5.0);
    arcball.update_from_input(&input, 1.0 / 60.0);
    arcball.apply_to_perspective(&mut camera);
    println!("arcball camera: {:?}", camera.position);

    input.end_frame();
    input.on_key_down(KeyCode::KeyW);
    input.set_gamepad_connected(GamepadId(0), true);
    input.set_gamepad_axis(GamepadId(0), GamepadAxis::LeftX, 0.8);
    let mut first_person = FirstPersonController::new(camera.position);
    first_person.update_from_input(&input, 1.0 / 60.0);
    println!("first-person camera: {:?}", first_person.position);
}
