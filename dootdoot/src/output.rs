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
///
/// Audio plays only when `--play` is set explicitly, or when the invocation is
/// a bare render with neither an output file (`-o`) nor `--explain`. Both `-o`
/// and `--explain` suppress the default live playback, since each is a
/// non-listening use (writing a file, or inspecting the per-token table).
pub fn output_route(cli: &Cli) -> OutputRoute {
    let bare_render = cli.output.is_none() && !cli.explain;

    OutputRoute {
        output: cli.output.clone(),
        play: cli.play || bare_render,
    }
}
