#![cfg_attr(not(feature = "std"), no_std)]

//! Platform-agnostic input state for scenix.

pub mod gamepad;
pub mod gesture;
pub mod keyboard;
pub mod pointer;
pub mod state;
pub mod touch;
pub mod viewport;

pub use gamepad::{
    GamepadAxis, GamepadButton, GamepadId, GamepadState, GamepadStates, MAX_GAMEPADS,
};
pub use gesture::{GestureRecognizer, GestureState};
pub use keyboard::{KeyCode, KeyboardState, Modifiers};
pub use pointer::{PointerButton, PointerState};
pub use state::{InputState, PointerLockState};
pub use touch::{MAX_TOUCH_POINTS, TouchId, TouchPhase, TouchPoint, TouchState};
pub use viewport::ViewportMetrics;
