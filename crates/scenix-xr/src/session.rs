//! XR session plans: modes, reference spaces, feature negotiation, and
//! the session lifecycle state machine.

use core::fmt;

use scenix_core::XrSessionId;

/// Runtime environment a session is requested against.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrSessionMode {
    /// Inline AR: the experience renders inside the existing page/window.
    Inline,
    /// Fully immersive virtual reality headed display.
    ImmersiveVr,
    /// Immersive passthrough augmented reality, the scenix default for
    /// mobile WebXR.
    #[default]
    ImmersiveAr,
}

impl XrSessionMode {
    /// Returns whether the mode expects passthrough video mixing.
    #[inline]
    #[must_use]
    pub const fn is_passthrough(self) -> bool {
        matches!(self, Self::ImmersiveAr | Self::Inline)
    }

    /// Returns whether the session is fully immersive (head-tracked).
    #[inline]
    #[must_use]
    pub const fn is_immersive(self) -> bool {
        !matches!(self, Self::Inline)
    }

    /// Returns a short human-readable label used by inspector tooling.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::ImmersiveVr => "immersive-vr",
            Self::ImmersiveAr => "immersive-ar",
        }
    }
}

/// Local coordinate reference space a session anchors world content to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrReferenceSpace {
    /// Head-locked space; never stationary, used for HUD content.
    Viewer,
    /// Stationary space at the initial spectator position.
    #[default]
    Local,
    /// Stationary space with the floor at `Y = 0`.
    BoundedFloor,
    /// Ultimate room-scale space with a reproducible floor origin.
    Stage,
}

impl XrReferenceSpace {
    /// Returns whether the space guarantees a known floor height.
    #[inline]
    #[must_use]
    pub const fn has_floor(self) -> bool {
        matches!(self, Self::BoundedFloor | Self::Stage)
    }

    /// Returns a short human-readable label used by inspector tooling.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Local => "local",
            Self::BoundedFloor => "local-floor",
            Self::Stage => "stage",
        }
    }
}

/// Fixed-capacity bitset of negotiated XR session features.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrFeatures {
    bits: u32,
}

/// Number of recognized XR features; the negotiation bitset width.
pub const XR_FEATURE_COUNT: usize = 8;

impl XrFeatures {
    /// Hand input sources (grip + pointer poses).
    pub const CONTROLLERS: Self = Self::from_bits(1 << 0);
    /// Full articulated hand tracking.
    pub const HANDS: Self = Self::from_bits(1 << 1);
    /// Ray/plane intersection probes against the real world.
    pub const HIT_TEST: Self = Self::from_bits(1 << 2);
    /// Persistent world-locked anchors.
    pub const ANCHORS: Self = Self::from_bits(1 << 3);
    /// Real-world plane detection (floor, wall, table, ...).
    pub const PLANES: Self = Self::from_bits(1 << 4);
    /// Depth-aware camera images or raw depth buffers.
    pub const DEPTH_SENSING: Self = Self::from_bits(1 << 5);
    /// Runtime-provided lighting estimates for scene lights.
    pub const LIGHT_ESTIMATION: Self = Self::from_bits(1 << 6);
    /// Video passthrough or environmental blending.
    pub const PASSTHROUGH: Self = Self::from_bits(1 << 7);

    /// Constructs a feature set from raw bits.
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self {
            bits: bits & ((1 << XR_FEATURE_COUNT) - 1),
        }
    }

    /// Returns the raw feature bits.
    #[inline]
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.bits
    }

    /// Returns the empty feature set.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Returns whether `self` contains every feature in `other`.
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Returns whether `self` contains none of `other`.
    #[inline]
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        (self.bits & other.bits) != 0
    }

    /// Returns whether no feature is negotiated.
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    /// Union of two feature sets.
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self::from_bits(self.bits | other.bits)
    }

    /// Removes every feature in `other`.
    #[inline]
    #[must_use]
    pub const fn without(self, other: Self) -> Self {
        Self::from_bits(self.bits & !other.bits)
    }

    /// Keeps only the features of `other`.
    #[inline]
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self::from_bits(self.bits & other.bits)
    }

    /// Returns the name of feature `index`, `None` when out of bounds.
    #[inline]
    #[must_use]
    pub fn name_at(self, index: usize) -> Option<&'static str> {
        if index >= XR_FEATURE_COUNT {
            return None;
        }
        if (self.bits & (1 << index)) == 0 {
            return None;
        }
        Some(FEATURE_NAMES[index])
    }

    /// Iterates the enabled feature names in stable bit order.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        (0..XR_FEATURE_COUNT)
            .filter(move |&i| (self.bits & (1 << i)) != 0)
            .filter_map(move |i| self.name_at(i))
            .take(XR_FEATURE_COUNT)
    }

    /// Feature set typical of mobile AR (passthrough + hit tests + planes).
    #[inline]
    #[must_use]
    pub const fn mobile_ar() -> Self {
        Self::from_bits((1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 6) | (1 << 7))
    }

    /// Feature set typical of room-scale VR: controllers and hands, no
    /// passthrough (that belongs to AR sessions).
    #[inline]
    #[must_use]
    pub const fn room_scale_vr() -> Self {
        Self::from_bits((1 << 0) | (1 << 1))
    }
}

