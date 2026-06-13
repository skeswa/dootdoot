//! Command-line shell for dootdoot.

use std::{error::Error, fs, process::ExitCode};

use clap::Parser;
use dootdoot::{Cli, ResolvedInput, output_route, play_buffer};
use dootdoot_core::{render_canonical_buffer, render_text_canonical_buffer, wav_bytes};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let input = dootdoot::read_resolved_input(&cli)?;
    let route = output_route(&cli);
    let buffer = match input {
        ResolvedInput::Text(text) => render_text_canonical_buffer(&text)?,
        ResolvedInput::EmptyChirp => render_canonical_buffer(&[]),
    };

    if let Some(path) = &route.output {
        fs::write(path, wav_bytes(&buffer)?)?;
    }

    if route.play {
        play_buffer(&buffer)?;
    }

    Ok(())
}
