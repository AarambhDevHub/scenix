use alloc::string::String;

use scenix_math::Vec2;

/// Largest characters retained by a [`SmallTextBuffer`].
pub const TEXT_BUFFER_CAP: usize = 48;

/// Largest widget ids tracked by one [`FocusChain`].
pub const FOCUS_CHAIN_MAX: usize = 8;

/// Default numeric step for [`Slider`] drags.
pub const SLIDER_DEFAULT_STEP: f32 = 0.01;

/// Identifier of a tool widget inside one overlay/panel.
///
/// Hosts allocate them per-app; the UI layer only ever compares ids, so a
/// plain `u16` keeps the copies free.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WidgetId(pub u16);

impl WidgetId {
    pub const FIRST: Self = Self(0);
}

/// Input-driven widget transition returned by edits.
///
/// Editors log these with replay-friendly semantics: `Button::press` yields
/// `Pressed` only on the inactive-to-active edge, so held inputs never spam
/// the history log to death.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WidgetEvent {
    Pressed,
    Toggled(bool),
    Dragged(f32),
    Typed,
    Submitted,
}

/// Latched switch state.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Toggle {
    pub on: bool,
}

impl Toggle {
    pub const OFF: Self = Self { on: false };
    pub const ON: Self = Self { on: true };

    /// Flip state, returning the resulting edge for history/telemetry logs.
    pub fn flip(&mut self) -> WidgetEvent {
        self.on = !self.on;
        WidgetEvent::Toggled(self.on)
    }
}

/// Momentary button: state, not release timing.
///
/// `press`/`release` mirror pointer down/up so overlay hosts can drive them
/// from raw pointer events without their own bookkeeping.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Button {
    pub held: bool,
}

impl Button {
    /// Press from inactive state; `Some(Pressed)` only on the edge, `None`
    /// while already held so key-repeat never floods history logs.
    pub fn press(&mut self) -> Option<WidgetEvent> {
        if self.held {
            None
        } else {
            self.held = true;
            Some(WidgetEvent::Pressed)
        }
    }

    /// Pointer released over/next to the widget.
    pub fn release(&mut self) {
        self.held = false;
    }
}

/// Numeric slider with snap-to-step, drag state, and clamped math.
///
/// All math is allocation free; `set_ratio`/`nudge` are the only mutation
/// paths so every input source records the same determinism-relevant value.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub value: f32,
    pub dragging: bool,
}

impl Slider {
    /// Slider over `[min, max]` clamped to a finite span with [`SLIDER_DEFAULT_STEP`].
    pub fn new(min: f32, max: f32) -> Self {
        let (min, max) = if min < max { (min, max) } else { (max, min) };
        Self {
            min,
            max,
            step: SLIDER_DEFAULT_STEP,
            value: min,
            dragging: false,
        }
    }

    /// Snap an arbitrary value onto the step grid inside `[min, max]`.
    ///
    /// Rounding happens without `f32::round`, which is unavailable in
    /// `no_std`: the value is clamped first, so the step index is a plain
    /// non-negative `as u32` truncation (`+ 0.5` emulates round-half-up).
    pub fn snap(&self, value: f32) -> f32 {
        let value = value.clamp(self.min, self.max);
        let span = self.max - self.min;
        let step = if self.step > 0.0 {
            self.step
        } else if span > 0.0 {
            span
        } else {
            return self.min;
        };
        let steps = ((value - self.min) / step + 0.5) as u32;
        (self.min + steps as f32 * step).min(self.max)
    }

    /// Snap a pointer position into the scalar range by dividing it against
    /// the slider's on-screen rect (host composes the rect; the math stays
    /// axis-agnostic with a single scalar `length`).
    pub fn from_ratio(&mut self, ratio: f32) -> WidgetEvent {
        self.value = self.snap(self.min + ratio.clamp(0.0, 1.0) * (self.max - self.min));
        WidgetEvent::Dragged(self.value)
    }

    /// Move the value by whole steps (keyboard), clamped into `[min, max]`.
    pub fn nudge(&mut self, steps: f32) -> WidgetEvent {
        self.value = self.snap((self.value + steps * self.step).clamp(self.min, self.max));
        WidgetEvent::Dragged(self.value)
    }
}

/// Fixed-capacity text entry buffer.
///
/// Bounded by [`TEXT_BUFFER_CAP`] so a runaway paste in an inspector row can
/// never push the overlay out of its frame budget.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmallTextBuffer {
    text: String,
    cap: usize,
}

