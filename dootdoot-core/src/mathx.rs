//! Owned deterministic math functions for the audio path.

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
