use scenix_input::{GamepadAxis, GamepadButton, KeyCode, PointerButton, TouchPhase};

/// Maps a DOM `KeyboardEvent.code` string to a scenix key code.
pub fn key_code_from_dom(code: &str) -> Option<KeyCode> {
    Some(match code {
        "KeyW" => KeyCode::KeyW,
        "KeyA" => KeyCode::KeyA,
        "KeyS" => KeyCode::KeyS,
        "KeyD" => KeyCode::KeyD,
        "KeyQ" => KeyCode::KeyQ,
        "KeyE" => KeyCode::KeyE,
        "Space" => KeyCode::Space,
        "ShiftLeft" => KeyCode::ShiftLeft,
        "ShiftRight" => KeyCode::ShiftRight,
        "ControlLeft" => KeyCode::ControlLeft,
        "ControlRight" => KeyCode::ControlRight,
        "AltLeft" => KeyCode::AltLeft,
        "AltRight" => KeyCode::AltRight,
        "MetaLeft" => KeyCode::MetaLeft,
        "MetaRight" => KeyCode::MetaRight,
        "ArrowUp" => KeyCode::ArrowUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,
        "Escape" => KeyCode::Escape,
        "Enter" => KeyCode::Enter,
        "Tab" => KeyCode::Tab,
        _ => return None,
    })
}

/// Maps a DOM pointer button integer to a scenix pointer button.
pub const fn pointer_button_from_dom(button: i16) -> Option<PointerButton> {
    match button {
        0 => Some(PointerButton::Left),
        1 => Some(PointerButton::Middle),
        2 => Some(PointerButton::Right),
        3 => Some(PointerButton::Back),
        4 => Some(PointerButton::Forward),
        _ => None,
    }
}

/// Maps a compact browser touch phase code (`0..=3`) to scenix.
pub const fn touch_phase_from_dom(phase: u8) -> Option<TouchPhase> {
    match phase {
        0 => Some(TouchPhase::Started),
        1 => Some(TouchPhase::Moved),
        2 => Some(TouchPhase::Ended),
        3 => Some(TouchPhase::Cancelled),
        _ => None,
    }
}

/// Maps a standard gamepad axis index to scenix.
pub const fn gamepad_axis_from_standard(axis: u8) -> Option<GamepadAxis> {
    match axis {
        0 => Some(GamepadAxis::LeftX),
        1 => Some(GamepadAxis::LeftY),
        2 => Some(GamepadAxis::RightX),
        3 => Some(GamepadAxis::RightY),
        _ => None,
    }
}

/// Maps a standard gamepad button index to scenix.
pub const fn gamepad_button_from_standard(button: u8) -> Option<GamepadButton> {
    Some(match button {
        0 => GamepadButton::South,
        1 => GamepadButton::East,
        2 => GamepadButton::West,
        3 => GamepadButton::North,
        4 => GamepadButton::LeftBumper,
        5 => GamepadButton::RightBumper,
        6 => GamepadButton::LeftTrigger,
        7 => GamepadButton::RightTrigger,
        8 => GamepadButton::Select,
        9 => GamepadButton::Start,
        10 => GamepadButton::LeftStick,
        11 => GamepadButton::RightStick,
        12 => GamepadButton::DPadUp,
        13 => GamepadButton::DPadDown,
        14 => GamepadButton::DPadLeft,
        15 => GamepadButton::DPadRight,
        16 => GamepadButton::Home,
        _ => return None,
    })
}
