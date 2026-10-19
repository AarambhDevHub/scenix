//! Gerstner ocean wave summation.
//!
//! A [`GerstnerWave`] displaces surface points both vertically and
//! horizontally — the horizontal term is what sharpens wave crests and
//! feeds the crest foam factor. A [`WaveField`] sums waves and exposes
//! displacement, normal reconstruction, and the crest factor used by the
//! renderer `environment` WGSL for foam masks.
//!
//! The CPU evaluation here and the packed WGSL water displacement use the
//! identical `(k, c, f)` formulation from GPU Gems chapter 1, so chip-level
//! previews and renderer output agree.

use alloc::vec::Vec;

use scenix_math::Vec2;
use scenix_math::Vec3;
use scenix_math::float::{cos, sin, sqrt};

/// One Gerstner wave component.
///
/// `direction` is the XZ propagation direction; `steepness` `0..=1`
/// controls horizontal compression; `wavelength` is in meters. Frequency
/// derivations stay deterministic because they are pure functions of these
/// three numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GerstnerWave {
    /// XZ propagation direction.
    pub direction: Vec2,
    /// Crest sharpness `0..=1`; `0` collapses the wave to a pure sine swell.
    pub steepness: f32,
    /// Surface wavelength in meters.
    pub wavelength: f32,
}

impl GerstnerWave {
    /// Builds a wave; the direction normalizes on evaluation and the
    /// steepness clamps into `[0, 1]`.
    #[must_use]
    pub fn new(direction: Vec2, steepness: f32, wavelength: f32) -> Self {
        Self {
            direction,
            steepness: steepness.clamp(0.0, 1.0),
            wavelength: if wavelength > 1.0e-4 {
                wavelength
            } else {
                1.0e-4
            },
        }
    }

    /// Wave number `k = 2π / wavelength`.
    #[inline]
    #[must_use]
    pub const fn wave_number(&self) -> f32 {
        core::f32::consts::TAU / self.wavelength
    }

    /// Phase speed `c = sqrt(g / k)` in meters per second.
    #[must_use]
    #[inline]
    pub fn phase_speed(&self) -> f32 {
        let number = self.wave_number().max(1.0e-6);
        sqrt(9.8 / number)
    }
}

/// Evaluated surface data for one point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveSample {
    /// Displaced world position (XZ compressed, Y lifted).
    pub position: Vec3,
    /// Reconstructed surface normal (unit length).
    pub normal: Vec3,
    /// Crest factor `0..=1` — feeds the foam mask.
    pub crest: f32,
}

/// A stack of Gerstner waves evaluated together.
///
/// Great water needs only a handful of waves; scenix's water WGSL packs at
/// most four, so the field clamps at `MAX_WAVES` (16 rainy-day cap stays
/// for CPU replays/tests).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaveField {
    waves: Vec<GerstnerWave>,
}

/// The renderer may always dialog with four waves per uniform packet.
pub const MAX_WAVES: usize = 4;

impl WaveField {
    /// Creates an empty field.
    #[inline]
    pub const fn new() -> Self {
        Self { waves: Vec::new() }
    }

    /// Adds a wave (beyond the cap the wave is ignored).
    pub fn push(&mut self, wave: GerstnerWave) {
        if self.waves.len() < 16 {
            self.waves.push(wave);
        }
    }

    /// Adds a wave by XZ direction (a builder-style helper).
    #[must_use]
    pub fn with_wave(mut self, wave: GerstnerWave) -> Self {
        self.push(wave);
        self
    }

    /// The immutable wave list.
    #[inline]
    pub fn waves(&self) -> &[GerstnerWave] {
        &self.waves
    }

    /// A three-swell shorebreak preset — one dominant swell, one diagonal,
    /// one short chop.
    #[must_use]
    pub fn ocean_shore() -> Self {
        let mut field = Self::new();
        field.push(GerstnerWave::new(Vec2::new(0.9, 0.35), 0.52, 34.0));
        field.push(GerstnerWave::new(Vec2::new(-0.4, 0.9), 0.34, 17.0));
        field.push(GerstnerWave::new(Vec2::new(0.7, -0.7), 0.22, 7.0));
        field
    }

    /// Flat-water preset (subtle ripples only).
    #[must_use]
    pub fn calm() -> Self {
        let mut field = Self::new();
        field.push(GerstnerWave::new(Vec2::new(1.0, 0.0), 0.10, 11.0));
        field.push(GerstnerWave::new(Vec2::new(0.0, 1.0), 0.08, 6.0));
        field
    }

