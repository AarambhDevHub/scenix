//! Time-sampled keyframe tracks for clip-based animation.
//!
//! These complement the procedural tween/spring tracks in [`crate::tracks`]:
//! procedural tracks run a fixed tween/spring once; keyframe tracks sample an
//! arbitrary clip-local time across an ordered keyframe array, matching glTF /
//! FBX import semantics. The mixer in [`crate::mixer`] consumes these tracks
//! through [`crate::clip::ClipTrack`].

use alloc::vec::Vec;
use core::f32;

use scenix_core::Color;
use scenix_math::{Quat, Vec3};

/// Keyframe interpolation mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KeyframeInterpolation {
    /// Linear interpolation between adjacent keyframes.
    Linear,
    /// Hold the previous keyframe value until the next time boundary.
    Step,
    /// Cubic Hermite spline interpolation (tangents packed as in/out slopes).
    CubicSpline,
}

/// Locates the bracketing keyframe pair for `time` and returns
/// `(left_index, alpha)` where `alpha` is the normalized position in
/// `[0, 1]` between `times[left]` and `times[left + 1]`.
///
/// Times are clamped to the first/last keyframe. For `Step` interpolation
/// callers use `left_index` directly; `alpha` is ignored.
fn bracket(times: &[f32], time: f32) -> (usize, f32) {
    if times.is_empty() {
        return (0, 0.0);
    }
    if time <= times[0] {
        return (0, 0.0);
    }
    if let Some(&last) = times.last()
        && time >= last
    {
        return (times.len() - 1, 0.0);
    }
    // Binary search for the bracketing pair.
    let mut lo = 0usize;
    let mut hi = times.len() - 1;
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if times[mid] <= time {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let span = (times[hi] - times[lo]).max(f32::EPSILON);
    let alpha = ((time - times[lo]) / span).clamp(0.0, 1.0);
    (lo, alpha)
}

/// Validate a packed-scalar keyframe track: non-empty, monotonic non-decreasing
/// times, and `values.len() == times.len() * per_key`.
fn validate_scalar(times: &[f32], values: &[f32], per_key: usize) -> bool {
    if times.is_empty() || values.len() != times.len() * per_key {
        return false;
    }
    let mut prev = f32::NEG_INFINITY;
    for &t in times {
        if !t.is_finite() || t < prev {
            return false;
        }
        prev = t;
    }
    true
}

/// Scalar keyframe track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyframeScalar {
    /// Keyframe times in seconds, monotonically non-decreasing.
    pub times: Vec<f32>,
    /// Packed values (`len == times.len()` for Linear/Step,
    /// `3 * times.len()` for CubicSpline: in-tangent, value, out-tangent).
    pub values: Vec<f32>,
    /// Interpolation mode.
    pub interpolation: KeyframeInterpolation,
}

impl KeyframeScalar {
    /// Creates a validated scalar keyframe track.
    pub fn new(times: Vec<f32>, values: Vec<f32>, interpolation: KeyframeInterpolation) -> Self {
        let per_key = if interpolation == KeyframeInterpolation::CubicSpline {
            3
        } else {
            1
        };
        assert!(
            validate_scalar(&times, &values, per_key),
            "invalid scalar keyframe track"
        );
        Self {
            times,
            values,
            interpolation,
        }
    }

    /// Clip duration (last keyframe time).
    #[inline]
    pub fn duration(&self) -> f32 {
        self.times.last().copied().unwrap_or(0.0)
    }

    /// Samples the track at `time`.
    pub fn sample(&self, time: f32) -> f32 {
        let (i, a) = bracket(&self.times, time);
        match self.interpolation {
            KeyframeInterpolation::Step => self.values[i],
            KeyframeInterpolation::Linear => {
                if i + 1 >= self.times.len() {
                    self.values[i]
                } else {
                    self.values[i] + (self.values[i + 1] - self.values[i]) * a
                }
            }
            KeyframeInterpolation::CubicSpline => {
                // Packed layout: [in_tangent, value, out_tangent] per key.
                if i + 1 >= self.times.len() {
                    self.values[i * 3 + 1]
                } else {
                    let t = a;
                    let p0 = self.values[i * 3 + 1];
                    let m0 = self.values[i * 3 + 2];
                    let p1 = self.values[(i + 1) * 3 + 1];
                    let m1 = self.values[(i + 1) * 3];
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                    let h10 = t3 - 2.0 * t2 + t;
                    let h01 = -2.0 * t3 + 3.0 * t2;
                    let h11 = t3 - t2;
                    h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1
                }
            }
        }
    }
}

/// 3D vector keyframe track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyframeVec3 {
    /// Keyframe times in seconds.
    pub times: Vec<f32>,
    /// One value per keyframe.
    pub values: Vec<Vec3>,
    /// Interpolation mode (CubicSpline falls back to linear lerp for v1.4).
    pub interpolation: KeyframeInterpolation,
}

impl KeyframeVec3 {
    /// Creates a validated vector keyframe track.
    pub fn new(times: Vec<f32>, values: Vec<Vec3>, interpolation: KeyframeInterpolation) -> Self {
        assert_eq!(times.len(), values.len(), "keyframe count mismatch");
        Self {
            times,
            values,
            interpolation,
        }
    }

    /// Clip duration (last keyframe time).
    #[inline]
    pub fn duration(&self) -> f32 {
        self.times.last().copied().unwrap_or(0.0)
    }

