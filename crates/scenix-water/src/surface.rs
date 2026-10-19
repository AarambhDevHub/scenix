//! Configurable water surfaces and their deformed preview geometry.
//!
//! A [`WaterSurface`] owns the wave field, foam parameters, plane height,
//! and the deep/shallow tint pair the renderer water pipeline expects. It
//! can also bake a CPU geometry snapshot for previews, physics proxies, and
//! headless tests; live rendering displaces in the vertex shader.

use alloc::vec::Vec;

use scenix_core::Color;
use scenix_math::{Vec2, Vec3};
use scenix_mesh::Geometry;

use crate::foam::FoamParams;
use crate::fresnel::FresnelParams;
use crate::underwater::UnderwaterView;
use crate::waves::WaveField;

/// Placement and tessellation knobs for one rectangular water patch.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaterConfig {
    /// Centered patch width along X in meters.
    pub width: f32,
    /// Centered patch depth along Z in meters.
    pub depth: f32,
    /// Grid resolution per axis (`3..=256`); `N` yields `N+1` vertices per
    /// axis and `N²` quads.
    pub segments: u32,
    /// Resting plane height in meters.
    pub plane_height: f32,
}

impl WaterConfig {
    /// Creates a square patch config with clamped segments.
    #[inline]
    #[must_use]
    pub fn new(width: f32, depth: f32, segments: u32, plane_height: f32) -> Self {
        Self {
            width: width.max(1.0),
            depth: depth.max(1.0),
            segments: segments.clamp(3, 256),
            plane_height,
        }
    }

    /// World-space bounds `(min_xz, max_xz)` of the patch.
    #[inline]
    #[must_use]
    pub fn world_extents(&self) -> (Vec2, Vec2) {
        (
            Vec2::new(-self.width * 0.5, -self.depth * 0.5),
            Vec2::new(self.width * 0.5, self.depth * 0.5),
        )
    }
}

/// A deformable, foam-aware water plane.
///
/// The CPU snapshot ( [`WaterSurface::geometry` ] ) drives previews and
/// headless tests; the renderer displaces identically in its vertex shader
/// using the same wave parameters packed into the water uniform.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaterSurface {
    /// Patch placement and tessellation.
    pub config: WaterConfig,
    /// Wave stack evaluated at sample time.
    pub waves: WaveField,
    /// Crest foam mapping.
    pub foam: FoamParams,
    /// Fresnel reflectance parameters.
    pub fresnel: FresnelParams,
    /// Below-surface view tinting.
    pub underwater: UnderwaterView,
    /// Deep-water absorption tint.
    pub deep_color: Color,
    /// Shallow-water tint mixed near reflection grazing angles.
    pub shallow_color: Color,
    /// Reflection strength `0..=1` (scales the mirror texture mix).
    pub reflection_strength: f32,
    /// Refraction strength `0..=1` (scales distortion when sampling the
    /// refraction target).
    pub refraction_strength: f32,
}

impl WaterSurface {
    /// A default 64×64 m, 32-segment patch supplied with `waves`.
    #[must_use]
    pub fn new(waves: WaveField) -> Self {
        Self {
            config: WaterConfig::new(64.0, 64.0, 32, 0.0),
            waves,
            foam: FoamParams::shoreline(),
            fresnel: FresnelParams::water(),
            underwater: UnderwaterView::clear_pool(),
            deep_color: Color::rgb(0.02, 0.13, 0.18),
            shallow_color: Color::rgb(0.12, 0.42, 0.42),
            reflection_strength: 0.85,
            refraction_strength: 0.35,
        }
    }

    /// Returns the surface with a new placement config.
    #[inline]
    #[must_use]
    pub const fn with_config(mut self, config: WaterConfig) -> Self {
        self.config = config;
        self
    }

    /// Returns the surface with new foam parameters.
    #[inline]
    #[must_use]
    pub const fn with_foam(mut self, foam: FoamParams) -> Self {
        self.foam = foam;
        self
    }

    /// Returns the surface with a tint pair `(deep, shallow)`.
    #[inline]
    #[must_use]
    pub const fn with_tint(mut self, deep_color: Color, shallow_color: Color) -> Self {
        self.deep_color = deep_color;
        self.shallow_color = shallow_color;
        self
    }

    /// Returns the surface with strength knobs `(reflection, refraction)`.
    #[inline]
    #[must_use]
    pub fn with_strengths(mut self, reflection: f32, refraction: f32) -> Self {
        self.reflection_strength = reflection.clamp(0.0, 1.0);
        self.refraction_strength = refraction.clamp(0.0, 1.0);
        self
    }

