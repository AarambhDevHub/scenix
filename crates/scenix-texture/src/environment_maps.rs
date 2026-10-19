//! Environment sampling utilities (scenix v1.8.0 "Terrain, Sky, And Water").
//!
//! [`EquirectSampler`] reads equirectangular (lat-long) panorama pixels on
//! the CPU with the same row/column convention as
//! `scenix_sky::bake::bake_equirectangular`: row 0 is the zenith pole,
//! longitude grows from `+x` toward `+z`. [`height_to_normal_map`] derives a
//! tangent-space normal field from a height function, matching the CPU
//! normals that `scenix_terrain::Heightmap::normal_at` produces.

use alloc::vec::Vec;

use scenix_core::ValidationError;
use scenix_math::Vec3;
use scenix_math::float::{acos, atan2, sqrt};

use crate::{Texture2D, TextureFormat};

/// GPU-free bilinear reader for row-major equirectangular pixels.
///
/// Pixels are `RGBA` floats in row-major order with row 0 at the zenith
/// pole (`v = 0`), matching `scenix_sky::bake_equirectangular` and
/// `scenix_light::LightProbe::from_equirectangular_samples`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EquirectSampler {
    width: u32,
    height: u32,
    pixels: Vec<[f32; 4]>,
    intensity: f32,
}

impl EquirectSampler {
    /// Wraps panorama pixels; requires at least `2x2` texels.
    ///
    /// # Errors
    /// Returns a `ValidationError` when the sample count does not match
    /// `width * height` or the grid is degenerate.
    pub fn new(
        width: u32,
        height: u32,
        pixels: Vec<[f32; 4]>,
        intensity: f32,
    ) -> Result<Self, ValidationError> {
        if width < 2 || height < 2 {
            // Degenerate panorama sizes are out of the accepted range.
            return Err(ValidationError::OutOfRange);
        }
        let expected = (width as usize) * (height as usize);
        if pixels.len() != expected {
            return Err(ValidationError::OutOfRange);
        }
        Ok(Self {
            width,
            height,
            pixels,
            intensity: intensity.max(0.0),
        })
    }

    /// Panorama width in texels.
    #[inline]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Panorama height in texels.
    #[inline]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Bake intensity multiplier.
    #[inline]
    #[must_use]
    pub const fn intensity(&self) -> f32 {
        self.intensity
    }

    /// The wrapped panorama pixels.
    #[inline]
    #[must_use]
    pub fn pixels(&self) -> &[[f32; 4]] {
        &self.pixels
    }

    /// Bilinear RGBA sample at panorama coordinates with `u` wrapping around
    /// the seam and `v` clamped to the poles.
    #[inline]
    #[must_use]
    pub fn sample(&self, u: f32, v: f32) -> [f32; 4] {
        // Texel-center mapping keeps the poles at texel centers like a GPU
        // sampler would.
        // sx lands in [0, width) so `as u32` truncation equals floor; sy is
        // clamped below the bottom-pole texel edge. `f32::rem_euclid` is
        // std-gated, so the euclidean remainder is expressed directly: the
        // `%` remainder carries the dividend's sign, and adding a positive
        // modulus shifts negative remainders back into range.
        let scaled = u * self.width as f32 - 0.5;
        let wrapped = scaled % self.width as f32;
        let sx = if wrapped < 0.0 {
            wrapped + self.width as f32
        } else {
            wrapped
        };
        let sy = (v * self.height as f32 - 0.5).clamp(0.0, self.height as f32 - 1.0e-3);
        let x0 = sx as u32;
        let y0 = sy as u32;
        let x1 = (x0 + 1) % self.width;
        let y1 = (y0 + 1).min(self.height - 1);
        let tx = sx - x0 as f32;
        let ty = sy - y0 as f32;
        let a = self.pixels[y0 as usize * self.width as usize + x0 as usize];
        let b = self.pixels[y0 as usize * self.width as usize + x1 as usize];
        let c = self.pixels[y1 as usize * self.width as usize + x0 as usize];
        let d = self.pixels[y1 as usize * self.width as usize + x1 as usize];
        let mut out = [0.0; 4];
        for (channel, value) in out.iter_mut().enumerate() {
            let top = a[channel] + (b[channel] - a[channel]) * tx;
            let bottom = c[channel] + (d[channel] - c[channel]) * tx;
            *value = top + (bottom - top) * ty;
        }
        out
    }

    /// Samples the panorama along a normalized direction using the
    /// lat-long convention of the bakers (`v = 0` zenith, `atan2(z, x)`).
    #[must_use]
    pub fn sample_direction(&self, direction: Vec3) -> [f32; 4] {
        let length_squared = direction.length_squared();
        if length_squared < 1.0e-12 {
            return [0.0; 4];
        }
        let scale = 1.0 / sqrt(length_squared);
        let x = direction.x * scale;
        let y = direction.y * scale;
        let z = direction.z * scale;
        // acos on the clamped dot with +y inverts the bake latitude.
        let latitude = acos(y.clamp(-1.0, 1.0));
        let v = 0.5 - latitude / core::f32::consts::PI;
        let longitude = atan2(z, x);
        let u = if longitude < 0.0 {
            (longitude + core::f32::consts::TAU) / core::f32::consts::TAU
        } else {
            longitude / core::f32::consts::TAU
        };
        self.sample(u, v)
    }

