//! Presentation-frame bookkeeping: viewports, display timing, and
//! per-frame stats the compositor expects.

use core::time::Duration;

use scenix_camera::XrEyeProjection;
use scenix_math::Transform;

/// Largest viewport scale a runtime will validate for depth targets.
pub const XR_MAX_VIEWPORT_SCALE: f32 = 2.0;

/// Swapchain viewport for one eye, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrViewport {
    /// Logical width in pixels.
    pub width: u32,
    /// Logical height in pixels.
    pub height: u32,
    /// Render target scale `>= 1.0`; `1.0` uses native resolution.
    pub scale: f32,
}

impl XrViewport {
    /// Creates a viewport with native scale.
    #[inline]
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            scale: 1.0,
        }
    }

    /// Returns the viewport with a new render scale clamped to
    /// [`XR_MAX_VIEWPORT_SCALE`].
    #[inline]
    #[must_use]
    pub fn with_scale(self, scale: f32) -> Self {
        Self {
            scale: scale.clamp(1.0, XR_MAX_VIEWPORT_SCALE),
            ..self
        }
    }

    /// Physical buffer width after scaling the native pixels.
    #[inline]
    #[must_use]
    pub fn physical_width(&self) -> u32 {
        ((scenix_math::float::round(self.width as f32 * self.scale)) as u32).max(1)
    }

    /// Physical buffer height after scaling the native pixels.
    #[inline]
    #[must_use]
    pub fn physical_height(&self) -> u32 {
        ((scenix_math::float::round(self.height as f32 * self.scale)) as u32).max(1)
    }

    /// Physical aspect ratio.
    #[inline]
    #[must_use]
    pub fn aspect(&self) -> f32 {
        self.physical_width() as f32 / self.physical_height() as f32
    }

    /// Total physical pixel count per frame for both eyes.
    #[inline]
    #[must_use]
    pub fn stereo_pixel_count(&self) -> u64 {
        let per_eye = self.physical_width() as u64 * self.physical_height() as u64;
        per_eye.saturating_mul(2)
    }
}

impl Default for XrViewport {
    #[inline]
    fn default() -> Self {
        // A conservative common-case: 2064x2208 style panels.
        Self::new(2064, 2208)
    }
}

/// Monotonic per-frame stats the frame loop carries.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrFrameStats {
    /// Incrementing frame counter since session start.
    pub frame_index: u64,
    /// Runtime-predicted display time of this frame.
    pub predicted_display: Duration,
    /// CPU time spent preparing this frame, microseconds.
    pub cpu_prep_us: u32,
    /// CPU time spent submitting this frame, microseconds.
    pub submit_us: u32,
    /// Tracking timestep in seconds for CPU simulation.
    pub dt_seconds: f32,
}

impl XrFrameStats {
    /// Frames expected per second from the last display interval.
    #[inline]
    #[must_use]
    pub fn estimated_fps(&self) -> f32 {
        if self.dt_seconds <= 0.0 {
            0.0
        } else {
            1.0 / self.dt_seconds
        }
    }

    /// Total CPU budget consumed this frame, microseconds.
    #[inline]
    #[must_use]
    pub const fn total_cpu_us(&self) -> u32 {
        match self.cpu_prep_us.checked_add(self.submit_us) {
            Some(total) => total,
            None => u32::MAX,
        }
    }
}

/// Full-bookkeeping plan for one presentation frame.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrFramePlan {
    /// Per-frame stats the app should log along with GPU timings.
    pub stats: XrFrameStats,
    /// Head transform predicted at display time.
    pub head: Transform,
    /// Per-eye projection the runtime reported.
    pub eye: XrEyeProjection,
    /// Swapchain viewport.
    pub viewport: XrViewport,
}

impl XrFramePlan {
    /// Creates a frame plan from a head pose and a 90° stereo layout.
    #[inline]
    #[must_use]
    pub fn new(head: Transform) -> Self {
        Self {
            stats: XrFrameStats::default(),
            head,
            eye: XrEyeProjection::default(),
            viewport: XrViewport::default(),
        }
    }

    /// Returns the plan with new per-eye projections.
    #[inline]
    #[must_use]
    pub const fn with_eye(mut self, eye: XrEyeProjection) -> Self {
        self.eye = eye;
        self
    }

    /// Returns the plan with a new viewport.
    #[inline]
    #[must_use]
    pub const fn with_viewport(mut self, viewport: XrViewport) -> Self {
        self.viewport = viewport;
        self
    }

    /// Returns the plan with new frame stats.
    #[inline]
    #[must_use]
    pub const fn with_stats(mut self, stats: XrFrameStats) -> Self {
        self.stats = stats;
        self
    }

    /// Advances to the next frame, ticking stats.
    #[must_use]
    pub fn next_frame(&self, head: Transform, dt_seconds: f32) -> Self {
        Self {
            stats: XrFrameStats {
                frame_index: self.stats.frame_index.saturating_add(1),
                predicted_display: self
                    .stats
                    .predicted_display
                    .saturating_add(Duration::from_secs_f32(dt_seconds.max(0.0))),
                cpu_prep_us: self.stats.cpu_prep_us,
                submit_us: self.stats.submit_us,
                dt_seconds: dt_seconds.max(0.0),
            },
            head,
            eye: self.eye,
            viewport: self.viewport,
        }
    }

    /// Builds the stereo camera rig for this frame's head pose.
    #[inline]
    #[must_use]
    pub fn camera_rig(&self) -> scenix_camera::XrCameraRig {
        scenix_camera::XrCameraRig::new()
            .with_head(self.head)
            .with_eye(self.eye)
    }
}

impl Default for XrFramePlan {
    #[inline]
    fn default() -> Self {
        Self::new(Transform::IDENTITY)
    }
}