impl core::ops::BitOr for XrFeatures {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl core::ops::BitOrAssign for XrFeatures {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

/// Names for [`XR_FEATURE_COUNT`] features, index-matched to the bitset.
pub const FEATURE_NAMES: [&str; XR_FEATURE_COUNT] = [
    "controllers",
    "hands",
    "hit-test",
    "anchors",
    "planes",
    "depth-sensing",
    "light-estimation",
    "passthrough",
];

/// Stable iterator over enabled feature names.
#[derive(Clone, Debug)]
pub struct XrFeatureIter<'a> {
    features: &'a XrFeatures,
    next: usize,
}

impl Iterator for XrFeatureIter<'_> {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next < XR_FEATURE_COUNT {
            let index = self.next;
            self.next += 1;
            let name = self.features.name_at(index);
            if name.is_some() {
                return name;
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(XR_FEATURE_COUNT))
    }
}

impl XrFeatures {
    /// Returns a stable, deterministic iterator over enabled feature names.
    #[inline]
    #[must_use]
    pub fn iter(&self) -> XrFeatureIter<'_> {
        XrFeatureIter {
            features: self,
            next: 0,
        }
    }
}

/// Foveation level: how aggressively outward-frame pixels are compressed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrFoveation {
    /// No foveation; maximum peripheral detail.
    None,
    /// Slight compression; a safe default.
    #[default]
    Low,
    /// Visible compression; recommended for heavy scenes.
    Medium,
    /// Aggressive compression; maximum throughput.
    High,
}

impl XrFoveation {
    /// Returns `0..=3` for WGSL packing into WebGPU uniform words.
    #[inline]
    #[must_use]
    pub const fn to_unit(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }

    /// Returns the level from an integer, clamping unknown values.
    #[inline]
    #[must_use]
    pub const fn from_unit(value: u32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Low,
            2 => Self::Medium,
            3 => Self::High,
            _ => Self::Low,
        }
    }
}

/// Errors raised while negotiating a session plan.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum XrSessionPlanError {
    /// An inline session requested a feature that only immersive modes
    /// support.
    FeatureRequiresImmersive,
    /// A VR session requested passthrough-only features.
    PassthroughRequiresAr,
    /// The requested floor reference space lacks passthrough support.
    FloorWithoutPassthrough,
    /// The requested state transition is illegal for the lifecycle.
    IllegalTransition {
        /// The state the plan was in.
        from: XrSessionState,
        /// The state the plan was asked to move to.
        to: XrSessionState,
    },
}

impl fmt::Debug for XrSessionPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FeatureRequiresImmersive => write!(f, "feature requires an immersive mode"),
            Self::PassthroughRequiresAr => write!(f, "passthrough requires immersive-ar"),
            Self::FloorWithoutPassthrough => write!(f, "floor space requires passthrough"),
            Self::IllegalTransition { from, to } => {
                write!(f, "illegal transition from {from:?} to {to:?}")
            }
        }
    }
}

impl fmt::Display for XrSessionPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FeatureRequiresImmersive => f.write_str("feature requires an immersive mode"),
            Self::PassthroughRequiresAr => f.write_str("passthrough requires immersive-ar"),
            Self::FloorWithoutPassthrough => f.write_str("floor space requires passthrough"),
            Self::IllegalTransition { from, to } => {
                write!(f, "illegal transition from {from:?} to {to:?}")
            }
        }
    }
}

impl core::error::Error for XrSessionPlanError {}

/// Lifecycle stages of an XR session.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrSessionState {
    /// No session has been requested yet.
    #[default]
    Idle,
    /// A request is in flight (waiting for user permission).
    Requesting,
    /// Request granted; the runtime is starting up.
    Starting,
    /// Presentation frames are flowing.
    Active,
    /// Presentation is paused (blur, notification, ...).
    Paused,
    /// A graceful `end()` was requested.
    Ending,
    /// Session ended; no more frames flow.
    Ended,
}

impl XrSessionState {
    /// Returns whether state `from` may advance directly to `to`.
    #[inline]
    #[must_use]
    pub const fn can_transition(from: Self, to: Self) -> bool {
        matches!(
            (from, to),
            (Self::Idle, Self::Requesting)
                | (Self::Requesting, Self::Starting)
                | (Self::Starting, Self::Active)
                | (Self::Active, Self::Paused)
                | (Self::Paused, Self::Active)
                | (Self::Active | Self::Paused, Self::Ending)
                | (Self::Ending, Self::Ended)
                | (
                    Self::Requesting | Self::Starting | Self::Active | Self::Paused,
                    Self::Ended
                )
        )
    }
}