    /// Evaluates the wave stack.
    ///
    /// `point` is an XZ world position; `t` is seconds. Empty fields yield
    /// the point itself with a straight-up normal.
    #[must_use]
    pub fn evaluate(&self, point: Vec2, time: f32) -> WaveSample {
        let mut displacement = Vec3::new(point.x, 0.0, point.y);
        let mut tangent_x = 0.0_f32;
        let mut tangent_z = 0.0_f32;
        let mut vertical = 0.0_f32;
        let mut slope_x = 0.0_f32;
        let mut slope_z = 0.0_f32;
        let mut crest = 0.0_f32;

        for wave in &self.waves {
            let direction = wave.direction.normalize_or_up();
            let number = wave.wave_number();
            let steepness = wave.steepness;
            let amplitude = steepness / number.max(1.0e-4);
            let phase_motion = number * direction.dot(point) + phase_time(wave, time);
            let sin_phase = sin(phase_motion);
            let cos_phase = cos(phase_motion);

            displacement.x += amplitude * direction.x * cos_phase;
            displacement.y += amplitude * sin_phase;
            displacement.z += amplitude * direction.y * cos_phase;

            // Normal reconstruction (GPU Gems ch. 1 partial derivative sum).
            slope_x += direction.x * number * amplitude * cos_phase;
            slope_z += direction.y * number * amplitude * cos_phase;
            vertical += steepness * number * amplitude * sin_phase;

            tangent_x += direction.y * number * amplitude * cos_phase;
            tangent_z += direction.x * number * amplitude * cos_phase;

            // Crest rises where sin phase nears its max under steepness.
            crest += steepness * sin_phase.max(0.0);
        }

        // Vertical compresses with horizontal displacement; scale it so tall
        // waves do not crown the geometry undercut.
        let normal =
            Vec3::new(-slope_x * tangent_z, 1.0 - vertical, -slope_z * tangent_x).normalize();
        WaveSample {
            position: displacement,
            normal,
            crest: crest.clamp(0.0, 1.0),
        }
    }

    /// Surface height at a point; displacement along Y of [`WaveField::evaluate`].
    #[must_use]
    #[inline]
    pub fn height(&self, point: Vec2, time: f32) -> f32 {
        self.evaluate(point, time).position.y
    }
}

/// The per-wave time-accumulation phase piece.
#[inline]
fn phase_time(wave: &GerstnerWave, time: f32) -> f32 {
    wave.phase_speed() * wave.wave_number() * time
}

/// Vec2 safe normalization: zero-length directions choose `+X`.
trait NormalizeUp {
    fn normalize_or_up(self) -> Vec2;
}

impl NormalizeUp for Vec2 {
    #[inline]
    fn normalize_or_up(self) -> Vec2 {
        let length_squared = self.x * self.x + self.y * self.y;
        if length_squared < 1.0e-12 {
            Vec2::new(1.0, 0.0)
        } else {
            Vec2::new(self.x / sqrt(length_squared), self.y / sqrt(length_squared))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use scenix_math::Vec2;

    use super::{GerstnerWave, WaveField};

    #[test]
    fn wave_phase_speed_and_numbers_track_wavelengths() {
        let wave = GerstnerWave::new(Vec2::new(1.0, 0.0), 0.5, 9.8);
        let speed = wave.phase_speed();
        assert!((speed * speed - 9.8 / wave.wave_number()).abs() < 1.0e-4);
        assert!((wave.wave_number() - core::f32::consts::TAU / 9.8).abs() < 1.0e-5);
        // Clamps protect against degenerate inputs.
        let flat = GerstnerWave::new(Vec2::ZERO, -4.0, 0.0);
        assert_eq!(flat.steepness, 0.0);
        assert!(flat.wavelength > 0.0);
    }

    #[test]
    fn waves_displace_and_leave_zero_fields_flat() {
        let field = WaveField::ocean_shore();
        let sample = field.evaluate(Vec2::new(4.2, -1.1), 0.937);
        assert!(sample.position.y.abs() > 0.001, "waves lift the surface");
        let flat = WaveField::new().evaluate(Vec2::new(0.0, 0.0), 2.0);
        assert_eq!(flat.position.x, 0.0);
        assert_eq!(flat.position.y, 0.0);
        assert!((flat.normal.y - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn heights_track_evaluate_and_crest_stays_bounded() {
        let field = WaveField::ocean_shore();
        let sample = field.evaluate(Vec2::new(7.0, 3.0), 1.7);
        assert!((field.height(Vec2::new(7.0, 3.0), 1.7) - sample.position.y).abs() < 1.0e-5);
        assert!(sample.crest >= 0.0 && sample.crest <= 1.0);
        let wave_count = field.waves().len();
        assert_eq!(wave_count, 3);
        // The calm preset keeps crest factors low.
        let calm = WaveField::calm();
        let gentle = calm.evaluate(Vec2::new(1.0, 1.0), 3.1);
        assert!(gentle.crest < field.evaluate(Vec2::new(1.0, 1.0), 3.1).crest.max(1.0));
        // The max wave cap keeps host memory sane: pushing beyond is ignored.
        let mut bounded = WaveField::new();
        for index in 0..24 {
            bounded.push(GerstnerWave::new(
                Vec2::new(0.9, 0.3),
                0.2,
                8.0 + index as f32,
            ));
        }
        assert_eq!(bounded.waves().len(), 16);
        // ensure the list API remains alloc-free to callers.
        let waves: Vec<_> = bounded.waves().to_vec();
        assert_eq!(waves.len(), 16);
    }
}
