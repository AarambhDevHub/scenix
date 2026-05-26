use scenix_input::{KeyCode, PointerButton};
use scenix_wasm::{clamp_canvas_size, key_code_from_dom, pointer_button_from_dom};

#[test]
fn dom_key_codes_map_to_input_codes() {
    assert_eq!(key_code_from_dom("KeyW"), Some(KeyCode::KeyW));
    assert_eq!(key_code_from_dom("ShiftRight"), Some(KeyCode::ShiftRight));
    assert_eq!(key_code_from_dom("ArrowLeft"), Some(KeyCode::ArrowLeft));
    assert_eq!(key_code_from_dom("Unknown"), None);
}

#[test]
fn dom_pointer_buttons_map_to_input_buttons() {
    assert_eq!(pointer_button_from_dom(0), Some(PointerButton::Left));
    assert_eq!(pointer_button_from_dom(1), Some(PointerButton::Middle));
    assert_eq!(pointer_button_from_dom(2), Some(PointerButton::Right));
    assert_eq!(pointer_button_from_dom(4), Some(PointerButton::Forward));
    assert_eq!(pointer_button_from_dom(99), None);
}

#[test]
fn zero_canvas_size_is_clamped() {
    assert_eq!(clamp_canvas_size(0, 0), (1, 1));
    assert_eq!(clamp_canvas_size(640, 0), (640, 1));
    assert_eq!(clamp_canvas_size(0, 480), (1, 480));
}
