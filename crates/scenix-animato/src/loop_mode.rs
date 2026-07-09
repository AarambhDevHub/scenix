//! Clip loop modes.

/// How an action wraps at the end of its clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoopMode {
    /// Play once and stop; the action becomes `Finished` after the last frame.
    Once,
    /// Repeat forever, or up to `max` iterations (`0` means unlimited).
    Repeat { max: u32 },
    /// Alternate forward/backward, up to `max` half-iterations (`0` = unlimited).
    PingPong { max: u32 },
}

impl LoopMode {
    /// Default `Repeat` (unlimited).
    pub const REPEAT: Self = Self::Repeat { max: 0 };
    /// Default `PingPong` (unlimited).
    pub const PING_PONG: Self = Self::PingPong { max: 0 };
}

impl Default for LoopMode {
    #[inline]
    fn default() -> Self {
        Self::Once
    }
}

/// Result of advancing an action clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopAdvance {
    /// New local time in `[0, duration]`.
    pub time: f32,
    /// Whether the play direction flipped (ping-pong turn).
    pub flipped: bool,
    /// Whether a loop wrap occurred this tick.
    pub wrapped: bool,
    /// New iteration counter.
    pub iteration: u32,
    /// Whether the action has finished (Once exhausted or max iters reached).
    pub finished: bool,
}

impl LoopMode {
    /// Advances a clock by `delta` seconds within `duration`.
    ///
    /// Deterministic: multiple wraps in one large delta are resolved fully so
    /// the resulting `iteration` counter is exact.
    pub fn advance(
        self,
        time: f32,
        delta: f32,
        duration: f32,
        iteration: u32,
        forward: bool,
    ) -> LoopAdvance {
        if duration <= 0.0 {
            return LoopAdvance {
                time: 0.0,
                flipped: false,
                wrapped: false,
                iteration,
                finished: true,
            };
        }
        let mut t = time + if forward { delta } else { -delta };
        let mut it = iteration;
        let mut fwd = forward;
        let mut wrapped = false;
        let mut finished = false;
        let mut guard = 0u32;

        // Resolve wraps; cap iterations to avoid pathological infinite loops.
        loop {
            guard += 1;
            if guard > 1_000_000 {
                finished = true;
                t = t.clamp(0.0, duration);
                break;
            }
            if t > duration {
                match self {
                    LoopMode::Once => {
                        t = duration;
                        finished = true;
                        break;
                    }
                    LoopMode::Repeat { max } => {
                        t -= duration;
                        it += 1;
                        wrapped = true;
                        if max > 0 && it >= max {
                            t = duration;
                            finished = true;
                            break;
                        }
                    }
                    LoopMode::PingPong { max } => {
                        t = duration - (t - duration);
                        fwd = !fwd;
                        wrapped = true;
                        it += 1;
                        if max > 0 && it >= max {
                            t = 0.0;
                            finished = true;
                            break;
                        }
                    }
                }
            } else if t < 0.0 {
                // Only reachable in ping-pong backward phase.
                t = -t;
                fwd = !fwd;
                wrapped = true;
                match self {
                    LoopMode::PingPong { max } => {
                        it += 1;
                        if max > 0 && it >= max {
                            t = 0.0;
                            finished = true;
                            break;
                        }
                    }
                    _ => {
                        // Should not happen for Once/Repeat backward, but clamp.
                        t = t.clamp(0.0, duration);
                        break;
                    }
                }
            } else {
                break;
            }
        }

        LoopAdvance {
            time: t,
            flipped: fwd != forward,
            wrapped,
            iteration: it,
            finished,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_finishes_at_end() {
        let r = LoopMode::Once.advance(0.8, 0.4, 1.0, 0, true);
        assert!(r.finished);
        assert_eq!(r.time, 1.0);
    }

    #[test]
    fn repeat_wraps_and_counts() {
        let r = LoopMode::REPEAT.advance(0.8, 0.4, 1.0, 0, true);
        assert!(r.wrapped);
        assert_eq!(r.iteration, 1);
        assert!((r.time - 0.2).abs() < 1e-4);
        assert!(!r.finished);
    }

    #[test]
    fn ping_pong_flips_direction() {
        let r = LoopMode::PING_PONG.advance(0.8, 0.4, 1.0, 0, true);
        assert!(r.flipped);
        assert!((r.time - 0.8).abs() < 1e-4);
    }
}
