//! Streaming audio buffer bookkeeping for schedulers.

/// Maximum number of buffered chunks a stream tracks.
pub const STREAM_MAX_CHUNKS: usize = 8;

/// Underrun counter wrapping point.
pub const STREAM_UNDERRUN_CAPACITY: usize = 4096;

/// Wire format of one streaming source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum StreamFormat {
    /// 16-bit signed PCM, interleaved.
    Pcm16,
    /// 32-bit float PCM, interleaved (WebAudio's default).
    PcmF32,
    /// Compressed bytes (Opus/MP3) passed through to the backend.
    Compressed,
}

impl StreamFormat {
    /// Bytes per sample for PCM formats; `0` for compressed streams.
    #[inline]
    #[must_use]
    pub const fn bytes_per_sample(self) -> usize {
        match self {
            Self::Pcm16 => 2,
            Self::PcmF32 => 4,
            Self::Compressed => 0,
        }
    }

    /// Whether the backend decodes this format natively per frame.
    #[inline]
    #[must_use]
    pub const fn is_pcm(self) -> bool {
        !matches!(self, Self::Compressed)
    }
}

/// State of one streaming source's buffer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum StreamState {
    /// No buffered audio; nothing sound exists to schedule.
    #[default]
    Empty,
    /// Some chunks queued; playback is healthy if enough are ready.
    Buffered,
    /// Starved: too few ready chunks, expect audible gaps.
    Underrun,
    /// End of the media pushed into the buffer.
    Finished,
}

impl StreamState {
    /// Returns whether the stream should attempt scheduling this frame.
    #[inline]
    #[must_use]
    pub const fn is_schedulable(self) -> bool {
        matches!(self, Self::Buffered)
    }
}

/// Scheduler bookkeeping for one streaming source.
///
/// Tracks a ring of chunk flags against a sample rate, deriving readiness
/// from buffered milliseconds so host engines can decide when to poke.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AudioStreamPlan {
    /// Opaque stream identifier.
    pub id: scenix_core::AudioSourceId,
    /// Wire format of buffered chunks.
    pub format: StreamFormat,
    /// Sample rate in Hz (per channel, interleaved frames).
    pub sample_rate: u32,
    /// Channel count of the source.
    pub channels: u16,
    /// Number of chunks that are wholly valid and ready.
    pub ready_chunks: u8,
    /// Number of chunks reserved but still arriving.
    pub pending_chunks: u8,
    /// Chunks already consumed by the scheduler since stream start.
    pub consumed_chunks: u64,
    /// Underrun events observed, saturating at the capacity.
    pub underruns: u32,
    /// Current state of the stream.
    pub state: StreamState,
}

impl AudioStreamPlan {
    /// Creates a healthy streaming plan with an empty ring.
    #[must_use]
    pub fn new(id: u64, format: StreamFormat, sample_rate: u32, channels: u16) -> Self {
        Self {
            id: scenix_core::AudioSourceId::new(id),
            format,
            sample_rate: sample_rate.max(1),
            channels: channels.max(1),
            ready_chunks: 0,
            pending_chunks: 0,
            consumed_chunks: 0,
            underruns: 0,
            state: StreamState::Empty,
        }
    }

    /// Total chunks currently occupying the ring.
    #[inline]
    #[must_use]
    pub const fn buffered_chunks(&self) -> u8 {
        self.ready_chunks.saturating_add(self.pending_chunks)
    }

    /// Buffered milliseconds available for scheduling.
    ///
    /// PENDING chunks do not count; only fully validated bytes do.
    #[must_use]
    pub fn buffered_milliseconds(&self, chunk_frames: u32) -> f32 {
        let frames = f32::from(self.ready_chunks) * chunk_frames.max(1) as f32;
        let seconds = frames / self.sample_rate.max(1) as f32;
        seconds * 1000.0
    }

    /// Ring byte capacity for one chunk of this stream's wiring format.
    #[must_use]
    pub fn chunk_byte_budget(&self, chunk_frames: u32) -> usize {
        let frames = chunk_frames.max(1) as usize;
        frames.saturating_mul(self.channels as usize) * self.format.bytes_per_sample()
    }

    /// Records a new arrival, unable to overflow the ring.
    #[inline]
    pub fn accept_chunk(&mut self) {
        if self.pending_chunks < STREAM_MAX_CHUNKS as u8 {
            self.pending_chunks += 1;
            self.state = StreamState::Buffered;
        }
    }

