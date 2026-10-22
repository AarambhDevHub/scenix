use core::fmt::Write as _;

use alloc::string::String;

/// Number of frames kept per rolling window.
///
/// 120 frames covers roughly two seconds at 60 FPS, long enough to smooth
/// loading spikes and short enough that the mean reflects the last visible
/// behavior in overlays.
pub const STATS_HISTORY_FRAMES: usize = 120;

/// Per-frame sample recorded by runtime hosts and inside an overlay.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameSample {
    /// Wall time of the frame in milliseconds.
    pub frame_ms: f32,
    /// Draw calls submitted during the frame.
    pub draw_calls: u32,
    /// Triangles drawn during the frame.
    pub triangles: u32,
    /// Bytes uploaded (buffers, textures, uniforms) during the frame.
    pub resource_bytes: u64,
}

impl FrameSample {
    /// Zero-sample: a warmup frame produces no counters.
    pub const ZERO: Self = Self {
        frame_ms: 0.0,
        draw_calls: 0,
        triangles: 0,
        resource_bytes: 0,
    };

    /// Non-finite wall times (debugger pauses, host timers) collapse to zero
    /// so statistics never poison the mean.
    pub fn sanitized(frame_ms: f32, draw_calls: u32, triangles: u32, resource_bytes: u64) -> Self {
        Self {
            frame_ms: if frame_ms.is_finite() && frame_ms > 0.0 {
                frame_ms
            } else {
                0.0
            },
            draw_calls,
            triangles,
            resource_bytes,
        }
    }
}

/// Static thresholds an overlay checks every window against.
///
/// Defaults describe a mixed 60 FPS desktop/mobile target: 16.7 ms frames,
/// 1000 draw calls, 2 million triangles, and 64 MiB of streaming uploads per
/// frame. Hosts override them per platform.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BudgetPolicy {
    /// Per-frame wall-time budget in milliseconds.
    pub max_frame_ms: f32,
    /// Mean draw-calls-per-frame budget.
    pub max_mean_draw_calls: u32,
    /// Mean triangles-per-frame budget.
    pub max_mean_triangles: u32,
    /// Mean per-frame upload budget in bytes.
    pub max_mean_upload_bytes: u64,
}

impl BudgetPolicy {
    /// 60 Hz mixed-target budget.
    pub const SIXTY_HZ: Self = Self {
        max_frame_ms: 16.67,
        max_mean_draw_calls: 1_000,
        max_mean_triangles: 2_000_000,
        max_mean_upload_bytes: 64 * 1024 * 1024,
    };

    /// 30 Hz battery/budget device policy.
    pub const THIRTY_HZ: Self = Self {
        max_frame_ms: 33.3,
        max_mean_draw_calls: 2_000,
        max_mean_triangles: 4_000_000,
        max_mean_upload_bytes: 128 * 1024 * 1024,
    };
}

/// Which static budget a window has exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BudgetWarning {
    FrameTime,
    DrawCalls,
    Triangles,
    ResourceUploads,
}

/// CPU frame statistics for a runtime window.
///
/// Fixed-capacity ring over [`STATS_HISTORY_FRAMES`] samples: recording never
/// allocates and every derived metric recomputes in O(history) so the struct
/// stays deterministic across hosts. Serde support (feature `serde`) is a
/// hand implementation that preserves the window in recording order — serde
/// does not derive for arrays as long as the 120-slot ring, and ordered
/// export is what a wasm/host inspector wants anyway.
#[derive(Clone, Copy, Debug)]
pub struct RendererStats {
    samples: [FrameSample; STATS_HISTORY_FRAMES],
    head: usize,
    len: usize,
}

impl RendererStats {
    /// Empty window — every metric is zero until a recording is seen.
    pub const fn new() -> Self {
        Self {
            samples: [FrameSample::ZERO; STATS_HISTORY_FRAMES],
            head: 0,
            len: 0,
        }
    }
    /// Record one frame's sample, overwriting the oldest entry once full.
    pub fn record_frame(&mut self, sample: FrameSample) {
        if self.head >= STATS_HISTORY_FRAMES {
            self.head = 0;
        }
        self.samples[self.head] = sample;
        self.head += 1;
        if self.len < STATS_HISTORY_FRAMES {
            self.len += 1;
        }
    }

