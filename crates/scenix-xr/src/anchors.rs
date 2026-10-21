//! Spatial anchors and detected planes with deterministic bookkeeping.

use alloc::vec::Vec;

use scenix_core::XrAnchorId;
use scenix_math::Vec3;

use crate::controllers::XrPose;

/// How long an anchor may go unseen before the store prunes it, frames.
pub const XR_ANCHOR_MAX_AGE: u64 = 120;

/// A world-locked anchor produced by the runtime.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrAnchor {
    /// Opaque anchor identifier.
    pub id: XrAnchorId,
    /// Pose in the session's reference space.
    pub pose: XrPose,
    /// Runtime confidence `0..=1`.
    pub confidence: f32,
    /// Frame index at which the anchor was last seen as valid.
    pub last_seen_frame: u64,
    /// Whether the runtime reports the anchor changed since last frame.
    pub needs_update: bool,
    /// Whether the anchor expired or was deleted by the runtime.
    pub expired: bool,
}

impl XrAnchor {
    /// Creates a confident, freshly-tracked anchor.
    #[inline]
    #[must_use]
    pub fn new(id: XrAnchorId, pose: XrPose, frame: u64) -> Self {
        Self {
            id,
            pose,
            confidence: 1.0,
            last_seen_frame: frame,
            needs_update: false,
            expired: false,
        }
    }

    /// Applies a runtime pose update with confidence.
    #[inline]
    pub fn update(&mut self, pose: XrPose, confidence: f32, frame: u64) {
        self.pose = pose;
        self.confidence = confidence.clamp(0.0, 1.0);
        self.last_seen_frame = frame.max(self.last_seen_frame);
        self.needs_update = true;
        self.expired = false;
    }

    /// Returns the anchor's age in frames relative to `current_frame`.
    #[inline]
    #[must_use]
    pub const fn age(&self, current_frame: u64) -> u64 {
        current_frame.saturating_sub(self.last_seen_frame)
    }

    /// Memory-copy of pose plus confidence for hot serialization paths.
    #[inline]
    #[must_use]
    pub fn pack(&self) -> ([f32; 7], f32) {
        let pose = self.pose;
        (
            [
                pose.position.x,
                pose.position.y,
                pose.position.z,
                pose.orientation.x,
                pose.orientation.y,
                pose.orientation.z,
                pose.orientation.w,
            ],
            self.confidence,
        )
    }
}

/// Fixed-capacity anchor store with deterministic, id-ordered bookkeeping.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrAnchorStore {
    anchors: Vec<XrAnchor>,
    capacity: usize,
}

impl XrAnchorStore {
    /// Creates a store with explicit capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            anchors: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    /// Creates the default 32-slot store.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(32)
    }

    /// Inserts a freshly tracked anchor, evicting the lowest-confidence
    /// stale entry when full.
    pub fn insert(&mut self, anchor: XrAnchor) -> bool {
        if self.contains(anchor.id) {
            return false;
        }
        if self.anchors.len() >= self.capacity {
            let eviction = self
                .anchors
                .iter()
                .enumerate()
                .min_by(|(ia, a), (ib, b)| {
                    a.confidence
                        .total_cmp(&b.confidence)
                        .then(b.last_seen_frame.cmp(&a.last_seen_frame))
                        .then((*ia as u64).cmp(&(*ib as u64)))
                })
                .map(|(index, _)| index);
            if let Some(index) = eviction {
                self.anchors.swap_remove(index);
            } else {
                return false;
            }
        }
        self.anchors.push(anchor);
        self.sort_for_determinism();
        true
    }

    /// Returns whether `id` is tracked.
    #[inline]
    #[must_use]
    pub fn contains(&self, id: XrAnchorId) -> bool {
        self.anchors.iter().any(|anchor| anchor.id == id)
    }

    /// Returns the anchor for `id`.
    #[inline]
    #[must_use]
    pub fn get(&self, id: XrAnchorId) -> Option<&XrAnchor> {
        self.anchors.iter().find(|anchor| anchor.id == id)
    }

    /// Returns the mutable anchor for `id`.
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, id: XrAnchorId) -> Option<&mut XrAnchor> {
        self.anchors.iter_mut().find(|anchor| anchor.id == id)
    }

    /// Applies a runtime update to `id`.
    #[inline]
    pub fn update(&mut self, id: XrAnchorId, pose: XrPose, confidence: f32, frame: u64) {
        if let Some(anchor) = self.get_mut(id) {
            anchor.update(pose, confidence, frame);
        }
    }

    /// Flags anchors that have not been seen for [`XR_ANCHOR_MAX_AGE`].
    #[must_use]
    pub fn prune(&mut self, current_frame: u64) -> Vec<XrAnchorId> {
        let mut pruned = Vec::new();
        for anchor in self.anchors.iter_mut() {
            if anchor.age(current_frame) > XR_ANCHOR_MAX_AGE {
                anchor.expired = true;
                pruned.push(anchor.id);
            }
        }
        self.anchors.retain(|anchor| !anchor.expired);
        pruned
    }

    /// Number of live (non-expired) anchors.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    /// Returns whether the store holds no anchors.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }

    /// Live anchors in deterministic id order.
    #[inline]
    #[must_use]
    pub fn anchors(&self) -> &[XrAnchor] {
        &self.anchors
    }

    /// Highest-confidence live anchor.
    #[must_use]
    pub fn best(&self) -> Option<&XrAnchor> {
        self.anchors.iter().max_by(|a, b| {
            a.confidence
                .total_cmp(&b.confidence)
                .then(a.id.get().cmp(&b.id.get()))
        })
    }

    #[inline]
    fn sort_for_determinism(&mut self) {
        self.anchors.sort_unstable_by_key(|anchor| anchor.id.get());
    }
}

