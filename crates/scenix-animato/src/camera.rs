use alloc::collections::BTreeMap;

use scenix_camera::{OrthographicCamera, PerspectiveCamera};
use scenix_core::{CameraId, ValidationError};
use scenix_math::Vec3;

use crate::{ScalarTrack, Vec3Track};

/// Mutable camera lookup used by camera animators.
pub trait CameraStoreMut {
    /// Returns a mutable perspective camera, when this store contains one.
    fn perspective_mut(&mut self, _id: CameraId) -> Option<&mut PerspectiveCamera> {
        None
    }

    /// Returns a mutable orthographic camera, when this store contains one.
    fn orthographic_mut(&mut self, _id: CameraId) -> Option<&mut OrthographicCamera> {
        None
    }
}

impl CameraStoreMut for BTreeMap<CameraId, PerspectiveCamera> {
    #[inline]
    fn perspective_mut(&mut self, id: CameraId) -> Option<&mut PerspectiveCamera> {
        self.get_mut(&id)
    }
}

impl CameraStoreMut for BTreeMap<CameraId, OrthographicCamera> {
    #[inline]
    fn orthographic_mut(&mut self, id: CameraId) -> Option<&mut OrthographicCamera> {
        self.get_mut(&id)
    }
}

/// Borrowed perspective and orthographic camera maps.
pub struct CameraStores<'a> {
    /// Perspective cameras by ID.
    pub perspective: &'a mut BTreeMap<CameraId, PerspectiveCamera>,
    /// Orthographic cameras by ID.
    pub orthographic: &'a mut BTreeMap<CameraId, OrthographicCamera>,
}

impl CameraStoreMut for CameraStores<'_> {
    #[inline]
    fn perspective_mut(&mut self, id: CameraId) -> Option<&mut PerspectiveCamera> {
        self.perspective.get_mut(&id)
    }

    #[inline]
    fn orthographic_mut(&mut self, id: CameraId) -> Option<&mut OrthographicCamera> {
        self.orthographic.get_mut(&id)
    }
}

/// Orthographic projection bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrthographicBounds {
    /// Left projection bound.
    pub left: f32,
    /// Right projection bound.
    pub right: f32,
    /// Bottom projection bound.
    pub bottom: f32,
    /// Top projection bound.
    pub top: f32,
}

impl OrthographicBounds {
    /// Creates bounds from camera planes.
    #[inline]
    pub const fn new(left: f32, right: f32, bottom: f32, top: f32) -> Self {
        Self {
            left,
            right,
            bottom,
            top,
        }
    }
}

/// Four scalar tracks for orthographic projection bounds.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrthographicBoundsTrack {
    /// Left bound track.
    pub left: ScalarTrack,
    /// Right bound track.
    pub right: ScalarTrack,
    /// Bottom bound track.
    pub bottom: ScalarTrack,
    /// Top bound track.
    pub top: ScalarTrack,
}

impl OrthographicBoundsTrack {
    /// Creates a linear bounds tween.
    pub fn tween(start: OrthographicBounds, end: OrthographicBounds, duration: f32) -> Self {
        Self {
            left: ScalarTrack::tween(start.left, end.left, duration),
            right: ScalarTrack::tween(start.right, end.right, duration),
            bottom: ScalarTrack::tween(start.bottom, end.bottom, duration),
            top: ScalarTrack::tween(start.top, end.top, duration),
        }
    }

    /// Advances all tracks and returns whether any are still running.
    #[inline]
    pub fn update(&mut self, dt: f32) -> bool {
        self.left.update(dt) | self.right.update(dt) | self.bottom.update(dt) | self.top.update(dt)
    }

    /// Returns current bounds.
    #[inline]
    pub fn value(&self) -> OrthographicBounds {
        OrthographicBounds::new(
            self.left.value(),
            self.right.value(),
            self.bottom.value(),
            self.top.value(),
        )
    }

    /// Returns whether all tracks have completed.
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.left.is_complete()
            && self.right.is_complete()
            && self.bottom.is_complete()
            && self.top.is_complete()
    }
}

