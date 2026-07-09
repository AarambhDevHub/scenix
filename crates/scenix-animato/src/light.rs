//! Light animation targets for the procedural driver and the mixer.

use alloc::collections::BTreeMap;

use scenix_core::{LightId, ValidationError};
use scenix_light::{DirectionalLight, PointLight, SpotLight};

use crate::{ColorTrack, ScalarTrack};

/// Mutable light lookup used by light animators.
pub trait LightStoreMut {
    /// Returns a mutable point light, when present.
    fn point_mut(&mut self, _id: LightId) -> Option<&mut PointLight> {
        None
    }
    /// Returns a mutable spot light, when present.
    fn spot_mut(&mut self, _id: LightId) -> Option<&mut SpotLight> {
        None
    }
    /// Returns a mutable directional light, when present.
    fn directional_mut(&mut self, _id: LightId) -> Option<&mut DirectionalLight> {
        None
    }
}

impl LightStoreMut for BTreeMap<LightId, PointLight> {
    #[inline]
    fn point_mut(&mut self, id: LightId) -> Option<&mut PointLight> {
        self.get_mut(&id)
    }
}
impl LightStoreMut for BTreeMap<LightId, SpotLight> {
    #[inline]
    fn spot_mut(&mut self, id: LightId) -> Option<&mut SpotLight> {
        self.get_mut(&id)
    }
}
impl LightStoreMut for BTreeMap<LightId, DirectionalLight> {
    #[inline]
    fn directional_mut(&mut self, id: LightId) -> Option<&mut DirectionalLight> {
        self.get_mut(&id)
    }
}

/// Borrowed point/spot/directional light maps.
pub struct LightStores<'a> {
    /// Point lights by ID.
    pub point: &'a mut BTreeMap<LightId, PointLight>,
    /// Spot lights by ID.
    pub spot: &'a mut BTreeMap<LightId, SpotLight>,
    /// Directional lights by ID.
    pub directional: &'a mut BTreeMap<LightId, DirectionalLight>,
}

impl LightStoreMut for LightStores<'_> {
    #[inline]
    fn point_mut(&mut self, id: LightId) -> Option<&mut PointLight> {
        self.point.get_mut(&id)
    }
    #[inline]
    fn spot_mut(&mut self, id: LightId) -> Option<&mut SpotLight> {
        self.spot.get_mut(&id)
    }
    #[inline]
    fn directional_mut(&mut self, id: LightId) -> Option<&mut DirectionalLight> {
        self.directional.get_mut(&id)
    }
}

/// Light fields that can be animated.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightAnimationTarget {
    /// Light color.
    Color(ColorTrack),
    /// Scalar intensity.
    Intensity(ScalarTrack),
    /// Maximum range (point/spot only).
    Range(ScalarTrack),
    /// Spot outer cone half-angle in radians (spot only).
    SpotAngle(ScalarTrack),
}

impl LightAnimationTarget {
    /// Advances the contained track and returns whether it is still running.
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::Color(t) => t.update(dt),
            Self::Intensity(t) | Self::Range(t) | Self::SpotAngle(t) => t.update(dt),
        }
    }
    /// Returns whether the contained track has completed.
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Color(t) => t.is_complete(),
            Self::Intensity(t) | Self::Range(t) | Self::SpotAngle(t) => t.is_complete(),
        }
    }
}

/// Applies a procedural track to a light store entry.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightAnimator {
    /// Target light ID.
    pub light_id: LightId,
    /// Field being animated.
    pub target: LightAnimationTarget,
}

impl LightAnimator {
    /// Creates a light animator.
    #[inline]
    pub const fn new(light_id: LightId, target: LightAnimationTarget) -> Self {
        Self { light_id, target }
    }

    /// Advances, applies, and returns completion.
    pub fn update(
        &mut self,
        dt: f32,
        lights: &mut impl LightStoreMut,
    ) -> Result<bool, ValidationError> {
        self.target.update(dt);
        match &self.target {
            LightAnimationTarget::Color(track) => {
                let v = track.value();
                if let Some(l) = lights.point_mut(self.light_id) {
                    l.color = v;
                }
                if let Some(l) = lights.spot_mut(self.light_id) {
                    l.color = v;
                }
                if let Some(l) = lights.directional_mut(self.light_id) {
                    l.color = v;
                }
            }
            LightAnimationTarget::Intensity(track) => {
                let v = track.value().max(0.0);
                if let Some(l) = lights.point_mut(self.light_id) {
                    l.intensity = v;
                }
                if let Some(l) = lights.spot_mut(self.light_id) {
                    l.intensity = v;
                }
                if let Some(l) = lights.directional_mut(self.light_id) {
                    l.intensity = v;
                }
            }
            LightAnimationTarget::Range(track) => {
                let v = track.value().max(0.0);
                if let Some(l) = lights.point_mut(self.light_id) {
                    l.range = v;
                }
                if let Some(l) = lights.spot_mut(self.light_id) {
                    l.range = v;
                }
            }
            LightAnimationTarget::SpotAngle(track) => {
                let v = track.value().clamp(0.0, core::f32::consts::FRAC_PI_2);
                if let Some(l) = lights.spot_mut(self.light_id) {
                    l.angle = v;
                }
            }
        }
        Ok(self.target.is_complete())
    }
}
