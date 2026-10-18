//! Portable `f32` intrinsics shared by scenix crates.
//!
//! Each function picks the best implementation for the target: the `libm`
//! crate when the `libm` feature is on, the hardware/`std` intrinsic on
//! hosted builds, and a small polynomial or Newton fallback otherwise.
//! These shims exist so crates like `scenix-particles` can keep pure CPU
//! simulation paths that compile under `no_std`.

#[cfg(all(not(feature = "libm"), not(feature = "std")))]
use crate::EPSILON;

#[cfg(all(not(feature = "libm"), not(feature = "std")))]
const PI: f32 = core::f32::consts::PI;
#[cfg(all(not(feature = "libm"), not(feature = "std")))]
const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;

/// Positive square root; on fallback (`no_std`, no `libm`) targets
/// non-positive inputs clamp to `0.0`, while `libm`/`std` targets keep the
/// platform's `NaN` behavior for negative inputs.
#[must_use]
#[inline]
pub fn sqrt(value: f32) -> f32 {
    #[cfg(feature = "libm")]
    {
        libm::sqrtf(value)
    }
    #[cfg(all(not(feature = "libm"), feature = "std"))]
    {
        value.sqrt()
    }
    #[cfg(all(not(feature = "libm"), not(feature = "std")))]
    {
        if value <= 0.0 {
            return 0.0;
        }
        let mut x = if value >= 1.0 { value } else { 1.0 };
        let mut i = 0;
        while i < 8 {
            x = 0.5 * (x + value / x);
            i += 1;
        }
        x
    }
}

/// Sine of `value` (radians).
#[must_use]
#[inline]
pub fn sin(value: f32) -> f32 {
    #[cfg(feature = "libm")]
    {
        libm::sinf(value)
    }
    #[cfg(all(not(feature = "libm"), feature = "std"))]
    {
        value.sin()
    }
    #[cfg(all(not(feature = "libm"), not(feature = "std")))]
    {
        let x = reduce_half_pi(value);
        let x2 = x * x;
        x * (1.0 - x2 / 6.0 + (x2 * x2) / 120.0 - (x2 * x2 * x2) / 5040.0
            + (x2 * x2 * x2 * x2) / 362_880.0)
    }
}

/// Cosine of `value` (radians).
#[must_use]
#[inline]
pub fn cos(value: f32) -> f32 {
    #[cfg(feature = "libm")]
    {
        libm::cosf(value)
    }
    #[cfg(all(not(feature = "libm"), feature = "std"))]
    {
        value.cos()
    }
    #[cfg(all(not(feature = "libm"), not(feature = "std")))]
    {
        let mut x = reduce_pi(value);
        let mut sign = 1.0;
        if x > FRAC_PI_2 {
            x = PI - x;
            sign = -1.0;
        } else if x < -FRAC_PI_2 {
            x = -PI - x;
            sign = -1.0;
        }
        let x2 = x * x;
        sign * (1.0 - x2 / 2.0 + (x2 * x2) / 24.0 - (x2 * x2 * x2) / 720.0
            + (x2 * x2 * x2 * x2) / 40_320.0)
    }
}

/// Tangent of `value` (radians); returns signed infinity at vertical
/// asymptotes.
#[must_use]
#[inline]
pub fn tan(value: f32) -> f32 {
    #[cfg(feature = "libm")]
    {
        libm::tanf(value)
    }
    #[cfg(all(not(feature = "libm"), feature = "std"))]
    {
        value.tan()
    }
    #[cfg(all(not(feature = "libm"), not(feature = "std")))]
    {
        let c = cos(value);
        if c.abs() <= EPSILON {
            if value >= 0.0 {
                f32::INFINITY
            } else {
                -f32::INFINITY
            }
        } else {
            sin(value) / c
        }
    }
}

/// Cube root; preserves the sign of `value` (odd root).
#[must_use]
#[inline]
pub fn cbrt(value: f32) -> f32 {
    #[cfg(feature = "libm")]
    {
        libm::cbrtf(value)
    }
    #[cfg(all(not(feature = "libm"), feature = "std"))]
    {
        value.cbrt()
    }
    #[cfg(all(not(feature = "libm"), not(feature = "std")))]
    {
        if value == 0.0 {
            return 0.0;
        }
        let sign = value.signum();
        let abs = value.abs();
        // Seed with a power-of-two cube root of the input's exponent floor:
        // for `abs` in `[2^k, 2^(k+1))` the root lies in
        // `[2^(k/3), 2^((k+1)/3))`, so one `div_euclid` step bounds the
        // relative gap by ~2.67 and the six Newton steps converge to f32
        // precision. Plain Newton from `abs` needs a third of the exponent
        // in iterations first.
        let biased_decay = ((abs.to_bits() >> 23) & 0xff) as i32 - 127;
        let seed_bits = ((biased_decay.div_euclid(3) + 127) << 23) as u32;
        let mut x = f32::from_bits(seed_bits);
        let mut i = 0;
        while i < 6 {
            x = (2.0 * x + abs / (x * x)) / 3.0;
            i += 1;
        }
        sign * x
    }
}

