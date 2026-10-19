//! Splat layers and normalized weight maps for terrain material blending.
//!
//! A [`SplatMap`] stores a weight channel per lattice cell; the four
//! channels correspond to the four terrain material layers. Shader-side the
//! weights arrive as one RGBA splat texture; CPU-side they can also ride
//! vertex colors through `TerrainGrid::chunk_geometry_with_splat`.

use alloc::vec::Vec;

use scenix_core::Color;

/// RGB splat channel count and the matching RGBA texture channels.
pub const SPLAT_LAYER_COUNT: usize = 4;

/// One terrain material layer descriptor: its texture surface look plus the
/// tiling used when the renderer repeats it across the map.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SplatLayer {
    /// Layer albedo (also the fallback when no texture is bound).
    pub albedo: Color,
    /// Surface roughness, clamped into `0.05..=1.0`.
    pub roughness: f32,
    /// Texture repeats across the whole map; `1.0` tiles once per grid cell
    /// of the splat layout, `0.0` means no repeat.
    pub tiling: f32,
}

impl SplatLayer {
    /// Creates a layer with clamped roughness and a non-negative tiling.
    #[inline]
    pub fn new(albedo: Color, roughness: f32, tiling: f32) -> Self {
        Self {
            albedo,
            roughness: roughness.clamp(0.05, 1.0),
            tiling: tiling.max(0.0),
        }
    }

    /// A mid-grass green ground layer.
    #[inline]
    #[must_use]
    pub const fn default_grass() -> Self {
        Self {
            albedo: Color::rgb(0.28, 0.42, 0.16),
            roughness: 0.92,
            tiling: 24.0,
        }
    }

    /// Gray-blue rock layer.
    #[inline]
    #[must_use]
    pub const fn default_rock() -> Self {
        Self {
            albedo: Color::rgb(0.46, 0.45, 0.43),
            roughness: 0.78,
            tiling: 18.0,
        }
    }

    /// Brown soil layer.
    #[inline]
    #[must_use]
    pub const fn default_dirt() -> Self {
        Self {
            albedo: Color::rgb(0.38, 0.28, 0.18),
            roughness: 0.85,
            tiling: 20.0,
        }
    }

    /// Pale sand layer.
    #[inline]
    #[must_use]
    pub const fn default_sand() -> Self {
        Self {
            albedo: Color::rgb(0.80, 0.72, 0.50),
            roughness: 0.95,
            tiling: 16.0,
        }
    }

    /// The four common environment layers in splat channel order.
    #[must_use]
    pub fn defaults() -> [Self; SPLAT_LAYER_COUNT] {
        [
            Self::default_grass(),
            Self::default_rock(),
            Self::default_dirt(),
            Self::default_sand(),
        ]
    }
}

/// Normalized 4-layer weight map over a rectangular lattice.
///
/// Cells are row-major like [`crate::Heightmap`]; unnormalized inputs are
/// renormalized in-place by [`SplatMap::normalize_in_place`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SplatMap {
    /// Weight map width in cells.
    pub width: u32,
    /// Weight map depth in cells.
    pub depth: u32,
    /// `[w0, w1, w2, w3]` channels per cell, row-major.
    pub weights: Vec<[f32; SPLAT_LAYER_COUNT]>,
}

impl SplatMap {
    /// Builds a splat map; every weight is clamped into `[0, 1]`.
    #[must_use]
    pub fn from_fn(
        width: u32,
        depth: u32,
        fill: impl Fn(u32, u32) -> [f32; SPLAT_LAYER_COUNT],
    ) -> Self {
        let mut weights = Vec::with_capacity((width * depth) as usize);
        for iz in 0..depth {
            for ix in 0..width {
                let channels = fill(ix, iz);
                let mut clamped = [0.0_f32; SPLAT_LAYER_COUNT];
                for (target, source) in clamped.iter_mut().zip(&channels) {
                    *target = source.clamp(0.0, 1.0);
                }
                weights.push(clamped);
            }
        }
        Self {
            width,
            depth,
            weights,
        }
    }

    /// A splat exclusively weighted onto layer 0 (grass-only mapping).
    #[must_use]
    pub fn default_flat(width: u32, depth: u32) -> Self {
        Self::from_fn(width, depth, |_, _| [1.0, 0.0, 0.0, 0.0])
    }