impl SmallTextBuffer {
    /// Empty buffer with [`TEXT_BUFFER_CAP`] capacity.
    pub fn new() -> Self {
        Self::with_capacity(TEXT_BUFFER_CAP)
    }

    /// Empty buffer with a custom cap.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            text: String::with_capacity(cap.min(TEXT_BUFFER_CAP)),
            cap: cap.min(TEXT_BUFFER_CAP),
        }
    }

    /// Current text (never a copy; hosts read directly for cheap rendering).
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Append characters, dropping the rest when the cap is hit.
    pub fn push_str(&mut self, text: &str) -> WidgetEvent {
        for ch in text.chars() {
            if self.text.chars().count() >= self.cap {
                break;
            }
            self.text.push(ch);
        }
        WidgetEvent::Typed
    }

    /// Delete the trailing grapheme (backspace emulation).
    pub fn pop(&mut self) -> WidgetEvent {
        self.text.pop();
        WidgetEvent::Typed
    }

    /// Wipe the buffer.
    pub fn clear(&mut self) {
        self.text.clear();
    }
}

impl Default for SmallTextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// RGBA color preview chip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorSwatch {
    /// Red/green/blue/alpha bytes, straight through to host rendering.
    pub rgba: [u8; 4],
}

impl ColorSwatch {
    /// Unclamped bytes straight from editor color pickers.
    pub const fn from_rgba(rgba: [u8; 4]) -> Self {
        Self { rgba }
    }

    /// Linear-space float color for hosts already ambi-light aware.
    ///
    /// Components arrive in `[0,1]` linear space; the alpha byte is the last
    /// lane.
    pub fn from_linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            rgba: [to_byte(r), to_byte(g), to_byte(b), to_byte(a)],
        }
    }

    /// `#RRGGBBAA` for HTML/web overlays.
    pub fn hex(&self) -> String {
        let [r, g, b, a] = self.rgba;
        alloc::format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

fn to_byte(linear: f32) -> u8 {
    if linear.is_finite() {
        (linear.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    } else {
        0
    }
}

/// Focus-order bookkeeping for keyboard navigation across tool widgets.
///
/// `next`/`prev` cycle within the registered order only: focus never escapes
/// the chain so scripted demos can't accidentally re-target after the host
/// closes a panel.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FocusChain {
    order: [Option<WidgetId>; FOCUS_CHAIN_MAX],
    index: usize,
}

impl FocusChain {
    /// Empty chain.
    pub const fn new() -> Self {
        Self {
            order: [None; FOCUS_CHAIN_MAX],
            index: 0,
        }
    }

    /// Register (or overwrite) one slot in the chain.
    pub fn register(&mut self, index: usize, id: WidgetId) {
        if index < FOCUS_CHAIN_MAX {
            self.order[index] = Some(id);
        }
    }

    /// Wipe the chain (all panels closed).
    pub fn clear(&mut self) {
        self.order = [None; FOCUS_CHAIN_MAX];
        self.index = 0;
    }

    /// Currently focused widget, if any slot is filled.
    pub fn focused(&self) -> Option<WidgetId> {
        self.order[self.index]
    }

    /// Advance (and wrap) focus within the chain.
    pub fn focus_next(&mut self) -> Option<WidgetId> {
        self.cycle(1)
    }

    /// Retreat (and wrap) focus within the chain.
    pub fn focus_prev(&mut self) -> Option<WidgetId> {
        self.cycle(-1)
    }

    fn cycle(&mut self, direction: isize) -> Option<WidgetId> {
        if self.order.iter().all(Option::is_none) {
            return None;
        }
        loop {
            self.index =
                (self.index as isize + direction).rem_euclid(FOCUS_CHAIN_MAX as isize) as usize;
            if self.order[self.index].is_some() {
                return self.order[self.index];
            }
        }
    }
}

impl Default for FocusChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Hit-test helper: rectangle containment check a host calls before
/// forwarding pointer events into a widget.
///
/// Reuses rectangle containment so panels, sliders, and swatches all use one
/// identical deterministic rule (left/top inclusive, right/bottom exclusive).
pub fn widget_hit(point: Vec2, rect_min: Vec2, rect_size: Vec2) -> bool {
    let hx = rect_min.x <= point.x && point.x < rect_min.x + rect_size.x;
    let hy = rect_min.y <= point.y && point.y < rect_min.y + rect_size.y;
    hx && hy
}