/// Deterministic CPU plan of an XR session: mode, reference space,
/// negotiated features, and foveation.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrSessionPlan {
    /// Opaque session identifier.
    pub id: XrSessionId,
    /// Requested session mode.
    pub mode: XrSessionMode,
    /// Reference space world content anchors to.
    pub reference_space: XrReferenceSpace,
    /// Features requested from the runtime.
    pub features: XrFeatures,
    /// Head-mounted foveation level.
    pub foveation: XrFoveation,
    /// Current lifecycle state.
    pub state: XrSessionState,
}

impl XrSessionPlan {
    /// Creates a fully immersive VR session plan.
    #[inline]
    #[must_use]
    pub fn immersive_vr() -> Self {
        Self {
            id: XrSessionId::new(1),
            mode: XrSessionMode::ImmersiveVr,
            reference_space: XrReferenceSpace::Local,
            features: XrFeatures::room_scale_vr(),
            foveation: XrFoveation::Low,
            state: XrSessionState::Idle,
        }
    }

    /// Creates an immersive AR session plan with mobile-AR features.
    #[inline]
    #[must_use]
    pub fn immersive_ar() -> Self {
        Self {
            id: XrSessionId::new(1),
            mode: XrSessionMode::ImmersiveAr,
            reference_space: XrReferenceSpace::BoundedFloor,
            features: XrFeatures::mobile_ar(),
            foveation: XrFoveation::Low,
            state: XrSessionState::Idle,
        }
    }

    /// Creates an inline (page-embedded) session plan.
    #[inline]
    #[must_use]
    pub fn inline() -> Self {
        Self {
            id: XrSessionId::new(1),
            mode: XrSessionMode::Inline,
            reference_space: XrReferenceSpace::Viewer,
            features: XrFeatures::empty(),
            foveation: XrFoveation::None,
            state: XrSessionState::Idle,
        }
    }

    /// Returns the plan with a new session id.
    #[inline]
    #[must_use]
    pub const fn with_id(mut self, id: XrSessionId) -> Self {
        self.id = id;
        self
    }

    /// Returns the plan with a new reference space.
    #[inline]
    #[must_use]
    pub const fn with_reference_space(mut self, space: XrReferenceSpace) -> Self {
        self.reference_space = space;
        self
    }

    /// Returns the plan with new requested features.
    #[inline]
    #[must_use]
    pub const fn with_features(mut self, features: XrFeatures) -> Self {
        self.features = features;
        self
    }

    /// Returns the plan with a new foveation level.
    #[inline]
    #[must_use]
    pub const fn with_foveation(mut self, foveation: XrFoveation) -> Self {
        self.foveation = foveation;
        self
    }

    /// Validates mode/feature/space consistency.
    ///
    /// Returns `Err` with the specific violation when the plan could not
    /// be granted by any known runtime.
    pub fn validate(&self) -> Result<(), XrSessionPlanError> {
        if !self.mode.is_immersive()
            && self.features.intersects(
                XrFeatures::HIT_TEST
                    | XrFeatures::ANCHORS
                    | XrFeatures::PLANES
                    | XrFeatures::DEPTH_SENSING,
            )
        {
            return Err(XrSessionPlanError::FeatureRequiresImmersive);
        }
        if self.mode == XrSessionMode::ImmersiveVr
            && self.features.contains(XrFeatures::PASSTHROUGH)
        {
            return Err(XrSessionPlanError::PassthroughRequiresAr);
        }
        if self.reference_space.has_floor()
            && !self.features.contains(XrFeatures::PASSTHROUGH)
            && self.mode == XrSessionMode::ImmersiveAr
        {
            return Err(XrSessionPlanError::FloorWithoutPassthrough);
        }
        Ok(())
    }

    /// Advances the lifecycle when `to` is a legal transition.
    ///
    /// Returns `Err` and leaves the plan untouched on illegal moves so the
    /// state machine stays deterministic.
    pub fn advance(&mut self, to: XrSessionState) -> Result<(), XrSessionPlanError> {
        let from = self.state;
        if XrSessionState::can_transition(from, to) {
            self.state = to;
            Ok(())
        } else {
            Err(XrSessionPlanError::IllegalTransition { from, to })
        }
    }

    /// Returns whether presentation frames should flow right now.
    #[inline]
    #[must_use]
    pub const fn is_presenting(&self) -> bool {
        matches!(self.state, XrSessionState::Active)
    }

    /// Builds a stereo camera rig from this session's head pose estimate.
    ///
    /// Rigs always carry the default headset frustum; callers override
    /// `with_eye` when the runtime reports real projection values.
    #[must_use]
    pub fn camera_rig(&self, head: scenix_math::Transform) -> scenix_camera::XrCameraRig {
        let _ = self.foveation; // foveation is a compositor setting, not a projection one.
        scenix_camera::XrCameraRig::new().with_head(head)
    }
}

impl Default for XrSessionPlan {
    #[inline]
    fn default() -> Self {
        Self::immersive_ar()
    }
}
