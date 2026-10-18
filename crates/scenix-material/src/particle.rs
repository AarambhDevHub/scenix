use scenix_core::Color;

use crate::traits::double_sided_bit;
use crate::{
    AlphaMode, FEATURE_ALBEDO_TEXTURE, FEATURE_FLIPBOOK, FEATURE_SIZE_ATTENUATION,
    FEATURE_SOFT_PARTICLES, Material, PipelineAlphaMode, PipelineKey, ShaderKind,
};

/// Blend behavior for particle rendering.
///
/// Particles are usually alpha-blended, but additive glow accumulation is a
/// common alternative; both build distinct pipelines.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParticleBlend {
    /// Standard alpha blending (`src * alpha + dst * (1 - alpha)`).
    #[default]
    Alpha,
    /// Additive blending (`src + dst`) for glow, sparks, and magic effects.
    Additive,
}

impl ParticleBlend {
    /// Returns whether this blend should sort into the transparent pass.
    #[inline]
    pub const fn is_transparent(self) -> bool {
        matches!(self, Self::Alpha | Self::Additive)
    }
}

/// Flipbook animation parameters for particle atlas sampling.
///
/// A flipbook treat the particle texture as a uniform grid of frames; every
/// frame the renderer maps the particle's age to one grid cell. Pair the grid
/// with `scenix-texture` `FlipbookGrid` data when authoring atlases.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlipbookParams {
    /// Grid columns.
    pub columns: u32,
    /// Grid rows.
    pub rows: u32,
    /// Frames used before the animation stops; `0` selects `columns * rows`.
    pub frame_count: u32,
    /// Frames per second.
    pub fps: f32,
    /// Restart the animation once the last frame was sampled.
    pub looping: bool,
}

impl FlipbookParams {
    /// Creates flipbook parameters covering a full grid.
    #[inline]
    pub const fn full(columns: u32, rows: u32, fps: f32) -> Self {
        Self {
            columns,
            rows,
            frame_count: 0,
            fps,
            looping: true,
        }
    }

    /// Returns the effective frame count, clamped to the grid.
    #[inline]
    pub const fn effective_frame_count(&self) -> u32 {
        let grid = self.columns * self.rows;
        if grid == 0 {
            0
        } else if self.frame_count == 0 || self.frame_count > grid {
            grid
        } else {
            self.frame_count
        }
    }

    /// Returns the sampled frame for a particle age in seconds.
    #[inline]
    pub fn frame_at(&self, age: f32) -> u32 {
        let frames = self.effective_frame_count();
        if frames == 0 || self.fps <= 0.0 {
            return 0;
        }
        let frame = (age * self.fps) as u32;
        if self.looping {
            frame % frames
        } else {
            frame.min(frames - 1)
        }
    }
}

/// CPU-side particle material description for the v1.7 particle draw path.
///
/// The particle renderer packs this into the shared material uniform block
/// and binds the optional texture for soft sprite sampling. Particle colors
/// per instance ride the instance buffer; `color` acts as a tint.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParticleMaterial {
    /// Tint color multiplied over the instance color and texture.
    pub color: Color,
    /// Sprite texture; `None` binds the default white texture.
    pub texture: Option<TextureIdRef>,
    /// Flipbook animation over the bound texture.
    pub flipbook: Option<FlipbookParams>,
    /// Soft-edge fade (fills the sprite edge; actual depth-map softening
    /// requires a scene depth binding planned for a later release).
    pub soft_particles: bool,
    /// Whether world-space sizes attenuate with distance.
    ///
    /// The v1.7 particle WGSL always renders world-space sizes; the flag is
    /// CPU metadata mirrored into the pipeline key for tools.
    pub size_attenuation: bool,
    /// Alpha behavior.
    pub alpha_mode: AlphaMode,
    /// Blend behavior.
    pub blend: ParticleBlend,
}

// `TextureId` is re-imported through `scenix_core`; alias keeps the doc
// signature short in published docs.
use scenix_core::TextureId as TextureIdRef;

impl ParticleMaterial {
    /// Creates a default alpha-blended white particle material.
    #[inline]
    pub const fn new() -> Self {
        Self {
            color: Color::WHITE,
            texture: None,
            flipbook: None,
            soft_particles: false,
            size_attenuation: true,
            alpha_mode: AlphaMode::Blend,
            blend: ParticleBlend::Alpha,
        }
    }

    /// Returns this material with a tint color.
    #[inline]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns this material with a sprite texture.
    #[inline]
    pub fn with_texture(mut self, texture: Option<TextureIdRef>) -> Self {
        self.texture = texture;
        self
    }

    /// Returns this material with flipbook parameters.
    #[inline]
    pub fn with_flipbook(mut self, flipbook: Option<FlipbookParams>) -> Self {
        self.flipbook = flipbook;
        self
    }

    /// Returns this material with additive blending.
    #[inline]
    pub const fn additively_blended(mut self) -> Self {
        self.blend = ParticleBlend::Additive;
        self
    }

    /// Returns the renderer feature bits for this material state.
    #[inline]
    pub fn feature_bits(&self) -> u64 {
        let mut bits = double_sided_bit(true);
        if self.texture.is_some() {
            bits |= FEATURE_ALBEDO_TEXTURE;
        }
        if self.flipbook.is_some() {
            bits |= FEATURE_FLIPBOOK;
        }
        if self.soft_particles {
            bits |= FEATURE_SOFT_PARTICLES;
        }
        if self.size_attenuation {
            bits |= FEATURE_SIZE_ATTENUATION;
        }
        bits
    }
}

impl Default for ParticleMaterial {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Material for ParticleMaterial {
    #[inline]
    fn pipeline_key(&self) -> PipelineKey {
        let alpha = match self.blend {
            ParticleBlend::Additive => PipelineAlphaMode::Blend,
            ParticleBlend::Alpha => self.alpha_mode.pipeline_alpha(),
        };
        PipelineKey::new(ShaderKind::Particles, alpha, self.feature_bits())
    }

    #[inline]
    fn is_transparent(&self) -> bool {
        self.blend.is_transparent() || self.alpha_mode.is_transparent()
    }

    /// Particle sprites are camera-facing quads; back faces are visible.
    #[inline]
    fn double_sided(&self) -> bool {
        true
    }

    #[inline]
    fn alpha_cutoff(&self) -> Option<f32> {
        self.alpha_mode.cutoff()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flipbook_frame_count_and_frames_track_age() {
        let flip = FlipbookParams::full(2, 2, 4.0);
        assert_eq!(flip.effective_frame_count(), 4);
        assert_eq!(flip.frame_at(0.0), 0);
        assert_eq!(flip.frame_at(0.26), 1);
        assert_eq!(flip.frame_at(1.5), 2, "looping wraps");
    }

    #[test]
    fn particle_material_reports_particle_pipeline_bits() {
        let material = ParticleMaterial::new()
            .with_texture(Some(TextureIdRef::new(3)))
            .with_flipbook(Some(FlipbookParams::full(4, 4, 8.0)));
        let bits = material.feature_bits();
        assert!(bits & FEATURE_FLIPBOOK != 0);
        assert!(bits & FEATURE_ALBEDO_TEXTURE != 0);
        let key = material.pipeline_key();
        assert!(matches!(key.shader, ShaderKind::Particles));
        assert!(material.is_transparent());
    }
}
