//! This mod re-exports the correct versions of floating-point operations with
//! unspecified precision in the standard library depending on whether the `libm`
//! crate feature is enabled.
//!
//! All the functions here are named according to their versions in the standard
//! library.
//!
//! It also provides `no_std` compatible alternatives to certain floating-point
//! operations which are not provided in the [`core`] library.

#![allow(
    clippy::disallowed_methods,
    reason = "Many of the disallowed methods are disallowed to force code to use the feature-conditional re-exports from this module, but this module itself is exempt from that rule."
)]

// Note: There are some Rust methods with unspecified precision without a `libm`
// equivalent:
// - `f32::powi` (integer powers)
// - `f32::log` (logarithm with specified base)
// - `f32::abs_sub` (actually unsure if `libm` has this, but don't use it regardless)
//
// Additionally, the following nightly API functions are not presently integrated
// into this, but they would be candidates once standardized:
// - `f32::gamma`
// - `f32::ln_gamma`

use crate::cfg::{self, switch};

/// Raises a number to a floating point power.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn powf(x: f32, y: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::powf(x, y) }
        cfg::std => { f32::powf(x, y) }
        _ => { libm::powf(x, y) }
    }}
}

/// Returns `e^(self)`, (the exponential function).
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn exp(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::expf(x) }
        cfg::std => { f32::exp(x) }
        _ => { libm::expf(x) }
    }}
}

/// Returns `2^(self)`.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn exp2(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::exp2f(x) }
        cfg::std => { f32::exp2(x) }
        _ => { libm::exp2f(x) }
    }}
}

/// Returns the natural logarithm of the number.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn ln(x: f32) -> f32 {
    // This isn't documented in `libm` but this is actually the base e logarithm.
    switch! {{
        cfg::libm => { libm::logf(x) }
        cfg::std => { f32::ln(x) }
        _ => { libm::logf(x) }
    }}
}

/// Returns the base 2 logarithm of the number.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn log2(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::log2f(x) }
        cfg::std => { f32::ln(x) }
        _ => { libm::log2f(x) }
    }}
}

/// Returns the base 10 logarithm of the number.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn log10(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::log10f(x) }
        cfg::std => { f32::log10(x) }
        _ => { libm::log10f(x) }
    }}
}

/// Returns the cube root of a number.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn cbrt(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::cbrtf(x) }
        cfg::std => { f32::cbrt(x) }
        _ => { libm::cbrtf(x) }
    }}
}

/// Compute the distance between the origin and a point `(x, y)` on the Euclidean plane.
/// Equivalently, compute the length of the hypotenuse of a right-angle triangle with other sides having length `x.abs()` and `y.abs()`.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn hypot(x: f32, y: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::hypotf(x, y) }
        cfg::std => { f32::hypot(x, y) }
        _ => { libm::hypotf(x, y) }
    }}
}

/// Computes the sine of a number (in radians).
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn sin(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::sinf(x) }
        cfg::std => { f32::sin(x) }
        _ => { libm::sinf(x) }
    }}
}

/// Computes the cosine of a number (in radians).
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn cos(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::cosf(x) }
        cfg::std => { f32::cos(x) }
        _ => { libm::cosf(x) }
    }}
}

/// Computes the tangent of a number (in radians).
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn tan(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::tanf(x) }
        cfg::std => { f32::tan(x) }
        _ => { libm::tanf(x) }
    }}
}

/// Computes the arcsine of a number. Return value is in radians in
/// the range [-pi/2, pi/2] or NaN if the number is outside the range
/// [-1, 1].
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn asin(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::asinf(x) }
        cfg::std => { f32::asin(x) }
        _ => { libm::asinf(x) }
    }}
}

/// Computes the arccosine of a number. Return value is in radians in
/// the range [0, pi] or NaN if the number is outside the range
/// [-1, 1].
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn acos(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::acosf(x) }
        cfg::std => { f32::acos(x) }
        _ => { libm::acosf(x) }
    }}
}

/// Computes the arctangent of a number. Return value is in radians in the
/// range [-pi/2, pi/2];
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn atan(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::atanf(x) }
        cfg::std => { f32::atan(x) }
        _ => { libm::atanf(x) }
    }}
}

/// Computes the four-quadrant arctangent of `y` and `x` in radians.
///
/// * `x = 0`, `y = 0`: `0`
/// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
/// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
/// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn atan2(y: f32, x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::atan2f(y, x) }
        cfg::std => { f32::atan2(y, x) }
        _ => { libm::atan2f(y, x) }
    }}
}

/// Simultaneously computes the sine and cosine of the number, `x`. Returns
/// `(sin(x), cos(x))`.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn sin_cos(x: f32) -> (f32, f32) {
    switch! {{
        cfg::libm => { libm::sincosf(x) }
        cfg::std => { f32::sin_cos(x) }
        _ => { libm::sincosf(x) }
    }}
}

/// Returns `e^(self) - 1` in a way that is accurate even if the
/// number is close to zero.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn exp_m1(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::expm1f(x) }
        cfg::std => { f32::exp_m1(x) }
        _ => { libm::expm1f(x) }
    }}
}