    /// Samples the track at `time`.
    pub fn sample(&self, time: f32) -> Vec3 {
        let (i, a) = bracket(&self.times, time);
        match self.interpolation {
            KeyframeInterpolation::Step => self.values[i],
            _ => {
                if i + 1 >= self.times.len() {
                    self.values[i]
                } else {
                    self.values[i].lerp(self.values[i + 1], a)
                }
            }
        }
    }
}

/// Quaternion keyframe track (uses slerp for Linear, nearest for Step).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyframeQuat {
    /// Keyframe times in seconds.
    pub times: Vec<f32>,
    /// One quaternion per keyframe.
    pub values: Vec<Quat>,
    /// Interpolation mode (Step holds; Linear/CubicSpline use slerp).
    pub interpolation: KeyframeInterpolation,
}

impl KeyframeQuat {
    /// Creates a validated quaternion keyframe track.
    pub fn new(times: Vec<f32>, values: Vec<Quat>, interpolation: KeyframeInterpolation) -> Self {
        assert_eq!(times.len(), values.len(), "keyframe count mismatch");
        Self {
            times,
            values,
            interpolation,
        }
    }

    /// Clip duration (last keyframe time).
    #[inline]
    pub fn duration(&self) -> f32 {
        self.times.last().copied().unwrap_or(0.0)
    }

    /// Samples the track at `time`, always taking the shortest arc.
    pub fn sample(&self, time: f32) -> Quat {
        let (i, a) = bracket(&self.times, time);
        match self.interpolation {
            KeyframeInterpolation::Step => self.values[i],
            _ => {
                if i + 1 >= self.times.len() {
                    self.values[i].normalize()
                } else {
                    let v0 = self.values[i];
                    let mut v1 = self.values[i + 1];
                    if v0.dot(v1) < 0.0 {
                        v1 = -v1;
                    }
                    v0.slerp(v1, a).normalize()
                }
            }
        }
    }
}

/// Color keyframe track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyframeColor {
    /// Keyframe times in seconds.
    pub times: Vec<f32>,
    /// One color per keyframe.
    pub values: Vec<Color>,
    /// Interpolation mode (CubicSpline falls back to linear lerp for v1.4).
    pub interpolation: KeyframeInterpolation,
}

impl KeyframeColor {
    /// Creates a validated color keyframe track.
    pub fn new(times: Vec<f32>, values: Vec<Color>, interpolation: KeyframeInterpolation) -> Self {
        assert_eq!(times.len(), values.len(), "keyframe count mismatch");
        Self {
            times,
            values,
            interpolation,
        }
    }

    /// Clip duration (last keyframe time).
    #[inline]
    pub fn duration(&self) -> f32 {
        self.times.last().copied().unwrap_or(0.0)
    }

    /// Samples the track at `time`.
    pub fn sample(&self, time: f32) -> Color {
        let (i, a) = bracket(&self.times, time);
        match self.interpolation {
            KeyframeInterpolation::Step => self.values[i],
            _ => {
                if i + 1 >= self.times.len() {
                    self.values[i]
                } else {
                    self.values[i].lerp(self.values[i + 1], a)
                }
            }
        }
    }
}

/// Boolean keyframe track (Step interpolation only).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyframeBool {
    /// Keyframe times in seconds.
    pub times: Vec<f32>,
    /// One boolean per keyframe.
    pub values: Vec<bool>,
}

impl KeyframeBool {
    /// Creates a validated boolean keyframe track.
    pub fn new(times: Vec<f32>, values: Vec<bool>) -> Self {
        assert_eq!(times.len(), values.len(), "keyframe count mismatch");
        Self { times, values }
    }

    /// Clip duration (last keyframe time).
    #[inline]
    pub fn duration(&self) -> f32 {
        self.times.last().copied().unwrap_or(0.0)
    }

    /// Samples the track at `time` (holds the most recent keyframe value).
    pub fn sample(&self, time: f32) -> bool {
        let (i, _) = bracket(&self.times, time);
        self.values[i]
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn scalar_linear_samples_midpoint() {
        let track = KeyframeScalar::new(
            vec![0.0, 1.0],
            vec![0.0, 10.0],
            KeyframeInterpolation::Linear,
        );
        assert_eq!(track.sample(0.0), 0.0);
        assert!((track.sample(0.5) - 5.0).abs() < 1e-4);
        assert_eq!(track.sample(1.0), 10.0);
    }

    #[test]
    fn step_holds_previous_value() {
        let track =
            KeyframeScalar::new(vec![0.0, 1.0], vec![0.0, 10.0], KeyframeInterpolation::Step);
        assert_eq!(track.sample(0.99), 0.0);
        assert_eq!(track.sample(1.0), 10.0);
    }

    #[test]
    fn quat_takes_shortest_arc() {
        let track = KeyframeQuat::new(
            vec![0.0, 1.0],
            vec![Quat::IDENTITY, Quat::from_axis_angle(Vec3::Y, 3.0)],
            KeyframeInterpolation::Linear,
        );
        let mid = track.sample(0.5);
        assert!((mid.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn bool_holds_until_boundary() {
        let track = KeyframeBool::new(vec![0.0, 0.5, 1.0], vec![false, true, false]);
        assert!(!track.sample(0.0));
        assert!(track.sample(0.7));
        assert!(!track.sample(1.0));
    }
}