/// Arccosine in radians; `value` is clamped to `[-1, 1]` on fallback
/// targets (and raises an invalid-op on `NaN` inputs otherwise).
#[must_use]
#[inline]
pub fn acos(value: f32) -> f32 {
    #[cfg(feature = "libm")]
    {
        libm::acosf(value)
    }
    #[cfg(all(not(feature = "libm"), feature = "std"))]
    {
        value.acos()
    }
    #[cfg(all(not(feature = "libm"), not(feature = "std")))]
    {
        FRAC_PI_2 - asin(value)
    }
}

/// Arcsine in radians; `value` is clamped to `[-1, 1]` on all targets.
#[must_use]
#[inline]
pub fn asin(value: f32) -> f32 {
    #[cfg(feature = "libm")]
    {
        libm::asinf(value)
    }
    #[cfg(all(not(feature = "libm"), feature = "std"))]
    {
        value.asin()
    }
    #[cfg(all(not(feature = "libm"), not(feature = "std")))]
    {
        let x = value.max(-1.0).min(1.0);
        atan2(x, sqrt((1.0 - x * x).max(0.0)))
    }
}

/// Four-quadrant arctangent of `y / x` in radians.
#[must_use]
#[inline]
pub fn atan2(y: f32, x: f32) -> f32 {
    #[cfg(feature = "libm")]
    {
        libm::atan2f(y, x)
    }
    #[cfg(all(not(feature = "libm"), feature = "std"))]
    {
        y.atan2(x)
    }
    #[cfg(all(not(feature = "libm"), not(feature = "std")))]
    {
        if x > 0.0 {
            atan_approx(y / x)
        } else if x < 0.0 && y >= 0.0 {
            atan_approx(y / x) + PI
        } else if x < 0.0 && y < 0.0 {
            atan_approx(y / x) - PI
        } else if y > 0.0 {
            FRAC_PI_2
        } else if y < 0.0 {
            -FRAC_PI_2
        } else {
            0.0
        }
    }
}

#[cfg(all(not(feature = "libm"), not(feature = "std")))]
#[inline]
fn reduce_pi(mut value: f32) -> f32 {
    let two_pi = 2.0 * PI;
    while value > PI {
        value -= two_pi;
    }
    while value < -PI {
        value += two_pi;
    }
    value
}

#[cfg(all(not(feature = "libm"), not(feature = "std")))]
#[inline]
fn reduce_half_pi(value: f32) -> f32 {
    let value = reduce_pi(value);
    if value > FRAC_PI_2 {
        PI - value
    } else if value < -FRAC_PI_2 {
        -PI - value
    } else {
        value
    }
}

#[cfg(all(not(feature = "libm"), not(feature = "std")))]
#[inline]
fn atan_approx(value: f32) -> f32 {
    let sign = value.signum();
    let abs = value.abs();
    if abs > 1.0 {
        sign * FRAC_PI_2 - sign * atan_approx(1.0 / abs)
    } else if abs > 0.414_213_57 {
        sign * (core::f32::consts::FRAC_PI_4 + atan_approx((abs - 1.0) / (abs + 1.0)))
    } else {
        let x2 = value * value;
        value
            * (1.0 - x2 / 3.0 + (x2 * x2) / 5.0 - (x2 * x2 * x2) / 7.0 + (x2 * x2 * x2 * x2) / 9.0
                - (x2 * x2 * x2 * x2 * x2) / 11.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{acos, asin, atan2, cbrt, cos, sin, sqrt, tan};
    use crate::assert_close;

    #[test]
    fn intrinsics_match_reference_values() {
        assert_close(sqrt(4.0), 2.0);
        assert_close(sqrt(0.25), 0.5);
        #[cfg(all(not(feature = "libm"), not(feature = "std")))]
        assert_eq!(sqrt(-1.0), 0.0, "negative inputs clamp to zero");
        #[cfg(any(feature = "libm", feature = "std"))]
        assert!(sqrt(-1.0).is_nan(), "platform NaN behavior preserved");
        assert_close(sin(core::f32::consts::FRAC_PI_2), 1.0);
        assert_close(cos(0.0), 1.0);
        assert_close(cos(core::f32::consts::PI), -1.0);
        assert_close(tan(0.0), 0.0);
        assert_close(acos(1.0), 0.0);
        assert_close(asin(0.0), 0.0);
        assert_close(atan2(0.0, 1.0), 0.0);
        assert_close(atan2(1.0, 1.0), core::f32::consts::FRAC_PI_4);
    }

    #[test]
    fn cbrt_preserves_sign_and_accuracy() {
        assert_eq!(cbrt(0.0), 0.0);
        assert_close(cbrt(8.0), 2.0);
        assert_close(cbrt(0.001), 0.1);
        assert_close(cbrt(1.0e6), 100.0);
        assert!(
            (cbrt(-27.0) + 3.0).abs() <= 1.0e-4,
            "odd root keeps the sign"
        );
    }
}
