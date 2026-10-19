//! Sun position helpers: elevation/azimuth representation packed into a
//! [`SunLight`] and the scene light facets scenix-sky exposes.

use scenix_light::{DirectionalLight, SunLight, sun_direction};
use scenix_math::Vec3;

/// Sun elevation/azimuth pair (radians).
///
/// Degrees helpers make tooling ergonomics easy; radians drive the math and
/// the GPU uniform.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SunPosition {
    /// Elevation above the horizon in radians.
    pub elevation: f32,
    /// Azimuth around `+Y` in radians.
    pub azimuth: f32,
}

impl SunPosition {
    /// Creates a radians-based position.
    #[inline]
    pub const fn radians(elevation: f32, azimuth: f32) -> Self {
        Self { elevation, azimuth }
    }

    /// Creates a degrees-based position; angles convert to radians.
    #[must_use]
    pub fn degrees(elevation_deg: f32, azimuth_deg: f32) -> Self {
        Self {
            elevation: elevation_deg.to_radians(),
            azimuth: azimuth_deg.to_radians(),
        }
    }

    /// The unit direction vector shining from the sun toward the sky.
    #[must_use]
    #[inline]
    pub fn direction(&self) -> Vec3 {
        sun_direction(self.elevation, self.azimuth)
    }

    /// Combined "sun and sky" descriptor: a [`SunLight`] plus a panic-free
    /// [`DirectionalLight`] conversion for registration.
    #[must_use]
    pub fn sun_light(&self, intensity: f32) -> SunLight {
        SunLight::from_direction(self.direction(), intensity)
    }

    /// Converts into a registered directional light directly.
    #[must_use]
    pub fn directional_light(&self, intensity: f32) -> DirectionalLight {
        self.sun_light(intensity).to_directional_light()
    }
}

#[cfg(test)]
mod tests {
    use super::SunPosition;

    #[test]
    fn degree_and_radian_positions_agree() {
        let deg = SunPosition::degrees(30.0, 60.0);
        let rad = SunPosition::radians(core::f32::consts::FRAC_PI_6, core::f32::consts::FRAC_PI_3);
        // Elevation 30 degrees maps exactly to FRAC_PI_6; azimuth 60 deg to
        // FRAC_PI_3 — both are exact float binaries, so compare tightly.
        assert!((deg.direction().y - rad.direction().y).abs() < 1.0e-6);
        assert!((deg.direction().x - rad.direction().x).abs() < 1.0e-6);
    }

    #[test]
    fn sun_position_builds_light_state() {
        let position = SunPosition::radians(0.75, 1.2);
        let sun = position.sun_light(2.4);
        assert!((sun.direction.length() - 1.0).abs() < 1.0e-5);
        assert!((sun.intensity - 2.4).abs() < 1.0e-6);
        let directional = position.directional_light(2.4);
        assert!((directional.intensity - 2.4).abs() < 1.0e-6);
        assert_eq!(directional.color, sun.color);
    }
}