    /// Rescales every cell so its four weights sum to one.
    ///
    /// Zero cells fall back to layer 0 weighted at 1.0 so the shader never
    /// receives an undefined blend.
    pub fn normalize_in_place(&mut self) {
        for channels in &mut self.weights {
            let total: f32 = channels.iter().sum();
            if total <= f32::EPSILON {
                *channels = [1.0, 0.0, 0.0, 0.0];
                continue;
            }
            for weight in channels.iter_mut() {
                *weight /= total;
            }
        }
    }

    /// Bilinear splat sample at world-independent texture coordinates
    /// normalized into the map rect; off-map inputs clamp to the edge.
    #[must_use]
    pub fn sample(&self, u: f32, v: f32) -> [f32; SPLAT_LAYER_COUNT] {
        if self.width == 0 || self.depth == 0 {
            return [1.0, 0.0, 0.0, 0.0];
        }
        // scaled/fractions are non-negative, so truncation equals floor.
        let scaled_u = u.clamp(0.0, 1.0) * (self.width as f32 - 1.0);
        let scaled_v = v.clamp(0.0, 1.0) * (self.depth as f32 - 1.0);
        let fx = scaled_u as u32 as f32;
        let fz = scaled_v as u32 as f32;
        let ix = (fx as u32).min(self.width - 1);
        let iz = (fz as u32).min(self.depth - 1);
        let ix1 = (ix + 1).min(self.width - 1);
        let iz1 = (iz + 1).min(self.depth - 1);
        let tx = scaled_u - fx;
        let tz = scaled_v - fz;
        let mut blended = [0.0_f32; SPLAT_LAYER_COUNT];
        for (index, item) in blended.iter_mut().enumerate() {
            let corner = self.weights[(iz * self.width + ix) as usize][index].max(0.0);
            let corner_right = self.weights[(iz * self.width + ix1) as usize][index].max(0.0);
            let corner_down = self.weights[(iz1 * self.width + ix) as usize][index].max(0.0);
            let corner_far = self.weights[(iz1 * self.width + ix1) as usize][index].max(0.0);
            let top = corner + (corner_right - corner) * tx;
            let bottom = corner_down + (corner_far - corner_down) * tx;
            *item = (top + (bottom - top) * tz).clamp(0.0, 1.0);
        }
        let total: f32 = blended.iter().sum();
        if total <= f32::EPSILON {
            blended = [1.0, 0.0, 0.0, 0.0];
        } else {
            for item in &mut blended {
                *item /= total;
            }
        }
        blended
    }
}

#[cfg(test)]
mod tests {
    use scenix_core::Color;

    use super::{SplatLayer, SplatMap};

    #[test]
    fn default_layers_carry_valid_material_fields() {
        for layer in SplatLayer::defaults() {
            assert!(layer.roughness >= 0.05 && layer.roughness <= 1.0);
            assert!(layer.tiling > 0.0);
            assert!(layer.albedo != Color::WHITE);
        }
        assert_eq!(SplatLayer::new(Color::BLACK, 9.0, -1.0).roughness, 1.0);
        assert_eq!(SplatLayer::new(Color::BLACK, 9.0, -1.0).tiling, 0.0);
    }

    #[test]
    fn normalize_and_sample_produce_weighted_blends() {
        let mut splat = SplatMap::from_fn(2, 2, |ix, iz| {
            if ix == 0 && iz == 0 {
                [2.0, 2.0, 0.0, 0.0]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            }
        });
        splat.normalize_in_place();
        // Every other cell is an all-zero -> fallback layer 0.
        let back = splat.weights[3];
        assert_eq!(back[0], 1.0);
        let corner = splat.sample(0.0, 0.0);
        assert!((corner[0] - 0.5).abs() < 1.0e-6);
        assert!((corner[1] - 0.5).abs() < 1.0e-6);

        let flat = SplatMap::default_flat(3, 3);
        let far = flat.sample(0.9, 0.9);
        assert_eq!(far[0], 1.0);
        assert_eq!(
            flat.weights[2][0], 1.0,
            "cell 2 keeps its pure layer-0 weight"
        );
    }
}