    /// Reset the window after separators such as scene reloads.
    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }

    /// Frames currently in the window.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether no frame has been recorded.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterates the newest-first window of samples.
    pub fn frames(&self) -> impl Iterator<Item = &FrameSample> + '_ {
        self.samples[0..self.len].iter().rev()
    }

    /// Latest recorded sample, if any.
    pub fn last(&self) -> Option<&FrameSample> {
        if self.len == 0 {
            None
        } else {
            self.samples.get(self.head.wrapping_sub(1))
        }
    }

    /// Rolling mean of wall times, in milliseconds.
    pub fn mean_frame_ms(&self) -> f32 {
        if self.len == 0 {
            return 0.0;
        }
        let sum: f64 = self.samples[0..self.len]
            .iter()
            .map(|sample| sample.frame_ms as f64)
            .sum();
        (sum / self.len as f64) as f32
    }

    /// Frames per second implied by the mean wall time.
    pub fn fps(&self) -> f32 {
        let mean = self.mean_frame_ms();
        if mean > 0.0 { 1000.0 / mean } else { 0.0 }
    }

    /// Worst (maximum) wall time in the window, for hitch reporting.
    pub fn max_frame_ms(&self) -> f32 {
        self.samples[0..self.len]
            .iter()
            .map(|sample| sample.frame_ms)
            .fold(0.0_f32, f32::max)
    }

    /// Rolling mean of draw calls.
    pub fn mean_draw_calls(&self) -> u32 {
        if self.len == 0 {
            return 0;
        }
        let sum: u64 = self.samples[0..self.len]
            .iter()
            .map(|sample| sample.draw_calls as u64)
            .sum();
        (sum / self.len as u64) as u32
    }

    /// Rolling mean of triangles per frame.
    pub fn mean_triangles(&self) -> u32 {
        if self.len == 0 {
            return 0;
        }
        let sum: u64 = self.samples[0..self.len]
            .iter()
            .map(|sample| sample.triangles as u64)
            .sum();
        (sum / self.len as u64) as u32
    }

    /// Rolling mean of bytes uploaded per frame.
    pub fn mean_resource_bytes(&self) -> u64 {
        if self.len == 0 {
            return 0;
        }
        let sum: u64 = self
            .samples
            .iter()
            .map(|sample| sample.resource_bytes)
            .sum();
        sum / self.len as u64
    }

    /// Check the rolling window against a `BudgetPolicy`.
    ///
    /// Fills `out` with exactly the warnings currently violated, in the fixed
    /// order of the enum variants, without allocating.
    pub fn budget_warnings(&self, policy: &BudgetPolicy, out: &mut alloc::vec::Vec<BudgetWarning>) {
        out.clear();
        if self.mean_frame_ms() > policy.max_frame_ms {
            out.push(BudgetWarning::FrameTime);
        }
        if self.mean_draw_calls() > policy.max_mean_draw_calls {
            out.push(BudgetWarning::DrawCalls);
        }
        if self.mean_triangles() > policy.max_mean_triangles {
            out.push(BudgetWarning::Triangles);
        }
        if self.mean_resource_bytes() > policy.max_mean_upload_bytes {
            out.push(BudgetWarning::ResourceUploads);
        }
    }

    /// One human-readable line for overlay banners.
    ///
    /// Pre-sized with capacity to avoid repeated growth while recording the
    /// frame loop, and finite-guarded so corrupt hosts never render `nan`.
    pub fn summary_line(&self) -> String {
        let mut line = String::with_capacity(96);
        let _ = write!(
            line,
            "fps {:.1} | mean {:.1} ms | worst {:.1} ms",
            finite(self.fps()),
            finite(self.mean_frame_ms()),
            finite(self.max_frame_ms())
        );
        line
    }

    /// Second overlay line with geometry/upload counters.
    pub fn summary_counts(&self) -> String {
        let mut line = String::with_capacity(96);
        let _ = write!(
            line,
            "draw {} | tris {} | upload {} KiB",
            self.mean_draw_calls(),
            self.mean_triangles(),
            self.mean_resource_bytes() / 1024
        );
        line
    }
}

impl Default for RendererStats {
    fn default() -> Self {
        Self::new()
    }
}

fn finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RendererStats {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Export the window in recording order (oldest first) plus the
        // cursor state, so a host inspector replays exactly what was shown.
        use serde::ser::SerializeStruct;
        let ordered: alloc::vec::Vec<FrameSample> = self.samples[0..self.len]
            .iter()
            .map(|sample| {
                FrameSample::sanitized(
                    sample.frame_ms,
                    sample.draw_calls,
                    sample.triangles,
                    sample.resource_bytes,
                )
            })
            .collect();
        let mut state = serializer.serialize_struct("RendererStats", 3)?;
        state.serialize_field("head", &self.head)?;
        state.serialize_field("len", &self.len)?;
        state.serialize_field("samples", &ordered)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RendererStats {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            head: usize,
            len: usize,
            samples: alloc::vec::Vec<FrameSample>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let mut stats = RendererStats::new();
        for sample in raw.samples.into_iter().take(STATS_HISTORY_FRAMES) {
            stats.record_frame(sample);
        }
        stats.head = raw.head.min(STATS_HISTORY_FRAMES);
        stats.len = raw.len.min(STATS_HISTORY_FRAMES);
        Ok(stats)
    }
}
