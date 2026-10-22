use scenix_animato::{LoopAdvance, LoopMode};

/// Frame grid the scrub bar snaps to.
pub const TIMELINE_SNAP_FPS: f32 = 60.0;

/// Minimum playback rate multiplier.
pub const TIMELINE_RATE_MIN: f32 = 0.05;
/// Maximum playback rate multiplier.
pub const TIMELINE_RATE_MAX: f32 = 8.0;

/// One solver tick result, mirroring the animato [`LoopAdvance`] fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineFrame {
    /// Local time in `[0, duration]`.
    pub time: f32,
    /// Iteration counter after the tick.
    pub iteration: u32,
    /// Whether a loop wrap happened this tick.
    pub wrapped: bool,
    /// Whether the play direction flipped this tick (ping-pong turn).
    pub flipped: bool,
    /// Whether playback finished (Once exhausted or max iterations).
    pub finished: bool,
}

impl From<LoopAdvance> for TimelineFrame {
    fn from(advance: LoopAdvance) -> Self {
        Self {
            time: advance.time,
            iteration: advance.iteration,
            wrapped: advance.wrapped,
            flipped: advance.flipped,
            finished: advance.finished,
        }
    }
}

/// CPU timeline preview state for one clip.
///
/// Playback delegates to animato's `LoopMode::advance` — the same exact
/// solver runtime actions run — so an editor preview cannot disagree with
/// game behavior. Scrubbing snaps to the [`TIMELINE_SNAP_FPS`] frame grid so
/// keyframe rows line up with rendered frames.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelinePreview {
    /// Clip duration in seconds.
    pub duration: f32,
    /// Playback rate multiplier (`rate > 0`).
    pub rate: f32,
    /// Whether advance moves the playhead.
    pub playing: bool,
    /// Loop wrapping behavior.
    pub loop_mode: LoopMode,
    time: f32,
    iteration: u32,
    forward: bool,
}

impl TimelinePreview {
    /// Preview over `duration` seconds, paused, repeating, at rate one.
    pub fn new(duration: f32) -> Self {
        Self {
            duration: duration.max(0.0),
            rate: 1.0,
            playing: false,
            loop_mode: LoopMode::REPEAT,
            time: 0.0,
            iteration: 0,
            forward: true,
        }
    }

    /// Current playhead (seconds).
    pub fn time(&self) -> f32 {
        self.time
    }

    /// Current iteration counter.
    pub fn iteration(&self) -> u32 {
        self.iteration
    }

    /// Current play direction (`true` = forward).
    pub fn forward(&self) -> bool {
        self.forward
    }

    /// Advance by wall-clock `delta_seconds`, scaled by the rate.
    ///
    /// Paused previews return the current playhead unchanged; finished clips
    /// stay pinned and report `finished` every tick a host asks.
    pub fn advance(&mut self, delta_seconds: f32) -> TimelineFrame {
        if !self.playing || self.duration <= 0.0 {
            return TimelineFrame {
                time: self.time,
                iteration: self.iteration,
                wrapped: false,
                flipped: false,
                finished: self.finished_now(),
            };
        }
        let advance = self.loop_mode.advance(
            self.time,
            delta_seconds * self.rate,
            self.duration,
            self.iteration,
            self.forward,
        );
        self.time = advance.time;
        self.iteration = advance.iteration;
        self.forward = if advance.flipped {
            !self.forward
        } else {
            self.forward
        };
        TimelineFrame::from(advance)
    }

    /// Scrub the playhead, snapping to the frame grid.
    pub fn scrub(&mut self, time_seconds: f32) -> TimelineFrame {
        let snapped = (time_seconds * TIMELINE_SNAP_FPS).round() / TIMELINE_SNAP_FPS;
        self.time = snapped.clamp(0.0, self.duration);
        TimelineFrame {
            time: self.time,
            iteration: self.iteration,
            wrapped: false,
            flipped: false,
            finished: false,
        }
    }

    /// Toggle playback, returning the new playing state.
    pub fn toggle_play(&mut self) -> bool {
        self.playing = !self.playing;
        self.playing
    }

    /// Set the rate, clamped into `[MIN, MAX]`.
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.clamp(TIMELINE_RATE_MIN, TIMELINE_RATE_MAX);
    }

    /// Replace the clip duration (playhead clamps into the new span).
    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration.max(0.0);
        self.time = self.time.clamp(0.0, self.duration);
    }

    /// Replace loop behavior (iteration/time state is preserved).
    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    fn finished_now(&self) -> bool {
        matches!(self.loop_mode, LoopMode::Once) && self.time >= self.duration
    }
}