    /// Marks an arriving chunk fully validated.
    #[must_use]
    pub fn promote_pending(&mut self) -> bool {
        if self.pending_chunks == 0 {
            return false;
        }
        self.pending_chunks -= 1;
        self.ready_chunks = self.ready_chunks.min(STREAM_MAX_CHUNKS as u8 - 1) + 1;
        if self.state != StreamState::Finished {
            self.state = StreamState::Buffered;
        }
        true
    }

    /// Consumes one ready chunk for scheduling.
    #[must_use]
    pub fn consume_chunk(&mut self) -> bool {
        if self.ready_chunks == 0 {
            return false;
        }
        self.ready_chunks -= 1;
        self.consumed_chunks = self.consumed_chunks.saturating_add(1);
        if self.ready_chunks == 0 {
            self.state = if self.pending_chunks > 0 {
                StreamState::Buffered
            } else {
                StreamState::Underrun
            };
        }
        true
    }

    /// Records an audible-gap event, saturating counters.
    #[inline]
    pub fn register_underrun(&mut self) {
        self.underruns = self.underruns.saturating_add(1);
        if self.ready_chunks == 0 {
            self.state = StreamState::Underrun;
        }
    }

    /// Clears the ring after media termination.
    #[inline]
    pub fn finish(&mut self) {
        self.state = StreamState::Finished;
        self.pending_chunks = 0;
    }

    /// Resets for a replay.
    #[inline]
    pub fn reset(&mut self) {
        self.ready_chunks = 0;
        self.pending_chunks = 0;
        self.consumed_chunks = 0;
        self.underruns = 0;
        self.state = StreamState::Empty;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_formats() {
        assert_eq!(StreamFormat::Pcm16.bytes_per_sample(), 2);
        assert_eq!(StreamFormat::PcmF32.bytes_per_sample(), 4);
        assert_eq!(StreamFormat::Compressed.bytes_per_sample(), 0);
        assert!(StreamFormat::PcmF32.is_pcm());
        assert!(!StreamFormat::Compressed.is_pcm());
    }

    #[test]
    fn buffer_health_math() {
        let mut plan = AudioStreamPlan::new(2, StreamFormat::PcmF32, 48_000, 2);
        assert_eq!(plan.buffered_chunks(), 0);
        plan.accept_chunk();
        assert!(plan.promote_pending());
        assert_eq!(plan.buffered_chunks(), 1);
        assert_eq!(plan.ready_chunks, 1);
        // 1024 frames of stereo float32 at 48 kHz is ~21.3 ms.
        let ms = plan.buffered_milliseconds(1024);
        assert!((ms - 1024.0 * 1000.0 / 48_000.0).abs() < 1.0e-3);
        assert_eq!(
            plan.chunk_byte_budget(1024),
            1024 * 2 * 4,
            "f32 stereo budget"
        );
    }

    #[test]
    fn consumption_drains_to_underrun() {
        let mut plan = AudioStreamPlan::new(2, StreamFormat::Pcm16, 22_050, 1);
        plan.accept_chunk();
        plan.accept_chunk();
        assert!(plan.promote_pending());
        assert!(plan.promote_pending());
        assert_eq!(plan.ready_chunks, 2);
        assert!(plan.consume_chunk());
        assert!(plan.consume_chunk());
        assert!(!plan.consume_chunk());
        assert_eq!(plan.state, StreamState::Underrun);
        plan.register_underrun();
        assert_eq!(plan.underruns, 1);
        plan.reset();
        assert_eq!(plan.state, StreamState::Empty);
    }

    #[test]
    fn finish_and_replay() {
        let mut plan = AudioStreamPlan::new(2, StreamFormat::Compressed, 48_000, 2);
        plan.accept_chunk();
        assert!(plan.promote_pending());
        plan.finish();
        assert_eq!(plan.state, StreamState::Finished);
        assert_eq!(plan.pending_chunks, 0);
        plan.accept_chunk();
        assert_eq!(plan.state, StreamState::Buffered, "replay revives buffers");
        plan.reset();
        assert_eq!(plan.state, StreamState::Empty);
    }

    #[test]
    fn considers_underruns_schedulable() {
        assert!(StreamState::Buffered.is_schedulable());
        assert!(!StreamState::Underrun.is_schedulable());
        assert!(!StreamState::Finished.is_schedulable());
        assert_eq!(STREAM_MAX_CHUNKS, 8);
        assert_eq!(STREAM_UNDERRUN_CAPACITY, 4096);
    }
}
