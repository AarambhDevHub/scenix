use scenix_math::Vec2;

use crate::{
    GamepadAxis, GamepadButton, GamepadId, GamepadStates, GestureRecognizer, GestureState, KeyCode,
    KeyboardState, PointerButton, PointerState, TouchId, TouchPhase, TouchState, ViewportMetrics,
};

/// Pointer-lock state and relative movement accumulated this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointerLockState {
    /// Whether the platform currently owns pointer lock.
    pub locked: bool,
    /// Relative movement reported while locked.
    pub delta: Vec2,
}

/// Complete platform-independent input snapshot.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InputState {
    /// Keyboard held state.
    pub keyboard: KeyboardState,
    /// Pointer position, movement, and held buttons.
    pub pointer: PointerState,
    /// Active touch contacts.
    pub touches: TouchState,
    /// Current-frame recognized gesture.
    pub gesture: GestureState,
    /// Fixed gamepad slots.
    pub gamepads: GamepadStates,
    /// Pointer-lock state.
    pub pointer_lock: PointerLockState,
    /// Logical and physical viewport measurements.
    pub viewport: ViewportMetrics,
    /// Accumulated scroll-wheel delta.
    pub scroll_delta: f32,
    gesture_recognizer: GestureRecognizer,
    key_pressed: u128,
    key_released: u128,
    pointer_pressed: u8,
    pointer_released: u8,
}

impl InputState {
    /// Creates an empty input snapshot for the given viewport.
    pub fn new(viewport: ViewportMetrics) -> Self {
        Self {
            keyboard: KeyboardState::new(),
            pointer: PointerState::new(),
            touches: TouchState::new(),
            gesture: GestureState::default(),
            gamepads: GamepadStates::new(),
            pointer_lock: PointerLockState::default(),
            viewport,
            scroll_delta: 0.0,
            gesture_recognizer: GestureRecognizer::new(),
            key_pressed: 0,
            key_released: 0,
            pointer_pressed: 0,
            pointer_released: 0,
        }
    }

    /// Applies a keyboard press, recording its transition once per frame.
    pub fn on_key_down(&mut self, key: KeyCode) {
        if !self.keyboard.is_pressed(key) {
            self.key_pressed |= 1_u128 << key as u8;
        }
        self.keyboard.on_key_down(key);
    }

    /// Applies a keyboard release.
    pub fn on_key_up(&mut self, key: KeyCode) {
        if self.keyboard.is_pressed(key) {
            self.key_released |= 1_u128 << key as u8;
        }
        self.keyboard.on_key_up(key);
    }

    /// Returns whether the key transitioned down this frame.
    pub const fn was_key_pressed(&self, key: KeyCode) -> bool {
        self.key_pressed & (1_u128 << key as u8) != 0
    }

    /// Returns whether the key transitioned up this frame.
    pub const fn was_key_released(&self, key: KeyCode) -> bool {
        self.key_released & (1_u128 << key as u8) != 0
    }

    /// Accumulates an absolute logical pointer movement event.
    pub fn on_pointer_move(&mut self, position: Vec2) {
        self.pointer.delta += position - self.pointer.position;
        self.pointer.position = position;
    }

    /// Accumulates relative movement while pointer lock is active.
    pub fn on_pointer_motion(&mut self, delta: Vec2) {
        if self.pointer_lock.locked {
            self.pointer_lock.delta += delta;
        }
    }

    /// Applies a pointer button press.
    pub fn on_pointer_down(&mut self, button: PointerButton) {
        if !self.pointer.is_pressed(button) {
            self.pointer_pressed |= 1 << button as u8;
        }
        self.pointer.on_button_down(button);
    }

    /// Applies a pointer button release.
    pub fn on_pointer_up(&mut self, button: PointerButton) {
        if self.pointer.is_pressed(button) {
            self.pointer_released |= 1 << button as u8;
        }
        self.pointer.on_button_up(button);
    }

