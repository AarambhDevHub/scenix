//! Keyframed value curves for particle properties.
//!
//! Two curve types cover the common particle authoring surface:
//! [`ScalarCurve`] for sizes and spin multipliers, and [`ColorCurve`] for
//! rgba ramps. Both evaluate `t` in `[0, 1]` with linear interpolation
//! between keys, clamp outside the first and last key, and keep the key
//! order strict ascending so evaluation stays branch-predictable.

use alloc::vec::Vec;

#[cfg(feature = "std")]
use scenix_core::Color;

/// A keyframe in a [`ScalarCurve`].
pub type ScalarKey = (f32, f32);

/// A keyframe in a [`ColorCurve`] (linear rgba).
pub type ColorKey = (f32, [f32; 4]);

/// Sorted keyframe list for one scalar channel.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScalarCurve {
    keys: Vec<ScalarKey>,
}

impl ScalarCurve {
    /// Creates a curve from keys; they do not need to come pre-sorted.
    #[inline]
    pub fn new(mut keys: Vec<ScalarKey>) -> Self {
        keys.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { keys }
    }

    /// Creates a curve that evaluates to `value` everywhere.
    #[inline]
    pub fn constant(value: f32) -> Self {
        Self {
            keys: alloc::vec![(0.0, value), (1.0, value)],
        }
    }

    /// Returns the sorted keys.
    #[inline]
    pub fn keys(&self) -> &[ScalarKey] {
        &self.keys
    }

    /// Returns whether the curve has no keys.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Evaluates at `t`; `t` is clamped to the key span, so values outside
    /// repeat the first/last keys. Empty curves return `1.0`.
    pub fn eval(&self, t: f32) -> f32 {
        let keys = self.keys.as_slice();
        let (Some(first), Some(last)) = (keys.first(), keys.last()) else {
            return 1.0;
        };
        let t = t.clamp(first.0, last.0);
        for window in keys.windows(2) {
            let (t0, v0) = (window[0].0, window[0].1);
            let (t1, v1) = (window[1].0, window[1].1);
            if t >= t0 && t <= t1 {
                let span = t1 - t0;
                if span <= f32::EPSILON {
                    return v1;
                }
                return v0 + (v1 - v0) * ((t - t0) / span);
            }
        }
        last.1
    }
}

/// Sorted keyframe list for one rgba channel (componentwise linear blend).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorCurve {
    keys: Vec<ColorKey>,
}

impl ColorCurve {
    /// Creates a curve from unsorted keys.
    #[inline]
    pub fn new(mut keys: Vec<ColorKey>) -> Self {
        keys.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { keys }
    }

    /// Creates a curve that fades `from` to `to` across `[0, 1]`.
    #[inline]
    pub fn from_to(from: [f32; 4], to: [f32; 4]) -> Self {
        Self {
            keys: alloc::vec![(0.0, from), (1.0, to)],
        }
    }

    /// Returns the sorted keys.
    #[inline]
    pub fn keys(&self) -> &[ColorKey] {
        &self.keys
    }

    /// Returns whether the curve has no keys.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Evaluates at `t`; clamped to the key span, componentwise linear.
    /// Empty curves return white.
    pub fn eval(&self, t: f32) -> [f32; 4] {
        let keys = self.keys.as_slice();
        let (Some(first), Some(last)) = (keys.first(), keys.last()) else {
            return [1.0, 1.0, 1.0, 1.0];
        };
        let t = t.clamp(first.0, last.0);
        for window in keys.windows(2) {
            let (t0, v0) = (window[0].0, window[0].1);
            let (t1, v1) = (window[1].0, window[1].1);
            if t >= t0 && t <= t1 {
                let span = t1 - t0;
                if span <= f32::EPSILON {
                    return v1;
                }
                let k = (t - t0) / span;
                let mut out = [0.0_f32; 4];
                for channel in 0..4 {
                    out[channel] = v0[channel] + (v1[channel] - v0[channel]) * k;
                }
                return out;
            }
        }
        last.1
    }
}

#[cfg(feature = "std")]
impl ColorCurve {
    /// Creates a curve from provided colors evaluated as linear rgba.
    #[inline]
    pub fn from_colors(keys: Vec<(f32, Color)>) -> Self {
        Self::new(
            keys.into_iter()
                .map(|(t, color)| (t, color.to_array()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_curve_sorts_keys_and_evaluates_linearly() {
        let curve = ScalarCurve::new(alloc::vec![(0.0, 0.5), (0.8, 1.0), (-1.0, 0.0)]);
        let keys = curve.keys();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0].0, -1.0);
        assert_eq!(keys[1].0, 0.0);
        assert_eq!(keys[2].0, 0.8);
        assert_eq!(curve.eval(-1.5), 0.0);
        assert_eq!(curve.eval(1.5), 1.0);
        assert!((curve.eval(0.4) - 0.75).abs() < 1e-6, "halfway 0..0.8");
    }

    #[test]
    fn curves_are_total_order_stable() {
        // NaN keys would poison a plain sort; total_cmp pins them
        // deterministically so evaluation never misbehaves.
        let curve = ScalarCurve::new(alloc::vec![(1.0, 1.0), (f32::NAN, 0.5), (0.5, 0.2)]);
        assert_eq!(curve.keys().len(), 3);
        assert!(curve.keys()[0].0 <= curve.keys()[1].0);
    }

    #[test]
    fn empty_curves_return_neutral_values() {
        assert_eq!(ScalarCurve::default().eval(0.5), 1.0);
        assert_eq!(ColorCurve::default().eval(0.5), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn color_curve_blends_componentwise() {
        let curve = ColorCurve::from_to([1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.0]);
        let mid = curve.eval(0.5);
        assert!((mid[0] - 0.5) < 1e-6 && (mid[1] - 0.5) < 1e-6);
        assert!((mid[3] - 0.5) < 1e-6);
        assert_eq!(curve.eval(-1.0), [1.0, 0.0, 0.0, 1.0]);
    }
}
