//! XR controller input mapping (scenix v1.9).
//!
//! The [`XrControllerInput`] snapshot mirrors the WebXR `xr-standard`
//! interaction profile: an analog trigger, an analog squeeze grip, a
//! thumbstick with a pressable center, a trackpad, and the A/B/thumbrest
//! buttons. For native OpenXR targets the same vocabulary applies and the
//! host runtime copies values across.
//!
//! Hand-side vocabulary lives here so both `scenix-input` and
//! `scenix-xr` share it without crate duplication.

/// Maximum XR controllers tracked without allocating: one per hand.
pub const MAX_XR_CONTROLLERS: usize = 2;

/// Analog threshold above which an XR controller input counts as pressed.
pub const XR_PRESS_THRESHOLD: f32 = 0.5;

/// Which hand an XR controller is held in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrHand {
    /// The left-hand controller, slot `0`.
    #[default]
    Left,
    /// The right-hand controller, slot `1`.
    Right,
}

impl XrHand {
    /// Returns the dense slot index (`0` for left, `1` for right).
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    /// Returns whether this hand is the left hand.
    #[inline]
    #[must_use]
    pub const fn is_left(self) -> bool {
        matches!(self, Self::Left)
    }

    /// Returns the opposite hand.
    #[inline]
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Returns a short human-readable label used by inspector tooling.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// Per-frame tracking quality reported for an XR input source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrTrackingStatus {
    /// Pose is stale or hidden; do not trust the transform.
    #[default]
    Untracked,
    /// Pose is valid but jitter-estimated (rarely used by runtimes).
    Approximate,
    /// Pose is directly tracked this frame.
    Tracked,
}

impl XrTrackingStatus {
    /// Returns whether the pose should be consumed this frame.
    #[inline]
    #[must_use]
    pub const fn is_usable(self) -> bool {
        !matches!(self, Self::Untracked)
    }

    /// Returns a short human-readable label used by inspector tooling.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Untracked => "untracked",
            Self::Approximate => "approximate",
            Self::Tracked => "tracked",
        }
    }
}

/// Value snapshot for one XR controller, mirroring the WebXR
/// `xr-standard` interaction profile.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrControllerInput {
    /// Which hand this controller belongs to.
    pub hand: XrHand,
    /// Whether the controller is currently connected.
    pub connected: bool,
    /// Tracking quality for this frame's pose.
    pub tracking: XrTrackingStatus,
    /// Analog trigger `0..=1` (button `0` in `xr-standard`).
    pub trigger: f32,
    /// Analog squeeze/grip `0..=1` (button `1`).
    pub squeeze: f32,
    /// Trackpad `X` axis `-1..=1` (axis `2`), `NaN` when absent.
    pub trackpad_x: f32,
    /// Trackpad `Y` axis `-1..=1` (axis `3`), `NaN` when absent.
    pub trackpad_y: f32,
    /// Thumbstick `X` axis `-1..=1` (axis `0`).
    pub thumbstick_x: f32,
    /// Thumbstick `Y` axis `-1..=1` (axis `1`).
    pub thumbstick_y: f32,
    /// Trackpad pressed state (button `2`).
    pub trackpad_pressed: bool,
    /// Thumbstick pressed state (button `3`).
    pub thumbstick_pressed: bool,
    /// A/x-button pressed state (button `4`).
    pub button_a: bool,
    /// B/y-button pressed state (button `5`).
    pub button_b: bool,
    /// Thumbrest touch state (button `6`).
    pub thumbrest_touched: bool,
}

impl XrControllerInput {
    /// Creates an empty controller input for `hand`.
    #[inline]
    #[must_use]
    pub const fn new(hand: XrHand) -> Self {
        Self {
            hand,
            connected: false,
            tracking: XrTrackingStatus::Untracked,
            trigger: 0.0,
            squeeze: 0.0,
            trackpad_x: 0.0,
            trackpad_y: 0.0,
            thumbstick_x: 0.0,
            thumbstick_y: 0.0,
            trackpad_pressed: false,
            thumbstick_pressed: false,
            button_a: false,
            button_b: false,
            thumbrest_touched: false,
        }
    }

    /// Returns whether the analog trigger crosses [`XR_PRESS_THRESHOLD`].
    #[inline]
    #[must_use]
    pub fn trigger_pressed(&self) -> bool {
        self.connected && self.trigger >= XR_PRESS_THRESHOLD
    }

    /// Returns whether the analog squeeze crosses [`XR_PRESS_THRESHOLD`].
    #[inline]
    #[must_use]
    pub fn squeeze_pressed(&self) -> bool {
        self.connected && self.squeeze >= XR_PRESS_THRESHOLD
    }

    /// Applies a dead zone in `0..1` to both thumbstick axes.
    #[inline]
    pub fn apply_thumbstick_dead_zone(&mut self, dead_zone: f32) {
        self.thumbstick_x = dead_zone_axis(self.thumbstick_x, dead_zone);
        self.thumbstick_y = dead_zone_axis(self.thumbstick_y, dead_zone);
    }