impl Default for XrAnchorStore {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Orientation category of a detected plane.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrPlaneKind {
    /// Floor facing up.
    HorizontalUp,
    /// Ceiling facing down.
    HorizontalDown,
    /// Wall or other vertical surface.
    Vertical,
    /// Unclassified orientation.
    #[default]
    Arbitrary,
}

impl XrPlaneKind {
    /// Returns whether the plane should be trusted as a floor.
    #[inline]
    #[must_use]
    pub const fn is_floor(self) -> bool {
        matches!(self, Self::HorizontalUp)
    }
}

/// A detected real-world plane.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrPlane {
    /// Opaque runtime plane identifier.
    pub id: u64,
    /// Category reported by the runtime classifier.
    pub kind: XrPlaneKind,
    /// Center pose for the plane rectangle.
    pub center_pose: XrPose,
    /// Extent along the plane's local right axis, meters.
    pub extent_x: f32,
    /// Extent along the plane's local up-parallel axis, meters.
    pub extent_z: f32,
    /// Surface `0..=1` over which a normal points away from the wall.
    pub confidence: f32,
}

impl XrPlane {
    /// Creates a plane with a center pose and extents.
    #[inline]
    #[must_use]
    pub fn new(
        id: u64,
        kind: XrPlaneKind,
        center_pose: XrPose,
        extent_x: f32,
        extent_z: f32,
    ) -> Self {
        Self {
            id,
            kind,
            center_pose,
            extent_x: extent_x.max(0.0),
            extent_z: extent_z.max(0.0),
            confidence: 1.0,
        }
    }

    /// Area of the plane rectangle in square meters.
    #[inline]
    #[must_use]
    pub const fn area(&self) -> f32 {
        self.extent_x * self.extent_z
    }

    /// Surface normal in tracking space.
    #[inline]
    #[must_use]
    pub fn normal(&self) -> Vec3 {
        match self.kind {
            XrPlaneKind::HorizontalUp => Vec3::Y,
            XrPlaneKind::HorizontalDown => Vec3::new(0.0, -1.0, 0.0),
            XrPlaneKind::Vertical | XrPlaneKind::Arbitrary => self.center_pose.up(),
        }
    }

    /// Id-ordering of plane ids on insertion for deterministic stores.
    #[inline]
    #[must_use]
    pub const fn is_before(&self, other: &Self) -> bool {
        self.id < other.id
    }
}

/// Fixed-capacity plane list, id-ordered for deterministic framing.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrPlaneStore {
    planes: Vec<XrPlane>,
    capacity: usize,
}

impl XrPlaneStore {
    /// Creates the default 32-slot plane store.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            planes: Vec::new(),
            capacity: 32,
        }
    }

    /// Replaces the detected plane set with `detected`, keeping id order
    /// deterministic and returning whether the floor is among them.
    pub fn reconcile(&mut self, detected: Vec<XrPlane>) -> bool {
        detected.into_iter().take(self.capacity).for_each(|plane| {
            self.planes.retain(|existing| existing.id != plane.id);
            self.planes.push(plane);
        });
        self.planes
            .sort_unstable_by_key(|plane| (plane.id, plane.kind as u8));
        self.planes.iter().any(|plane| plane.kind.is_floor())
    }

    /// Live planes in deterministic order.
    #[inline]
    #[must_use]
    pub fn planes(&self) -> &[XrPlane] {
        &self.planes
    }

    /// Clears all detections for new runtime frames.
    #[inline]
    pub fn clear(&mut self) {
        self.planes.clear();
    }

    /// Returns the largest plane, area ties broken by id.
    #[must_use]
    pub fn largest(&self) -> Option<&XrPlane> {
        self.planes
            .iter()
            .max_by(|a, b| a.area().total_cmp(&b.area()).then(b.id.cmp(&a.id)))
    }
}

impl Default for XrPlaneStore {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
