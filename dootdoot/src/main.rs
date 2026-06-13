//! Command-line shell for dootdoot.

use std::{error::Error, fs, process::ExitCode};

use clap::Parser;
use dootdoot::{
    Cli, ResolvedInput, explain_table_for_empty_chirp, explain_table_for_text, output_route,
    play_buffer,
};
use dootdoot_core::{
    estimate_utterance_sample_count, render_canonical_buffer, sequence_events_for_text, wav_bytes,
};

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
    let buffer = match &input {
        ResolvedInput::Text(text) => {
            let events = sequence_events_for_text(text)?;

            if let Some(warning) =
                dootdoot::enforce_input_limits(estimate_utterance_sample_count(&events))?
            {
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
        fs::write(path, wav_bytes(&buffer)?)?;
    }

    if route.play {
        play_buffer(&buffer)?;
    }

    Ok(())
}