/// Returns `ln(1+n)` (natural logarithm) more accurately than if
/// the operations were performed separately.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn ln_1p(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::log1pf(x) }
        cfg::std => { f32::ln_1p(x) }
        _ => { libm::log1pf(x) }
    }}
}

/// Hyperbolic sine function.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn sinh(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::sinhf(x) }
        cfg::std => { f32::sinh(x) }
        _ => { libm::sinhf(x) }
    }}
}

/// Hyperbolic cosine function.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn cosh(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::coshf(x) }
        cfg::std => { f32::cosh(x) }
        _ => { libm::coshf(x) }
    }}
}

/// Hyperbolic tangent function.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn tanh(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::tanhf(x) }
        cfg::std => { f32::tanh(x) }
        _ => { libm::tanhf(x) }
    }}
}

/// Inverse hyperbolic sine function.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn asinh(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::asinhf(x) }
        cfg::std => { f32::asinh(x) }
        _ => { libm::asinhf(x) }
    }}
}

/// Inverse hyperbolic cosine function.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn acosh(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::acoshf(x) }
        cfg::std => { f32::acosh(x) }
        _ => { libm::acoshf(x) }
    }}
}

/// Inverse hyperbolic tangent function.
///
/// Precision is specified when the `libm` feature is enabled.
#[inline(always)]
pub fn atanh(x: f32) -> f32 {
    switch! {{
        cfg::libm => { libm::atanhf(x) }
        cfg::std => { f32::atanh(x) }
        _ => { libm::atanhf(x) }
    }}
}

/// Calculates the least nonnegative remainder of `x (mod y)`.
///
/// The result of this operation is guaranteed to be the rounded infinite-precision result.
#[inline(always)]
pub fn rem_euclid(x: f32, y: f32) -> f32 {
    switch! {{
        cfg::std => { f32::rem_euclid(x, y) }
        _ => {
            let result = libm::remainderf(x, y);

            // libm::remainderf has a range of -y/2 to +y/2
            if result < 0. {
                result + y
            } else {
                result
            }
        }
    }}
}

/// Computes the absolute value of x.
///
/// This function always returns the precise result.
#[inline(always)]
pub fn abs(x: f32) -> f32 {
    switch! {{
        cfg::std => { f32::abs(x) }
        _ => { libm::fabsf(x) }
    }}
}

/// Returns the square root of a number.
///
/// The result of this operation is guaranteed to be the rounded infinite-precision result.
/// It is specified by IEEE 754 as `squareRoot` and guaranteed not to change.
#[inline(always)]
pub fn sqrt(x: f32) -> f32 {
    switch! {{
        cfg::std => { f32::sqrt(x) }
        _ => { libm::sqrtf(x) }
    }}
}

/// Returns a number composed of the magnitude of `x` and the sign of `y`.
///
/// Equal to `x` if the sign of `x` and `y` are the same, otherwise equal to `-x`. If `x` is a
/// `NaN`, then a `NaN` with the sign bit of `y` is returned. Note, however, that conserving the
/// sign bit on `NaN` across arithmetical operations is not generally guaranteed.
#[inline(always)]
pub fn copysign(x: f32, y: f32) -> f32 {
    switch! {{
        cfg::std => { f32::copysign(x, y) }
        _ => { libm::copysignf(x, y) }
    }}
}

/// Returns the nearest integer to `x`. If a value is half-way between two integers, round away from `0.0`.
///
/// This function always returns the precise result.
#[inline(always)]
pub fn round(x: f32) -> f32 {
    switch! {{
        cfg::std => { f32::round(x) }
        _ => { libm::roundf(x) }
    }}
}

/// Returns the largest integer less than or equal to `x`.
///
/// This function always returns the precise result.
#[inline(always)]
pub fn floor(x: f32) -> f32 {
    switch! {{
        cfg::std => { f32::floor(x) }
        _ => { libm::floorf(x) }
    }}
}

/// Returns the smallest integer greater than or equal to `x`.
///
/// This function always returns the precise result.
#[inline(always)]
pub fn ceil(x: f32) -> f32 {
    switch! {{
        cfg::std => { f32::ceil(x) }
        _ => { libm::ceilf(x) }
    }}
}

/// Returns the fractional part of `x`.
///
/// This function always returns the precise result.
#[inline(always)]
pub fn fract(x: f32) -> f32 {
    switch! {{
        cfg::std => { f32::fract(x) }
        _ => { libm::modff(x).0 }
    }}
}

/// This extension trait covers shortfall in determinacy from the lack of a `libm` counterpart
/// to `f32::powi`. Use this for the common small exponents.
pub trait FloatPow {
    /// Squares the f32
    fn squared(self) -> Self;
    /// Cubes the f32
    fn cubed(self) -> Self;
}

impl FloatPow for f32 {
    #[inline]
    fn squared(self) -> Self {
        self * self
    }
    #[inline]
    fn cubed(self) -> Self {
        self * self * self
    }
}
