//! Command-line shell for dootdoot.

use std::{error::Error, fmt, fs, io, path::PathBuf, process::ExitCode};

use clap::Parser;
use dootdoot::{
    Cli, InputLimitError, PlaybackError, ResolvedInput, enforce_input_limits,
    explain_table_for_empty_chirp, explain_table_for_text, output_route, play_buffer,
};
use dootdoot_core::{
    EngineError, WavError, estimate_utterance_sample_count, render_canonical_buffer,
    sequence_events_for_text, wav_bytes,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            CliError::exit_code()
        }
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    let input = dootdoot::read_resolved_input(&cli).map_err(CliError::ReadStdin)?;
    let route = output_route(&cli);
    let buffer = match &input {
        ResolvedInput::Text(text) => {
            let events = sequence_events_for_text(text)?;

            if let Some(warning) = enforce_input_limits(estimate_utterance_sample_count(&events))? {
                eprintln!("{warning}");
            }

            render_canonical_buffer(&events)
        }
        ResolvedInput::EmptyChirp => render_canonical_buffer(&[]),
    };

    if cli.explain {
        match &input {
            ResolvedInput::Text(text) => eprint!("{}", explain_table_for_text(text)?),
            ResolvedInput::EmptyChirp => eprint!("{}", explain_table_for_empty_chirp()),
        }
    }

    if let Some(path) = &route.output {
        let bytes = wav_bytes(&buffer)?;

        fs::write(path, bytes).map_err(|source| CliError::WriteOutput {
            path: path.clone(),
            source,
        })?;
    }

    if route.play {
        play_buffer(&buffer)?;
    }

    Ok(())
}

#[derive(Debug)]
enum CliError {
    ReadStdin(io::Error),
    Engine(EngineError),
    InputLimit(InputLimitError),
    Wav(WavError),
    WriteOutput { path: PathBuf, source: io::Error },
    Playback(PlaybackError),
}

impl CliError {
    fn exit_code() -> ExitCode {
        ExitCode::from(1)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadStdin(source) => write!(formatter, "failed to read stdin: {source}"),
            Self::Engine(source) => write!(formatter, "failed to synthesize input: {source}"),
            Self::InputLimit(source) => write!(formatter, "{source}"),
            Self::Wav(source) => write!(formatter, "failed to encode WAV output: {source}"),
            Self::WriteOutput { path, source } => {
                write!(
                    formatter,
                    "failed to write WAV output to {}: {source}",
                    path.display(),
                )
            }
            Self::Playback(source) => write!(formatter, "failed to play audio: {source}"),
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadStdin(source) | Self::WriteOutput { source, .. } => Some(source),
            Self::Engine(source) => Some(source),
            Self::InputLimit(source) => Some(source),
            Self::Wav(source) => Some(source),
            Self::Playback(source) => Some(source),
        }
    }
}

impl From<EngineError> for CliError {
    fn from(source: EngineError) -> Self {
        Self::Engine(source)
    }
}

impl From<InputLimitError> for CliError {
    fn from(source: InputLimitError) -> Self {
        Self::InputLimit(source)
    }
}

impl From<WavError> for CliError {
    fn from(source: WavError) -> Self {
        Self::Wav(source)
    }
}

impl From<PlaybackError> for CliError {
    fn from(source: PlaybackError) -> Self {
        Self::Playback(source)
    }
}