/// Camera fields that can be animated.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CameraAnimationTarget {
    /// Perspective vertical field of view in radians.
    FovY(ScalarTrack),
    /// Camera position.
    Position(Vec3Track),
    /// Camera look target.
    Target(Vec3Track),
    /// Camera up vector.
    Up(Vec3Track),
    /// Orthographic projection bounds.
    OrthographicBounds(OrthographicBoundsTrack),
}

impl CameraAnimationTarget {
    /// Advances the contained track.
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::FovY(track) => track.update(dt),
            Self::Position(track) | Self::Target(track) | Self::Up(track) => track.update(dt),
            Self::OrthographicBounds(track) => track.update(dt),
        }
    }

    /// Returns whether the contained track has completed.
    pub fn is_complete(&self) -> bool {
        match self {
            Self::FovY(track) => track.is_complete(),
            Self::Position(track) | Self::Target(track) | Self::Up(track) => track.is_complete(),
            Self::OrthographicBounds(track) => track.is_complete(),
        }
    }
}

/// Applies an animation track to a camera store entry.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CameraAnimator {
    /// Target camera ID.
    pub camera_id: CameraId,
    /// Field being animated.
    pub target: CameraAnimationTarget,
}

impl CameraAnimator {
    /// Creates a camera animator.
    #[inline]
    pub const fn new(camera_id: CameraId, target: CameraAnimationTarget) -> Self {
        Self { camera_id, target }
    }

    /// Advances the animator, applies the current value, and returns completion.
    pub fn update(
        &mut self,
        dt: f32,
        cameras: &mut impl CameraStoreMut,
    ) -> Result<bool, ValidationError> {
        self.target.update(dt);
        match &self.target {
            CameraAnimationTarget::FovY(track) => {
                let camera = cameras
                    .perspective_mut(self.camera_id)
                    .ok_or(ValidationError::InvalidId)?;
                camera.fov_y = track.value().clamp(
                    core::f32::consts::PI / 180.0,
                    179.0 * core::f32::consts::PI / 180.0,
                );
            }
            CameraAnimationTarget::Position(track) => {
                apply_position(cameras, self.camera_id, track.value())?;
            }
            CameraAnimationTarget::Target(track) => {
                apply_target(cameras, self.camera_id, track.value())?;
            }
            CameraAnimationTarget::Up(track) => {
                let up = track.value().normalize();
                let up = if up == Vec3::ZERO { Vec3::Y } else { up };
                apply_up(cameras, self.camera_id, up)?;
            }
            CameraAnimationTarget::OrthographicBounds(track) => {
                let camera = cameras
                    .orthographic_mut(self.camera_id)
                    .ok_or(ValidationError::InvalidId)?;
                let bounds = track.value();
                camera.left = bounds.left;
                camera.right = bounds.right;
                camera.bottom = bounds.bottom;
                camera.top = bounds.top;
            }
        }
        Ok(self.target.is_complete())
    }
}

fn apply_position(
    cameras: &mut impl CameraStoreMut,
    id: CameraId,
    value: Vec3,
) -> Result<(), ValidationError> {
    if let Some(camera) = cameras.perspective_mut(id) {
        camera.position = value;
        return Ok(());
    }
    if let Some(camera) = cameras.orthographic_mut(id) {
        camera.position = value;
        return Ok(());
    }
    Err(ValidationError::InvalidId)
}

fn apply_target(
    cameras: &mut impl CameraStoreMut,
    id: CameraId,
    value: Vec3,
) -> Result<(), ValidationError> {
    if let Some(camera) = cameras.perspective_mut(id) {
        camera.target = value;
        return Ok(());
    }
    if let Some(camera) = cameras.orthographic_mut(id) {
        camera.target = value;
        return Ok(());
    }
    Err(ValidationError::InvalidId)
}

fn apply_up(
    cameras: &mut impl CameraStoreMut,
    id: CameraId,
    value: Vec3,
) -> Result<(), ValidationError> {
    if let Some(camera) = cameras.perspective_mut(id) {
        camera.up = value;
        return Ok(());
    }
    if let Some(camera) = cameras.orthographic_mut(id) {
        camera.up = value;
        return Ok(());
    }
    Err(ValidationError::InvalidId)
}