    /// Fills this snapshot from a WebXR `xr-standard` gamepad payload.
    ///
    /// `buttons` indexes follow the profile order: `0` trigger, `1`
    /// squeeze, `2` trackpad, `3` thumbstick, `4` A/x, `5` B/y,
    /// `6` thumbrest, where each entry is `(value, pressed)`. Missing
    /// entries fall back to released values.
    pub fn from_xr_standard(
        &mut self,
        connected: bool,
        tracking: XrTrackingStatus,
        buttons: &[(f32, bool)],
        axes: &[f32],
    ) {
        self.connected = connected;
        self.tracking = tracking;
        let at = |i: usize| buttons.get(i).copied().unwrap_or((0.0, false));
        let trigger = at(0);
        self.trigger = trigger.0.clamp(0.0, 1.0);
        let squeeze = at(1);
        self.squeeze = squeeze.0.clamp(0.0, 1.0);
        let trackpad = at(2);
        self.trackpad_pressed = trackpad.1;
        let thumbstick = at(3);
        self.thumbstick_pressed = thumbstick.1;
        let a = at(4);
        self.button_a = a.1;
        let b = at(5);
        self.button_b = b.1;
        let thumbrest = at(6);
        self.thumbrest_touched = thumbrest.1;
        self.thumbstick_x = axes.first().copied().unwrap_or(0.0).clamp(-1.0, 1.0);
        self.thumbstick_y = axes.get(1).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
        self.trackpad_x = axes.get(2).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
        self.trackpad_y = axes.get(3).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
    }
}

/// Fixed-slot container for the two handed XR controller inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrControllerInputs {
    slots: [XrControllerInput; MAX_XR_CONTROLLERS],
}

impl XrControllerInputs {
    /// Creates an empty pair of controller inputs.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: [
                XrControllerInput::new(XrHand::Left),
                XrControllerInput::new(XrHand::Right),
            ],
        }
    }

    /// Returns the input snapshot for `hand`.
    #[inline]
    #[must_use]
    pub fn get(&self, hand: XrHand) -> &XrControllerInput {
        &self.slots[hand.index()]
    }

    /// Returns the mutable input snapshot for `hand`.
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, hand: XrHand) -> &mut XrControllerInput {
        &mut self.slots[hand.index()]
    }

    /// Returns how many slots currently report as connected.
    #[inline]
    #[must_use]
    pub const fn connected(&self) -> usize {
        let mut count = 0;
        if self.slots[0].connected {
            count += 1;
        }
        if self.slots[1].connected {
            count += 1;
        }
        count
    }
    /// Marks both slots disconnected and untracked, preserving analog
    /// values so host runtimes can smooth transitions.
    #[inline]
    pub fn reset(&mut self) {
        for slot in &mut self.slots {
            slot.connected = false;
            slot.tracking = XrTrackingStatus::Untracked;
        }
    }
}

/// Applies a symmetric dead zone to one axis value.
#[inline]
#[must_use]
fn dead_zone_axis(axis: f32, dead_zone: f32) -> f32 {
    if !axis.is_finite() {
        return 0.0;
    }
    let dz = dead_zone.clamp(0.0, 1.0);
    let magnitude = axis.abs();
    if magnitude <= dz {
        0.0
    } else {
        let center = dz + (1.0 - dz) * 0.5;
        let scaled = (magnitude - dz) / (1.0 - dz);
        axis * (center / magnitude).min(1.0) * scaled / center
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_helpers() {
        assert_eq!(XrHand::Left.index(), 0);
        assert_eq!(XrHand::Right.index(), 1);
        assert!(XrHand::Left.is_left());
        assert!(!XrHand::Right.is_left());
        assert_eq!(XrHand::Left.other(), XrHand::Right);
        assert_eq!(XrHand::Right.label(), "right");
    }

    #[test]
    fn tracking_usability() {
        assert!(!XrTrackingStatus::Untracked.is_usable());
        assert!(XrTrackingStatus::Approximate.is_usable());
        assert!(XrTrackingStatus::Tracked.is_usable());
    }

    #[test]
    fn xr_standard_mapping() {
        let mut input = XrControllerInput::new(XrHand::Right);
        input.from_xr_standard(
            true,
            XrTrackingStatus::Tracked,
            &[(0.9, true), (0.2, false), (0.0, false), (0.0, false)],
            &[0.5, -0.25],
        );
        assert!(input.connected);
        assert_eq!(input.hand, XrHand::Right);
        assert!(input.trigger_pressed());
        assert!(!input.squeeze_pressed());
        assert!((input.thumbstick_x - 0.5).abs() < 1e-6);
        assert!((input.thumbstick_y + 0.25).abs() < 1e-6);
        assert!(!input.button_a);
    }

    #[test]
    fn slots_and_reset() {
        let mut inputs = XrControllerInputs::new();
        assert_eq!(inputs.connected(), 0);
        inputs.get_mut(XrHand::Left).connected = true;
        inputs.get_mut(XrHand::Right).connected = true;
        assert_eq!(inputs.connected(), 2);
        inputs.reset();
        assert_eq!(inputs.connected(), 0);
        assert!(!inputs.get(XrHand::Right).connected);
    }

    #[test]
    fn dead_zone_kills_and_scales() {
        assert_eq!(dead_zone_axis(0.05, 0.2), 0.0);
        assert!(dead_zone_axis(1.0, 0.2).abs() <= 1.0);
    }
}
