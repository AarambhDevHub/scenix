use alloc::collections::BTreeMap;

use scenix_core::{Color, MaterialId, ValidationError};
use scenix_material::{AlphaMode, PbrMaterial};

use crate::{ColorTrack, ScalarTrack, Vec3Track};

/// Mutable PBR material lookup used by material animators.
pub trait PbrMaterialStoreMut {
    /// Returns a mutable PBR material for `id`.
    fn pbr_material_mut(&mut self, id: MaterialId) -> Option<&mut PbrMaterial>;
}

impl PbrMaterialStoreMut for BTreeMap<MaterialId, PbrMaterial> {
    #[inline]
    fn pbr_material_mut(&mut self, id: MaterialId) -> Option<&mut PbrMaterial> {
        self.get_mut(&id)
    }
}

/// PBR material fields that can be animated.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MaterialAnimationTarget {
    /// Base color.
    Albedo(ColorTrack),
    /// Base color alpha.
    Opacity(ScalarTrack),
    /// Emissive RGB color.
    Emissive(Vec3Track),
    /// Roughness factor.
    Roughness(ScalarTrack),
    /// Metallic factor.
    Metallic(ScalarTrack),
}

impl MaterialAnimationTarget {
    /// Advances the contained track.
    pub fn update(&mut self, dt: f32) -> bool {
        match self {
            Self::Albedo(track) => track.update(dt),
            Self::Opacity(track) | Self::Roughness(track) | Self::Metallic(track) => {
                track.update(dt)
            }
            Self::Emissive(track) => track.update(dt),
        }
    }

    /// Returns whether the contained track has completed.
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Albedo(track) => track.is_complete(),
            Self::Opacity(track) | Self::Roughness(track) | Self::Metallic(track) => {
                track.is_complete()
            }
            Self::Emissive(track) => track.is_complete(),
        }
    }
}

/// Applies an animation track to a PBR material.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialAnimator {
    /// Target material ID.
    pub material_id: MaterialId,
    /// Field being animated.
    pub target: MaterialAnimationTarget,
}

impl MaterialAnimator {
    /// Creates a material animator.
    #[inline]
    pub const fn new(material_id: MaterialId, target: MaterialAnimationTarget) -> Self {
        Self {
            material_id,
            target,
        }
    }

    /// Advances the animator, applies the current value, and returns completion.
    pub fn update(
        &mut self,
        dt: f32,
        materials: &mut impl PbrMaterialStoreMut,
    ) -> Result<bool, ValidationError> {
        self.target.update(dt);
        let material = materials
            .pbr_material_mut(self.material_id)
            .ok_or(ValidationError::InvalidId)?;

        match &self.target {
            MaterialAnimationTarget::Albedo(track) => {
                material.albedo = track.value();
            }
            MaterialAnimationTarget::Opacity(track) => {
                let opacity = track.value().clamp(0.0, 1.0);
                material.albedo = Color::rgba(
                    material.albedo.r,
                    material.albedo.g,
                    material.albedo.b,
                    opacity,
                );
                if opacity < 1.0 {
                    material.alpha_mode = AlphaMode::Blend;
                } else if material.alpha_mode == AlphaMode::Blend {
                    material.alpha_mode = AlphaMode::Opaque;
                }
            }
            MaterialAnimationTarget::Emissive(track) => {
                material.emissive = track.value();
            }
            MaterialAnimationTarget::Roughness(track) => {
                material.roughness = track.value().clamp(0.0, 1.0);
            }
            MaterialAnimationTarget::Metallic(track) => {
                material.metallic = track.value().clamp(0.0, 1.0);
            }
        }

        Ok(self.target.is_complete())
    }
}
