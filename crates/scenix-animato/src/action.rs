//! Playback state for one clip instance inside the mixer.

extern crate alloc;

use alloc::vec::Vec;

use crate::loop_mode::LoopMode;

/// How an action's sampled values combine with other actions on the same binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BlendMode {
    /// Weighted-average blend (override). The default.
    Normal,
    /// Additive blend: `result += (sample - reference) * weight`.
    Additive,
}

impl Default for BlendMode {
    #[inline]
    fn default() -> Self {
        Self::Normal
    }
}

/// Action lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionState {
    /// Created but not yet started.
    Stopped,
    /// Advancing each tick.
    Playing,
    /// Paused; holds its local time.
    Paused,
    /// Completed (Once exhausted or max iterations reached).
    Finished,
}

impl Default for ActionState {
    #[inline]
    fn default() -> Self {
        Self::Stopped
    }
}

/// Stable handle returned by [`crate::mixer::AnimationMixer::add_action`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ActionHandle(pub usize);

/// One playing instance of a clip.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimationAction {
    /// Index into the mixer's clip table.
    pub clip_index: usize,
    /// Current local time in seconds.
    pub time: f32,
    /// Per-action time multiplier.
    pub time_scale: f32,
    /// Current blend weight in `[0, 1]`.
    pub weight: f32,
    /// Weight being faded toward (crossfade target).
    pub(crate) target_weight: f32,
    /// Per-second weight delta applied each tick (crossfade rate).
    pub(crate) weight_rate: f32,
    /// Loop behavior.
    pub loop_mode: LoopMode,
    /// Blend behavior.
    pub blend_mode: BlendMode,
    /// Lifecycle state.
    pub state: ActionState,
    /// Completed loop iterations.
    pub iteration: u32,
    /// Current play direction (flips in ping-pong).
    pub(crate) forward: bool,
    /// Marker indices not yet fired in the current pass.
    pending_markers: Vec<usize>,
    /// Clip-local start offset (sub-clip playback window start).
    pub start: f32,
    /// Clip-local end offset (`None` = clip duration).
    pub end: Option<f32>,
}

impl AnimationAction {
    /// Creates a stopped action referencing `clip_index`.
    pub fn new(clip_index: usize) -> Self {
        Self {
            clip_index,
            time: 0.0,
            time_scale: 1.0,
            weight: 1.0,
            target_weight: 1.0,
            weight_rate: 0.0,
            loop_mode: LoopMode::Once,
            blend_mode: BlendMode::Normal,
            state: ActionState::Stopped,
            iteration: 0,
            forward: true,
            pending_markers: Vec::new(),
            start: 0.0,
            end: None,
        }
    }

    /// Starts playback from `time`.
    #[inline]
    pub fn play(&mut self, time: f32) {
        self.time = time;
        self.state = ActionState::Playing;
        self.iteration = 0;
        self.forward = true;
    }

    /// Pauses playback (keeps local time).
    #[inline]
    pub fn pause(&mut self) {
        if self.state == ActionState::Playing {
            self.state = ActionState::Paused;
        }
    }

    /// Resumes playback.
    #[inline]
    pub fn resume(&mut self) {
        if self.state == ActionState::Paused {
            self.state = ActionState::Playing;
        }
    }

    /// Stops and resets the action.
    #[inline]
    pub fn stop(&mut self) {
        self.state = ActionState::Stopped;
        self.time = 0.0;
        self.iteration = 0;
        self.forward = true;
    }

    /// Sets the per-action time scale.
    #[inline]
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale;
    }

    /// Sets the loop mode.
    #[inline]
    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    /// Sets the blend mode.
    #[inline]
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    /// Sets the clip-local playback window `[start, end]`.
    #[inline]
    pub fn set_window(&mut self, start: f32, end: Option<f32>) {
        self.start = start.max(0.0);
        self.end = end;
    }

    /// Sets the current blend weight directly (clamped to `[0, 1]`).
    #[inline]
    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight.clamp(0.0, 1.0);
        self.target_weight = self.weight;
        self.weight_rate = 0.0;
    }

    /// Fades the weight to `target` over `duration` seconds (crossfade).
    pub fn fade_to(&mut self, target: f32, duration: f32) {
        self.target_weight = target.clamp(0.0, 1.0);
        if duration > 0.0 {
            self.weight_rate = (self.target_weight - self.weight) / duration;
        } else {
            self.weight = self.target_weight;
            self.weight_rate = 0.0;
        }
    }

    /// Returns whether the action is currently playing.
    #[inline]
    pub const fn is_playing(&self) -> bool {
        matches!(self.state, ActionState::Playing)
    }

    /// Returns whether the action has finished.
    #[inline]
    pub const fn is_finished(&self) -> bool {
        matches!(self.state, ActionState::Finished)
    }

    /// Returns the current play direction.
    #[inline]
    pub const fn forward(&self) -> bool {
        self.forward
    }

    /// Advances the weight fade and returns the new weight.
    #[inline]
    pub(crate) fn advance_weight(&mut self, dt: f32) -> f32 {
        if self.weight_rate != 0.0 {
            let next = self.weight + self.weight_rate * dt;
            if (self.weight_rate > 0.0 && next >= self.target_weight)
                || (self.weight_rate < 0.0 && next <= self.target_weight)
            {
                self.weight = self.target_weight;
                self.weight_rate = 0.0;
            } else {
                self.weight = next;
            }
        }
        self.weight
    }

    /// Resets the pending-marker list to all markers in `[time, end]`.
    pub(crate) fn reset_markers(&mut self, marker_count: usize) {
        self.pending_markers.clear();
        for i in 0..marker_count {
            self.pending_markers.push(i);
        }
    }

    /// Drains markers whose time is `<= time`, returning their indices in order.
    pub(crate) fn drain_markers_until(&mut self, time: f32, marker_times: &[f32]) -> Vec<usize> {
        let mut fired = Vec::new();
        let mut i = 0;
        while i < self.pending_markers.len() {
            let midx = self.pending_markers[i];
            if marker_times.get(midx).is_some_and(|&mt| mt <= time) {
                fired.push(self.pending_markers.swap_remove(i));
            } else {
                i += 1;
            }
        }
        fired
    }
}