    /// Flattens the panorama into an `Rgba8Unorm` [`Texture2D`] suitable for
    /// GPU upload; channel values clamp into `0..=1`.
    #[must_use]
    pub fn to_texture_2d(&self) -> Texture2D {
        let mut data = Vec::with_capacity(self.pixels.len() * 4);
        for pixel in &self.pixels {
            for channel in pixel {
                data.push((clamp01(*channel) * 255.0 + 0.5) as u8);
            }
        }
        Texture2D::new(self.width, self.height, TextureFormat::Rgba8Unorm, data)
            .expect("generated byte count matches the grid")
    }
}

/// Derives unit tangent-space normals from a height field with central
/// differences.
///
/// `sample(x, y)` reads the height at integer lattice coordinates; out of
/// range probes are the caller's business, adjacent reads stay inside the
/// grid. `spacing` is the world distance between texels and `strength`
/// scales the horizontal slope before the surface normal is re-normalized
/// (the scenix convention: `n = normalize(-dh/dx, 1, -dh/dz)`).
#[must_use]
pub fn height_to_normal_map(
    width: u32,
    height: u32,
    spacing: f32,
    strength: f32,
    sample: impl Fn(u32, u32) -> f32,
) -> Vec<Vec3> {
    let width = width.max(1);
    let height = height.max(1);
    let spacing = if spacing > 1.0e-6 { spacing } else { 1.0e-6 };
    let strength = if strength >= 0.0 { strength } else { 0.0 };
    let mut normals = Vec::with_capacity((width * height) as usize);
    for row in 0..height {
        for column in 0..width {
            let left_x = column.saturating_sub(1);
            let right_x = (column + 1).min(width - 1);
            let up_y = row.saturating_sub(1);
            let down_y = (row + 1).min(height - 1);
            let slope_x = (sample(right_x, row) - sample(left_x, row))
                / ((right_x - left_x) as f32 * spacing);
            let slope_z = (sample(column, down_y) - sample(column, up_y))
                / ((down_y - up_y) as f32 * spacing);
            normals.push(Vec3::new(-slope_x * strength, 1.0, -slope_z * strength).normalize());
        }
    }
    normals
}

#[inline]
const fn clamp01(value: f32) -> f32 {
    if value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::{EquirectSampler, height_to_normal_map};
    use scenix_math::Vec3;

    #[test]
    fn sampler_round_trips_directions_to_pixels() {
        // 8x2 grid: a smooth horizontal ramp repeated on both rows.
        let mut pixels = vec![[0.0; 4]; 16];
        for (index, pixel) in pixels.iter_mut().enumerate() {
            pixel[0] = (index % 8) as f32 / 8.0;
        }
        let sampler = EquirectSampler::new(8, 2, pixels, 1.0).expect("valid grid");

        // +z points along the horizontal midline, landing in the upper
        // longitude half.
        let at = sampler.sample_direction(Vec3::new(0.0, 0.0, 1.0));
        // +z sits a quarter turn in: interpolating texels 1 and 2.
        assert!((at[0] - 0.1875).abs() < 1.0e-3, "got {}", at[0]);
        // Zenith direction reads the top pole without panicking.
        let sky_high = sampler.sample_direction(Vec3::new(0.0, 1.0, 0.0));
        assert!(sky_high[3] >= 0.0);
        // Degraders stay honest: zero direction is transparent black.
        assert_eq!(sampler.sample_direction(Vec3::ZERO), [0.0; 4]);
        // Bilinear sampling stays within the pixel value range.
        let between = sampler.sample(0.5, 0.5);
        assert!(between[0] >= 0.0 && between[0] <= 1.0, "got {}", between[0]);
        assert_eq!(sampler.intensity(), 1.0);
    }

    #[test]
    fn sampler_rejects_bad_grids() {
        let err = EquirectSampler::new(1, 2, vec![[0.0; 4]; 2], 1.0);
        assert!(err.is_err(), "1x2 grids are degenerate");
        let mismatch = EquirectSampler::new(2, 2, vec![[0.0; 4]; 3], 1.0);
        assert!(mismatch.is_err(), "pixel counts must match");
    }

    #[test]
    fn height_normals_tilt_away_from_slopes() {
        // A plane ramping up along +x tilts normals toward -x.
        let normals = height_to_normal_map(4, 4, 1.0, 1.0, |x, _y| x as f32 * 0.5);
        let top_right = normals[3];
        assert!(top_right.y > 0.8, "surface normal keeps the y component");
        assert!(top_right.x < 0.0, "ramp along +x pushes the normal to -x");
        let flat = height_to_normal_map(3, 3, 1.0, 4.0, |_, _| 0.0);
        assert_eq!(flat[4], Vec3::new(0.0, 1.0, 0.0));
        // Unit output everywhere.
        for normal in normals.iter().take(16) {
            assert!((normal.length() - 1.0).abs() < 1.0e-4);
        }
    }
}
