/// Maximum gamepads tracked without allocating.
pub const MAX_GAMEPADS: usize = 4;

/// A gamepad slot identifier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadId(pub u8);

/// Buttons from the standard browser/native gamepad mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum GamepadButton {
    South,
    East,
    West,
    North,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Select,
    Start,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Home,
}

/// Axes from the standard browser/native gamepad mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum GamepadAxis {
    /// Left stick horizontal axis.
    LeftX,
    /// Left stick vertical axis.
    LeftY,
    /// Right stick horizontal axis.
    RightX,
    /// Right stick vertical axis.
    RightY,
}

/// State for one standard-mapped gamepad.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadState {
    /// Whether the slot is connected.
    pub connected: bool,
    /// Axis dead zone in `0..1`.
    pub dead_zone: f32,
    axes: [f32; 4],
    buttons: [f32; 17],
}

impl GamepadState {
    /// Creates a disconnected state.
    pub const fn new() -> Self {
        Self {
            connected: false,
            dead_zone: 0.15,
            axes: [0.0; 4],
            buttons: [0.0; 17],
        }
    }

    /// Returns a dead-zone-filtered axis in `-1..=1`.
    pub fn axis(&self, axis: GamepadAxis) -> f32 {
        let value = self.axes[axis as usize];
        let dead_zone = self.dead_zone.clamp(0.0, 0.99);
        if value.abs() <= dead_zone {
            0.0
        } else {
            value.signum() * ((value.abs() - dead_zone) / (1.0 - dead_zone))
        }
    }

    /// Sets a raw axis value, clamped to `-1..=1`.
    pub fn set_axis(&mut self, axis: GamepadAxis, value: f32) {
        self.axes[axis as usize] = value.clamp(-1.0, 1.0);
    }

    /// Returns analog button pressure in `0..=1`.
    pub fn button_value(&self, button: GamepadButton) -> f32 {
        self.buttons[button as usize]
    }

    /// Returns whether the button passes a conventional pressed threshold.
    pub fn is_pressed(&self, button: GamepadButton) -> bool {
        self.button_value(button) >= 0.5
    }

    /// Sets analog button pressure, clamped to `0..=1`.
    pub fn set_button(&mut self, button: GamepadButton, value: f32) {
        self.buttons[button as usize] = value.clamp(0.0, 1.0);
    }

    /// Clears input values and marks the slot disconnected.
    pub fn disconnect(&mut self) {
        *self = Self::new();
    }
}

impl Default for GamepadState {
    fn default() -> Self {
        Self::new()
    }
}

/// Fixed-capacity collection of gamepad states.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadStates {
    pads: [GamepadState; MAX_GAMEPADS],
}

impl GamepadStates {
    /// Creates four disconnected gamepad slots.
    pub const fn new() -> Self {
        Self {
            pads: [GamepadState::new(); MAX_GAMEPADS],
        }
    }

    /// Returns a gamepad slot.
    pub fn get(&self, id: GamepadId) -> Option<&GamepadState> {
        self.pads.get(id.0 as usize)
    }

    /// Returns a mutable gamepad slot.
    pub fn get_mut(&mut self, id: GamepadId) -> Option<&mut GamepadState> {
        self.pads.get_mut(id.0 as usize)
    }

    /// Iterates connected gamepads in slot order.
    pub fn connected(&self) -> impl Iterator<Item = (GamepadId, &GamepadState)> {
        self.pads
            .iter()
            .enumerate()
            .filter(|(_, pad)| pad.connected)
            .map(|(index, pad)| (GamepadId(index as u8), pad))
    }
}

impl Default for GamepadStates {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_dead_zone_and_clamping_are_deterministic() {
        let mut pad = GamepadState::new();
        pad.set_axis(GamepadAxis::LeftX, 2.0);
        assert_eq!(pad.axis(GamepadAxis::LeftX), 1.0);
        pad.set_axis(GamepadAxis::LeftX, 0.1);
        assert_eq!(pad.axis(GamepadAxis::LeftX), 0.0);
    }
}
