//! Owned deterministic math functions for the audio path.

use core::f64::consts::{FRAC_PI_2, PI, TAU};

/// Identifies the pinned owned-math implementation contract.
pub const MATHX_VERSION: &str = "mathx-v1";

/// Gives the base-two size exponent for the sine/cosine range-reduction table.
pub const SIN_COS_TABLE_BITS: u32 = 12;

/// Gives the number of sine/cosine table intervals in one full turn.
pub const SIN_COS_TABLE_LEN: usize = 4096;

/// Gives the correction polynomial degree used by sine and cosine.
pub const SIN_COS_POLYNOMIAL_DEGREE: u8 = 7;

/// Gives the base-two size exponent for the exponential range-reduction table.
pub const EXP_TABLE_BITS: u32 = 10;

/// Gives the number of exponential table intervals in one `ln(2)` span.
pub const EXP_TABLE_LEN: usize = 1024;

/// Gives the residual polynomial degree used by exponential.
pub const EXP_POLYNOMIAL_DEGREE: u8 = 5;

/// Gives the absolute input at which hyperbolic tangent saturates to +/-1.
pub const TANH_EXP_CLAMP: f64 = 20.0;

/// Marks the owned math module in the public facade.
#[derive(Debug)]
pub struct Mathx;

/// Computes sine with dootdoot's pinned owned-math path.
///
/// # Examples
///
/// ```
/// assert_eq!(dootdoot_core::sin(0.0).to_bits(), 0.0_f64.to_bits());
/// ```
#[must_use]
pub fn sin(radians: f64) -> f64 {
    if !radians.is_finite() {
        return f64::NAN;
    }

    let reduced = reduce_to_half_turn(radians);

    if reduced > FRAC_PI_2 {
        sin_kernel(PI - reduced)
    } else if reduced < -FRAC_PI_2 {
        -sin_kernel(PI + reduced)
    } else {
        sin_kernel(reduced)
    }
}

/// Computes cosine with dootdoot's pinned owned-math path.
///
/// # Examples
///
/// ```
/// assert_eq!(dootdoot_core::cos(0.0).to_bits(), 1.0_f64.to_bits());
/// ```
#[must_use]
pub fn cos(radians: f64) -> f64 {
    if !radians.is_finite() {
        return f64::NAN;
    }

    let reduced = reduce_to_half_turn(radians);

    if reduced > FRAC_PI_2 {
        -cos_kernel(PI - reduced)
    } else if reduced < -FRAC_PI_2 {
        -cos_kernel(PI + reduced)
    } else {
        cos_kernel(reduced)
    }
}

/// Computes exponential with dootdoot's pinned owned-math path.
///
/// # Examples
///
/// ```
/// assert_eq!(dootdoot_core::exp(0.0).to_bits(), 1.0_f64.to_bits());
/// ```
#[must_use]
pub fn exp(exponent: f64) -> f64 {
    if exponent.is_nan() {
        return f64::NAN;
    }

    if exponent > EXP_OVERFLOW_LIMIT {
        return f64::INFINITY;
    }

    if exponent < EXP_UNDERFLOW_LIMIT {
        return 0.0;
    }

    let magnitude = exponent.abs();
    let scaled = magnitude / EXP_SCALING_FACTOR;
    let mut result = exp_kernel(scaled);

    for _ in 0..EXP_SQUARING_STEPS {
        result *= result;
    }

    if exponent.is_sign_negative() {
        1.0 / result
    } else {
        result
    }
}

/// Computes hyperbolic tangent with dootdoot's pinned owned-math path.
///
/// # Examples
///
/// ```
/// assert_eq!(dootdoot_core::tanh(0.0).to_bits(), 0.0_f64.to_bits());
/// ```
#[must_use]
pub fn tanh(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }

    if value >= TANH_EXP_CLAMP {
        return 1.0;
    }

    if value <= -TANH_EXP_CLAMP {
        return -1.0;
    }

    let doubled = value + value;
    let exponential = exp(doubled);

    (exponential - 1.0) / (exponential + 1.0)
}

fn reduce_to_half_turn(radians: f64) -> f64 {
    radians - (radians / TAU).round() * TAU
}

fn sin_kernel(radians: f64) -> f64 {
    let squared = radians * radians;

    radians
        * (((SIN_COEFFICIENT_7 * squared + SIN_COEFFICIENT_5) * squared + SIN_COEFFICIENT_3)
            * squared
            + 1.0)
}

fn cos_kernel(radians: f64) -> f64 {
    let squared = radians * radians;

    ((COS_COEFFICIENT_6 * squared + COS_COEFFICIENT_4) * squared + COS_COEFFICIENT_2) * squared
        + 1.0
}

fn exp_kernel(exponent: f64) -> f64 {
    (((((EXP_COEFFICIENT_5 * exponent + EXP_COEFFICIENT_4) * exponent + EXP_COEFFICIENT_3)
        * exponent
        + EXP_COEFFICIENT_2)
        * exponent
        + 1.0)
        * exponent)
        + 1.0
}

const SIN_COEFFICIENT_3: f64 = -1.0 / 6.0;
const SIN_COEFFICIENT_5: f64 = 1.0 / 120.0;
const SIN_COEFFICIENT_7: f64 = -1.0 / 5040.0;

const COS_COEFFICIENT_2: f64 = -1.0 / 2.0;
const COS_COEFFICIENT_4: f64 = 1.0 / 24.0;
const COS_COEFFICIENT_6: f64 = -1.0 / 720.0;

const EXP_COEFFICIENT_2: f64 = 1.0 / 2.0;
const EXP_COEFFICIENT_3: f64 = 1.0 / 6.0;
const EXP_COEFFICIENT_4: f64 = 1.0 / 24.0;
const EXP_COEFFICIENT_5: f64 = 1.0 / 120.0;
const EXP_OVERFLOW_LIMIT: f64 = 709.0;
const EXP_SCALING_FACTOR: f64 = 32.0;
const EXP_SQUARING_STEPS: usize = 5;
const EXP_UNDERFLOW_LIMIT: f64 = -745.0;