    /// Returns whether a pointer button transitioned down this frame.
    pub const fn was_pointer_pressed(&self, button: PointerButton) -> bool {
        self.pointer_pressed & (1 << button as u8) != 0
    }

    /// Returns whether a pointer button transitioned up this frame.
    pub const fn was_pointer_released(&self, button: PointerButton) -> bool {
        self.pointer_released & (1 << button as u8) != 0
    }

    /// Accumulates a scroll-wheel event.
    pub fn on_scroll(&mut self, delta: f32) {
        if delta.is_finite() {
            self.scroll_delta += delta;
        }
    }

    /// Applies a touch event and refreshes the current gesture.
    pub fn on_touch(
        &mut self,
        id: TouchId,
        phase: TouchPhase,
        position: Vec2,
        pressure: f32,
    ) -> bool {
        let accepted = self.touches.on_event(id, phase, position, pressure);
        if accepted {
            self.gesture = self.gesture_recognizer.update(&self.touches);
        }
        accepted
    }

    /// Sets pointer-lock state and clears stale relative movement.
    pub fn set_pointer_locked(&mut self, locked: bool) {
        if self.pointer_lock.locked != locked {
            self.pointer_lock.delta = Vec2::ZERO;
        }
        self.pointer_lock.locked = locked;
    }

    /// Sets one gamepad connection state.
    pub fn set_gamepad_connected(&mut self, id: GamepadId, connected: bool) -> bool {
        let Some(pad) = self.gamepads.get_mut(id) else {
            return false;
        };
        if connected {
            pad.connected = true;
        } else {
            pad.disconnect();
        }
        true
    }

    /// Sets one standard gamepad axis.
    pub fn set_gamepad_axis(&mut self, id: GamepadId, axis: GamepadAxis, value: f32) -> bool {
        let Some(pad) = self.gamepads.get_mut(id) else {
            return false;
        };
        pad.set_axis(axis, value);
        true
    }

    /// Sets one standard gamepad button.
    pub fn set_gamepad_button(&mut self, id: GamepadId, button: GamepadButton, value: f32) -> bool {
        let Some(pad) = self.gamepads.get_mut(id) else {
            return false;
        };
        pad.set_button(button, value);
        true
    }

    /// Clears transient deltas and transitions while retaining held state.
    pub fn end_frame(&mut self) {
        self.pointer.clear_delta();
        self.touches.end_frame();
        self.gesture.clear();
        self.pointer_lock.delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
        self.key_pressed = 0;
        self.key_released = 0;
        self.pointer_pressed = 0;
        self.pointer_released = 0;
    }

    /// Clears all held and transient input, such as after focus loss.
    pub fn clear(&mut self) {
        let viewport = self.viewport;
        *self = Self::new(viewport);
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new(ViewportMetrics::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_frame_preserves_held_state_only() {
        let mut input = InputState::default();
        input.on_key_down(KeyCode::KeyW);
        input.on_pointer_down(PointerButton::Left);
        input.on_pointer_move(Vec2::new(4.0, 5.0));
        input.on_scroll(2.0);
        assert!(input.was_key_pressed(KeyCode::KeyW));
        input.end_frame();
        assert!(input.keyboard.is_pressed(KeyCode::KeyW));
        assert!(input.pointer.is_pressed(PointerButton::Left));
        assert_eq!(input.pointer.delta, Vec2::ZERO);
        assert_eq!(input.scroll_delta, 0.0);
        assert!(!input.was_key_pressed(KeyCode::KeyW));
    }

    #[test]
    fn pointer_lock_gates_relative_movement() {
        let mut input = InputState::default();
        input.on_pointer_motion(Vec2::ONE);
        assert_eq!(input.pointer_lock.delta, Vec2::ZERO);
        input.set_pointer_locked(true);
        input.on_pointer_motion(Vec2::ONE);
        assert_eq!(input.pointer_lock.delta, Vec2::ONE);
    }
}
