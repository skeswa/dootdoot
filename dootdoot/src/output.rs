//! Output routing for file writing and playback.

use std::path::PathBuf;

use crate::Cli;

/// Gives the resolved output route for one invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputRoute {
    /// WAV file path to write, when requested.
    pub output: Option<PathBuf>,
    /// Whether audio should be played live.
    pub play: bool,
}

/// Resolves CLI flags into an output route.
pub fn output_route(cli: &Cli) -> OutputRoute {
    OutputRoute {
        output: cli.output.clone(),
        play: cli.play || cli.output.is_none(),
    }
}
