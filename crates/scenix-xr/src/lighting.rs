//! Estimated ambient lighting the runtime blends against real light.

use scenix_math::Vec3;

/// How detailed the runtime's light estimate is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum XrLightEstimationMode {
    /// No estimation is running.
    #[default]
    Off,
    /// Only a single ambient intensity/color scalar is published.
    AmbientOnly,
    /// Ambient plus a directional estimate (sun or brightest pane).
    AmbientWithDirection,
    /// Ambient plus spherical harmonic coefficients.
    SphericalHarmonics,
}

impl XrLightEstimationMode {
    /// Returns whether the runtime is publishing estimates right now.
    #[inline]
    #[must_use]
    pub const fn is_active(self) -> bool {
        !matches!(self, Self::Off)
    }

    /// Returns a short human-readable label used by inspector tooling.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::AmbientOnly => "ambient",
            Self::AmbientWithDirection => "ambient+direction",
            Self::SphericalHarmonics => "spherical-harmonics",
        }
    }
}

/// One estimated lighting frame published while a session is active.
///
/// Applications push `color`/`intensity` straight into `AmbientLight`
/// parameters and the optional `direction` into a sun light.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XrAmbientEstimate {
    /// Linear-space estimated ambient color.
    pub color: Vec3,
    /// Scene intensity in `0..=1`.
    pub intensity: f32,
    /// Optional dominant-light direction in tracking space, normalized.
    pub direction: Option<Vec3>,
    /// Detail mode that produced this estimate.
    pub mode: XrLightEstimationMode,
}

impl XrAmbientEstimate {
    /// Creates an ambient-only estimate from a color and intensity.
    #[inline]
    #[must_use]
    pub fn ambient(color: Vec3, intensity: f32) -> Self {
        Self {
            color: sanitize_color(color),
            intensity: intensity.clamp(0.0, 1.0),
            direction: None,
            mode: XrLightEstimationMode::AmbientOnly,
        }
    }

    /// Creates a light estimate with a dominant direction.
    #[must_use]
    pub fn with_direction(mut self, direction: Vec3) -> Self {
        if direction.length_squared() <= 1.0e-12 {
            return self;
        }
        self.direction = Some(direction.normalize());
        if self.mode == XrLightEstimationMode::AmbientOnly
            || self.mode == XrLightEstimationMode::Off
        {
            self.mode = XrLightEstimationMode::AmbientWithDirection;
        }
        self
    }

    /// Attenuated color renderers want: `color * intensity`.
    #[inline]
    #[must_use]
    pub fn linear_color(&self) -> Vec3 {
        self.color * self.intensity
    }

    /// Luminance proxy used for HUD contrast decisions.
    #[inline]
    #[must_use]
    pub fn luminance(&self) -> f32 {
        let rgb = self.color;
        0.2126 * rgb.x + 0.7152 * rgb.y + 0.0722 * rgb.z
    }

    /// Eases `self` toward `toward`'s estimate by `t`; keeps the richer
    /// detail mode and only swaps direction when fully converged.
    #[must_use]
    pub fn lerp(self, toward: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            color: self.color.lerp(sanitize_color(toward.color), t),
            intensity: self.intensity + (toward.intensity - self.intensity) * t,
            direction: if t >= 1.0 {
                toward.direction
            } else {
                self.direction
            },
            mode: if self.mode >= toward.mode {
                self.mode
            } else {
                toward.mode
            },
        }
    }
}

impl Default for XrAmbientEstimate {
    #[inline]
    fn default() -> Self {
        Self::ambient(Vec3::ONE, 1.0)
    }
}

/// Public alias so applications accept either estimate type.
pub type XrLightEstimate = XrAmbientEstimate;

/// Clamps negative or non-finite color channels into safe linear range.
#[inline]
#[must_use]
fn sanitize_color(color: Vec3) -> Vec3 {
    Vec3::new(
        sanitize_channel(color.x),
        sanitize_channel(color.y),
        sanitize_channel(color.z),
    )
}

/// Per-channel sanitizer: `NaN`/`Inf` collapse to white, negatives clamp.
#[inline]
#[must_use]
fn sanitize_channel(channel: f32) -> f32 {
    if !channel.is_finite() {
        return 1.0;
    }
    if channel < 0.0 { 0.0 } else { channel.min(8.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambient_estimate_basics() {
        let estimate = XrAmbientEstimate::ambient(Vec3::new(0.5, 0.5, 0.5), 0.8);
        assert!(estimate.linear_color().x < 0.5 + 1e-6);
        assert_eq!(estimate.mode, XrLightEstimationMode::AmbientOnly);
        assert!((estimate.luminance() - 0.5).abs() < 1e-5);
        assert!(estimate.mode.is_active());
    }

    #[test]
    fn directions_upgrade_mode() {
        let estimate =
            XrAmbientEstimate::ambient(Vec3::ONE, 0.5).with_direction(Vec3::new(0.0, -2.0, 0.0));
        assert_eq!(estimate.mode, XrLightEstimationMode::AmbientWithDirection);
        let d = estimate.direction.expect("direction stored");
        assert!((d.length() - 1.0).abs() < 1e-5);

        let degenerate = XrAmbientEstimate::ambient(Vec3::ONE, 0.5);
        assert!(degenerate.with_direction(Vec3::ZERO).direction.is_none());
    }

    #[test]
    fn sanitize_handles_bad_colors() {
        let estimate = XrAmbientEstimate::ambient(Vec3::new(f32::NAN, -1.0, 12.0), 2.0);
        assert_eq!(estimate.color.x, 1.0);
        assert_eq!(estimate.color.y, 0.0);
        assert_eq!(estimate.color.z, 8.0);
        assert!((estimate.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn lerp_eases_between_estimates() {
        let a = XrAmbientEstimate::ambient(Vec3::ZERO, 0.2);
        let b = XrAmbientEstimate::ambient(Vec3::ONE, 0.8);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.intensity - 0.5).abs() < 1e-6);
        assert!((mid.color.x - 0.5).abs() < 1e-6);
        // Mode always tracks the richer estimate.
        let rich = b.with_direction(Vec3::X);
        let poor = a.lerp(&rich, 0.4);
        assert_eq!(poor.mode, XrLightEstimationMode::AmbientWithDirection);
    }

    #[test]
    fn off_mode_is_inactive() {
        assert!(!XrLightEstimationMode::Off.is_active());
        assert_eq!(XrLightEstimationMode::Off.label(), "off");
    }
}
