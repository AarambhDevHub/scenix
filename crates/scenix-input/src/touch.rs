use scenix_math::Vec2;

/// Maximum simultaneous touch contacts tracked without allocating.
pub const MAX_TOUCH_POINTS: usize = 10;

/// Platform-provided touch contact identifier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TouchId(pub u64);

/// Phase of a touch event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TouchPhase {
    /// A contact started.
    Started,
    /// A contact moved.
    Moved,
    /// A contact ended normally.
    Ended,
    /// A contact was cancelled by the platform.
    Cancelled,
}

/// One active touch contact in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TouchPoint {
    /// Platform contact identifier.
    pub id: TouchId,
    /// Current logical position.
    pub position: Vec2,
    /// Movement accumulated during the current frame.
    pub delta: Vec2,
    /// Normalized pressure in `0..=1` when available.
    pub pressure: f32,
}

/// Fixed-capacity touch state suitable for `no_std` input loops.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TouchState {
    points: [Option<TouchPoint>; MAX_TOUCH_POINTS],
    len: u8,
}

impl TouchState {
    /// Creates an empty touch state.
    pub const fn new() -> Self {
        Self {
            points: [None; MAX_TOUCH_POINTS],
            len: 0,
        }
    }

    /// Number of active contacts.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns whether no contacts are active.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the contact with the given id.
    pub fn get(&self, id: TouchId) -> Option<&TouchPoint> {
        self.points.iter().flatten().find(|point| point.id == id)
    }

    /// Returns active contacts in their stable slot order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &TouchPoint> {
        self.points.iter().flatten()
    }

    /// Applies a platform touch event. Returns `false` only when a new contact
    /// cannot be stored because all fixed slots are occupied.
    pub fn on_event(
        &mut self,
        id: TouchId,
        phase: TouchPhase,
        position: Vec2,
        pressure: f32,
    ) -> bool {
        let pressure = pressure.clamp(0.0, 1.0);
        let existing = self
            .points
            .iter()
            .position(|point| point.is_some_and(|point| point.id == id));

        match phase {
            TouchPhase::Started => {
                if let Some(index) = existing {
                    self.points[index] = Some(TouchPoint {
                        id,
                        position,
                        delta: Vec2::ZERO,
                        pressure,
                    });
                    return true;
                }
                let Some(index) = self.points.iter().position(Option::is_none) else {
                    return false;
                };
                self.points[index] = Some(TouchPoint {
                    id,
                    position,
                    delta: Vec2::ZERO,
                    pressure,
                });
                self.len += 1;
                true
            }
            TouchPhase::Moved => {
                let Some(index) = existing else {
                    return false;
                };
                let point = self.points[index].as_mut().expect("occupied touch slot");
                point.delta += position - point.position;
                point.position = position;
                point.pressure = pressure;
                true
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                let Some(index) = existing else {
                    return false;
                };
                self.points[index] = None;
                self.len -= 1;
                true
            }
        }
    }

    /// Clears per-frame movement while retaining active contacts.
    pub fn end_frame(&mut self) {
        for point in self.points.iter_mut().flatten() {
            point.delta = Vec2::ZERO;
        }
    }

    /// Cancels every active contact.
    pub fn cancel_all(&mut self) {
        self.points = [None; MAX_TOUCH_POINTS];
        self.len = 0;
    }
}

impl Default for TouchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_lifecycle_accumulates_delta_and_reuses_slots() {
        let mut state = TouchState::new();
        assert!(state.on_event(TouchId(7), TouchPhase::Started, Vec2::new(1.0, 2.0), 2.0));
        assert!(state.on_event(TouchId(7), TouchPhase::Moved, Vec2::new(4.0, 6.0), 0.5));
        assert_eq!(state.get(TouchId(7)).unwrap().delta, Vec2::new(3.0, 4.0));
        assert_eq!(state.get(TouchId(7)).unwrap().pressure, 0.5);
        state.end_frame();
        assert_eq!(state.get(TouchId(7)).unwrap().delta, Vec2::ZERO);
        assert!(state.on_event(TouchId(7), TouchPhase::Ended, Vec2::ZERO, 0.0));
        assert!(state.is_empty());
    }

    #[test]
    fn capacity_is_bounded_and_cancel_clears_everything() {
        let mut state = TouchState::new();
        for id in 0..MAX_TOUCH_POINTS as u64 {
            assert!(state.on_event(TouchId(id), TouchPhase::Started, Vec2::ZERO, 0.0));
        }
        assert!(!state.on_event(TouchId(99), TouchPhase::Started, Vec2::ZERO, 0.0));
        state.cancel_all();
        assert!(state.is_empty());
    }
}
