//! Input-size limit checks for the command-line shell.

use std::{error::Error, fmt};

use dootdoot_core::SYNTH_SAMPLE_RATE_HZ;

const BYTES_PER_SAMPLE: u64 = 2;
const _: () = assert!(SYNTH_SAMPLE_RATE_HZ == 44_100);

/// Gives the warning threshold in rendered samples.
pub const WARNING_LIMIT_SAMPLES: u64 = 44_100 * 8 * 60;

/// Gives the hard ceiling in rendered samples.
pub const HARD_LIMIT_SAMPLES: u64 = 44_100 * 30 * 60;

/// Gives the warning threshold in rendered bytes.
pub const WARNING_LIMIT_BYTES: u64 = WARNING_LIMIT_SAMPLES * BYTES_PER_SAMPLE;

/// Gives the hard ceiling in rendered bytes.
pub const HARD_LIMIT_BYTES: u64 = HARD_LIMIT_SAMPLES * BYTES_PER_SAMPLE;

/// Gives the input-limit status for an estimated rendered sample count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputLimitStatus {
    /// The estimated output stays within the warning threshold.
    Accept,
    /// The estimated output passes the warning threshold but stays under the
    /// hard ceiling.
    Warn {
        /// Estimated rendered samples.
        sample_count: u64,
        /// Estimated rendered bytes for 16-bit mono PCM.
        byte_count: u64,
    },
    /// The estimated output exceeds the hard ceiling.
    Reject {
        /// Estimated rendered samples.
        sample_count: u64,
        /// Estimated rendered bytes for 16-bit mono PCM.
        byte_count: u64,
    },
}

/// Gives a user-visible warning for large but accepted input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputLimitWarning {
    sample_count: u64,
    byte_count: u64,
}

/// Reports that input is too large to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputLimitError {
    sample_count: u64,
    byte_count: u64,
}

/// Classifies an estimated rendered sample count against the fixed CLI limits.
pub fn check_input_limits(sample_count: u64) -> InputLimitStatus {
    let byte_count = sample_count.saturating_mul(BYTES_PER_SAMPLE);

    if sample_count > HARD_LIMIT_SAMPLES || byte_count > HARD_LIMIT_BYTES {
        return InputLimitStatus::Reject {
            sample_count,
            byte_count,
        };
    }

    if sample_count > WARNING_LIMIT_SAMPLES || byte_count > WARNING_LIMIT_BYTES {
        return InputLimitStatus::Warn {
            sample_count,
            byte_count,
        };
    }

    InputLimitStatus::Accept
}

/// Enforces the fixed CLI input limits.
///
/// # Errors
///
/// Returns an error when the estimate exceeds the hard output ceiling.
pub fn enforce_input_limits(
    sample_count: u64,
) -> Result<Option<InputLimitWarning>, InputLimitError> {
    match check_input_limits(sample_count) {
        InputLimitStatus::Accept => Ok(None),
        InputLimitStatus::Warn {
            sample_count,
            byte_count,
        } => Ok(Some(InputLimitWarning {
            sample_count,
            byte_count,
        })),
        InputLimitStatus::Reject {
            sample_count,
            byte_count,
        } => Err(InputLimitError {
            sample_count,
            byte_count,
        }),
    }
}

impl fmt::Display for InputLimitWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "warning: input would render to more than 8 minutes of audio ({} samples, {} bytes)",
            self.sample_count, self.byte_count,
        )
    }
}

impl fmt::Display for InputLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "input would render to {} samples ({} bytes), which exceeds the 30 minute / {} byte ceiling",
            self.sample_count, self.byte_count, HARD_LIMIT_BYTES,
        )
    }
}

impl Error for InputLimitError {}
