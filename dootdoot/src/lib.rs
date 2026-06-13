//! Testable command-line argument model for dootdoot.

mod explain;
mod input;
mod limits;
mod output;
mod playback;

use std::path::PathBuf;

use clap::Parser;
pub use explain::{explain_table_for_empty_chirp, explain_table_for_text};
pub use input::{ResolvedInput, StdinInput, read_resolved_input, resolve_input};
pub use limits::{
    HARD_LIMIT_BYTES, HARD_LIMIT_SAMPLES, InputLimitError, InputLimitStatus, InputLimitWarning,
    WARNING_LIMIT_BYTES, WARNING_LIMIT_SAMPLES, check_input_limits, enforce_input_limits,
};
pub use output::{OutputRoute, output_route};
pub use playback::{PlaybackError, play_buffer, playback_samples};

/// Parsed command-line arguments for the dootdoot binary.
#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(
    name = "dootdoot",
    version = dootdoot_core::FORMAT_V1,
    about = "Deterministically turns text into BB-8-style droid sound."
)]
pub struct Cli {
    /// Text to synthesize.
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,

    /// WAV output path.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Play audio even when writing an output file.
    #[arg(long = "play")]
    pub play: bool,

    /// Print the semantic token table to stderr.
    #[arg(long = "explain")]
    pub explain: bool,
}