    /// Surface height at `(x, z)` in meters (waves + plane height).
    #[must_use]
    #[inline]
    pub fn height_at(&self, x: f32, z: f32, time: f32) -> f32 {
        self.config.plane_height + self.waves.height(Vec2::new(x, z), time)
    }

    /// Resting-plane foam mask at `(x, z)`.
    #[must_use]
    #[inline]
    pub fn foam_at(&self, x: f32, z: f32, time: f32) -> f32 {
        let sample = self.waves.evaluate(Vec2::new(x, z), time);
        self.foam.mask(sample.crest)
    }

    /// Bakes the deformed patch into a CPU geometry (preview/physics).
    ///
    /// Vertices run row-major facing +Y; normals reconstruct from the wave
    /// field; UVs normalize the patch into `[0, 1]`² for texture mapping.
    #[must_use]
    pub fn geometry(&self, time: f32) -> Geometry {
        let min = self.config.world_extents().0;
        let segments = self.config.segments;
        let step_x = self.config.width / segments as f32;
        let step_z = self.config.depth / segments as f32;

        let mut positions = Vec::with_capacity(((segments + 1) * (segments + 1)) as usize);
        let mut normals = Vec::with_capacity(positions.capacity());
        let mut uvs = Vec::with_capacity(positions.capacity());
        for row in 0..=segments {
            for column in 0..=segments {
                let x = min.x + column as f32 * step_x;
                let z = min.y + row as f32 * step_z;
                let sample = self.waves.evaluate(Vec2::new(x, z), time).position;
                positions.push(Vec3::new(x, self.config.plane_height + sample.y, z));
                let wave = self.waves.evaluate(Vec2::new(x, z), time);
                normals.push(wave.normal);
                uvs.push(scenix_math::Vec2::new(
                    column as f32 / segments as f32,
                    row as f32 / segments as f32,
                ));
            }
        }

        let mut indices = Vec::with_capacity((segments * segments * 6) as usize);
        let stride = segments + 1;
        for row in 0..segments {
            for column in 0..segments {
                let top_left = row * stride + column;
                let top_right = top_left + 1;
                let bottom_left = top_left + stride;
                let bottom_right = bottom_left + 1;
                indices.extend_from_slice(&[
                    top_left,
                    bottom_left,
                    top_right,
                    top_right,
                    bottom_left,
                    bottom_right,
                ]);
            }
        }

        Geometry {
            positions,
            normals,
            uvs,
            uvs2: Vec::new(),
            colors: Vec::new(),
            indices,
            tangents: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{WaterConfig, WaterSurface};
    use crate::waves::WaveField;

    #[test]
    fn surface_height_tracks_waves_and_plane() {
        let surface = WaterSurface::new(WaveField::ocean_shore())
            .with_config(WaterConfig::new(16.0, 16.0, 8, 2.0));
        let center = surface.height_at(0.0, 0.0, 0.4);
        let sample = surface
            .waves
            .evaluate(scenix_math::Vec2::new(0.0, 0.0), 0.4)
            .position
            .y;
        assert!((center - (2.0 + sample)).abs() < 1.0e-5);
        assert!(surface.foam_at(0.0, 0.0, 0.4) >= 0.0);
    }

    #[test]
    fn geometry_bakes_a_displaced_grid() {
        let surface = WaterSurface::new(WaveField::ocean_shore())
            .with_config(WaterConfig::new(20.0, 20.0, 4, 0.0));
        let geometry = surface.geometry(1.1);
        assert_eq!(geometry.vertex_count(), 25);
        assert_eq!(geometry.triangle_count(), 32);
        assert_eq!(geometry.uvs.len(), 25);

        let heights: Vec<f32> = geometry.positions.iter().map(|p| p.y).collect();
        let (min, max) = heights
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), h| {
                (a.min(*h), b.max(*h))
            });
        assert!(max - min > 0.1, "waves displaced the surface");
        // Ocean-shore amplitudes peak near ±4 m; keep a sane band with margin.
        assert!(max <= 5.0 && min >= -5.0, "surface stayed in a sane band");
    }

    #[test]
    fn strength_knobs_clamp() {
        let surface = WaterSurface::new(WaveField::calm()).with_strengths(4.0, -4.0);
        assert_eq!(surface.reflection_strength, 1.0);
        assert_eq!(surface.refraction_strength, 0.0);
    }
}
