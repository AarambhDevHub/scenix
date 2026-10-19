//! Sun-directional light bridging for sky and environment systems
//! (scenix v1.8.0 "Terrain, Sky, And Water").
//!
//! A [`SunLight`] turns one elevation angle into the color/intensity ramp
//! scene lighting expects: near-zero elevations warm toward red, midday sun
//! cools toward pleasant white. `scenix-sky` wraps this for its procedural
//! sun helper, and callers can apply it directly to a
//! [`DirectionalLight`].

use scenix_core::Color;
use scenix_math::Vec3;
use scenix_math::float::{cos, sin, sqrt};

use crate::directional::DirectionalLight;

/// Warm spectral ramp used for low elevations.
const SUNRISE: Color = Color {
    r: 1.0,
    g: 0.42,
    b: 0.15,
    a: 1.0,
};

/// Directional sun representation derived from sky elevation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SunLight {
    /// Direction the light shines toward (normalized, pointing at ground).
    pub direction: Vec3,
    /// Physical sun disk size in radians (about `0.00935` at Earth scale).
    pub angular_radius: f32,
    /// Linear-light radiance color.
    pub color: Color,
    /// Scalar intensity multiplier.
    pub intensity: f32,
}

impl SunLight {
    /// Builds a sun from a light direction and intensity.
    ///
    /// `direction` is normalized; a zero direction falls back to straight
    /// down. The sun color follows the elevation of `direction.y`.
    #[inline]
    #[must_use]
    pub fn from_direction(direction: Vec3, intensity: f32) -> Self {
        let direction = direction.normalize();
        let direction = if direction.length_squared() < 1.0e-12 {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            direction
        };
        Self {
            color: sun_color_from_elevation(-direction.y),
            direction,
            intensity: intensity.max(0.0),
            angular_radius: 0.009_35,
        }
    }

    /// Builds a sun from elevation above (-)horizon and azimuth.
    #[inline]
    #[must_use]
    pub fn from_elevation(elevation_rad: f32, azimuth_rad: f32, intensity: f32) -> Self {
        Self::from_direction(sun_direction(elevation_rad, azimuth_rad), intensity)
    }

    /// Returns this sun with a physical angular radius.
    #[inline]
    #[must_use]
    pub const fn with_angular_radius(mut self, angular_radius: f32) -> Self {
        self.angular_radius = angular_radius.max(1.0e-5);
        self
    }

    /// Returns this sun with an explicit light color.
    #[inline]
    #[must_use]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns this sun with an intensity.
    #[inline]
    #[must_use]
    pub const fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.max(0.0);
        self
    }

    /// Converts into a directional light for registered renderer lighting.
    #[must_use]
    pub fn to_directional_light(&self) -> DirectionalLight {
        DirectionalLight::new(self.direction, self.color, self.intensity)
    }
}

/// Computes the unit direction of a sun at `(elevation, azimuth)` in
/// radians. Elevation is measured above the XZ horizon; azimuth rotates
/// clockwise ( looking down from +Y ).
#[must_use]
#[inline]
pub fn sun_direction(elevation_rad: f32, azimuth_rad: f32) -> Vec3 {
    let elevation_sin = sin(elevation_rad);
    let horizontal = 1.0 - elevation_sin * elevation_sin;
    let horizontal = sqrt(horizontal.max(0.0));
    Vec3::new(
        horizontal * cos(azimuth_rad),
        elevation_sin,
        horizontal * sin(azimuth_rad),
    )
}

/// The spectral ramp for a sun elevation angle `(radians)`.
///
/// Below the horizon the ramp saturates at the sunrise red; straight above
/// it converges to the near-white high-sun color used for noon scenes.
#[must_use]
pub fn sun_color_from_elevation(elevation_rad: f32) -> Color {
    // Negative elevations clamp so below-horizon suns stay sunrise.
    let t = (elevation_rad / (core::f32::consts::FRAC_PI_2)).clamp(-0.08, 1.0);
    let warm = (t / 0.28).clamp(0.0, 1.0);
    let noon = Color::rgb(1.0, 0.97, 0.90);
    if warm <= 0.0 {
        SUNRISE
    } else {
        SUNRISE.lerp(noon, warm)
    }
}

#[cfg(test)]
mod tests {
    use super::{SunLight, sun_color_from_elevation, sun_direction};
    use scenix_math::Vec3;

    #[test]
    fn directions_track_elevation_and_azimuth() {
        let up = sun_direction(core::f32::consts::FRAC_PI_2, 0.0);
        assert!((up.y - 1.0).abs() < 1.0e-5);
        let east = sun_direction(0.0, 0.0);
        assert!((east.x - 1.0).abs() < 1.0e-5);
        let south = sun_direction(0.0, core::f32::consts::FRAC_PI_2);
        assert!((south.z - 1.0).abs() < 1.0e-5);
        assert!((sun_direction(0.25, 0.7).length() - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn sun_colors_warm_at_low_elevations() {
        let noon = sun_color_from_elevation(core::f32::consts::FRAC_PI_2);
        let dawn = sun_color_from_elevation(0.02);
        // Reds saturate at 1.0 for both; green/blue prove the warm ramp.
        assert!(noon.g > dawn.g + 0.4 && noon.b > dawn.b + 0.5);
        assert!(dawn.r >= 0.99, "low sun keeps full red");
        let below = sun_color_from_elevation(-1.0);
        assert_eq!(below.g, 0.42, "below-horizon suns clamp to sunrise");
    }

    #[test]
    fn sun_light_maps_into_directional_lights() {
        let sun = SunLight::from_elevation(0.9, 1.4, 3.0).with_angular_radius(0.02);
        assert!(sun.intensity > 0.0);
        let directional = sun.to_directional_light();
        assert!((directional.direction.length() - 1.0).abs() < 1.0e-5);
        assert!((directional.intensity - 3.0).abs() < 1.0e-5);
        assert_eq!(directional.color, sun.color);

        let zero = SunLight::from_direction(Vec3::ZERO, 1.0);
        assert!((zero.direction.y + 1.0).abs() < 1.0e-5);
    }
}
