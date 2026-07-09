//! Animation events emitted by the mixer.

use alloc::string::String;
use alloc::vec::Vec;

/// A discrete event produced while advancing the mixer.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnimationEvent {
    /// An action completed a loop wrap.
    Loop {
        /// Slot index of the action.
        action: usize,
        /// New iteration counter.
        iteration: u32,
    },
    /// An action crossed a named marker.
    Marker {
        /// Slot index of the action.
        action: usize,
        /// Marker name.
        name: String,
    },
    /// An action finished (Once exhausted or max iterations reached).
    Finished {
        /// Slot index of the action.
        action: usize,
    },
}

/// Aggregate result of one [`crate::mixer::AnimationMixer::tick`].
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MixerTickResult {
    /// Events fired this tick, in deterministic order.
    pub events: Vec<AnimationEvent>,
    /// Actions that were active (playing) this tick.
    pub active_actions: usize,
    /// Actions that transitioned to `Finished` this tick.
    pub finished_actions: usize,
}
