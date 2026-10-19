//! CPU environment-map baking from the procedural [`SkyModel`].
//!
//! Bakes produce three outputs scenix uses directly:
//!
//! * an equirectangular RGB latticed row set (feeds texture uploads and the
//!   scene inspector previews),
//! * a [`LightProbe`] SH projection via `scenix-light`'s existing
//!   coefficients path, and
//! * a small "sky lighter" color summary for ambient hints.

use alloc::vec::Vec;

use scenix_light::LightProbe;
use scenix_math::Vec3;
use scenix_math::float::{cos, sin};

use crate::sky::SkyModel;
use crate::sun::SunPosition;

/// Bakes the model into an equirectangular lat/long grid.
///
/// Rows count latitude from the north pole; columns go from -X around
/// counterclockwise. Row-major `width × height` layout matches scenix
/// texture uploads; values are simple linear RGB (`f32` channels).
#[must_use]
pub fn bake_equirectangular(
    model: &SkyModel,
    sun: &SunPosition,
    width: u32,
    height: u32,
) -> Vec<[f32; 3]> {
    let width = width.max(4);
    let height = height.max(4);
    let sun_direction = sun.direction();
    let mut samples = Vec::with_capacity((width * height) as usize);
    for row in 0..height {
        // Latitude from the top down: v=0 → straight up.
        let pitch = (row as f32 + 0.5) / height as f32;
        let latitude = core::f32::consts::FRAC_PI_2 - pitch * core::f32::consts::PI;
        for column in 0..width {
            let longitude = column as f32 / width as f32 * core::f32::consts::TAU;
            let horizontal = cos(latitude).max(0.0);
            let direction = Vec3::new(
                horizontal * cos(longitude),
                sin(latitude),
                horizontal * sin(longitude),
            );
            samples.push(model.rgb_at(direction, sun_direction));
        }
    }
    samples
}

/// Bakes SH coefficients for a light probe from the procedural sky.
///
/// # Errors
/// Returns a `ValidationError` when the `scenix-light` `SH` projection is
/// invalid for degenerate grids; grid size is clamped internally, so a
/// passing invocation is always a valid probe.
#[allow(clippy::missing_errors_doc)]
pub fn bake_light_probe(
    model: &SkyModel,
    sun: &SunPosition,
    width: u32,
    height: u32,
    intensity: f32,
) -> Result<LightProbe, scenix_core::ValidationError> {
    let width = width.clamp(8, 32);
    let height = height.clamp(8, 32);
    let samples = bake_equirectangular(model, sun, width, height);
    let radian_samples: Vec<Vec3> = samples
        .iter()
        .map(|sample| Vec3::new(sample[0], sample[1], sample[2]))
        .collect();
    LightProbe::from_equirectangular_samples(&radian_samples, width, height, intensity)
}

/// One ambient color + intensity summary of a baked sky.
///
/// Editors and examples show the average sky colors split hemisphere-by-
/// hemisphere so a matching `scenix_light::HemisphereLight` gets sensible
/// defaults without an operator guessing values.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkyAmbient {
    /// Average upper-hemisphere radiance.
    pub sky: [f32; 3],
    /// Average lower-hemisphere radiance (ground bounce).
    pub ground: [f32; 3],
}

/// Summarizes a baked equirect map into hemisphere averages.
#[must_use]
pub fn bake_ambient(samples: &[[f32; 3]], width: u32, height: u32) -> SkyAmbient {
    if width == 0 || height == 0 || samples.is_empty() {
        return SkyAmbient {
            sky: [0.0; 3],
            ground: [0.0; 3],
        };
    }
    let half_rows = (height / 2).max(1) as usize;
    let mut sky = [0.0_f32; 3];
    let mut ground = [0.0_f32; 3];
    let mut sky_cells = 0.0_f32;
    let mut ground_cells = 0.0_f32;
    for (index, sample) in samples.iter().enumerate() {
        let row = index / width as usize;
        let is_sky = row < half_rows;
        if is_sky {
            sky_cells += 1.0;
            for (channel, value) in sky.iter_mut().zip(sample) {
                *channel += value;
            }
        } else {
            ground_cells += 1.0;
            for (channel, value) in ground.iter_mut().zip(sample) {
                *channel += value;
            }
        }
    }
    if sky_cells > 0.0 {
        for channel in &mut sky {
            *channel /= sky_cells;
        }
    }
    if ground_cells > 0.0 {
        for channel in &mut ground {
            *channel /= ground_cells;
        }
    }
    SkyAmbient { sky, ground }
}

#[cfg(test)]
mod tests {
    use super::{SkyAmbient, bake_ambient, bake_equirectangular, bake_light_probe};
    use crate::sky::SkyModel;
    use crate::sun::SunPosition;

    #[test]
    fn equirect_bake_is_row_ordered_and_deterministic() {
        let model = SkyModel::clear_day();
        let sun = SunPosition::radians(0.8, 1.1);
        let a = bake_equirectangular(&model, &sun, 9, 5);
        let b = bake_equirectangular(&model, &sun, 9, 5);
        assert_eq!(a.len(), 45);
        assert_eq!(a, b, "equal inputs replay exactly");

        // The very first row is upper-hemisphere blue.
        assert!(a[0][2] > a[0][0]);
        // The last row is below-horizon (clamped sky view).
        assert!(a.last().expect("rows")[1] <= a[0][1] + 0.5);
    }

    #[test]
    fn light_probe_projection_succeeds_for_bakes() {
        let model = SkyModel::clear_day();
        let sun = SunPosition::radians(1.0, 0.4);
        let probe = bake_light_probe(&model, &sun, 16, 16, 1.4).expect("valid probe");
        // Non-zero SH taps prove the baked equirect map fed real coefficients.
        let visited: f32 = probe
            .sh_coefficients
            .iter()
            .map(|c| c.x.abs() + c.y.abs() + c.z.abs())
            .sum();
        assert!(visited > 0.0, "SH projection must carry sky radiance");
    }

    #[test]
    fn ambient_summary_splits_sky_and_ground() {
        let model = SkyModel::clear_day();
        let sun = SunPosition::radians(1.0, 0.0);
        let samples = bake_equirectangular(&model, &sun, 8, 8);
        let summary: SkyAmbient = bake_ambient(&samples, 8, 8);
        assert!(summary.sky.iter().sum::<f32>() > 0.0);
        assert!(summary.ground.iter().sum::<f32>() > 0.0);
    }
}
