//! Command-line shell for dootdoot.

use std::process::ExitCode;

use clap::Parser;
use dootdoot::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match dootdoot::read_resolved_input(&cli) {
        Ok(_input) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("failed to read stdin: {error}");
            ExitCode::from(1)
        }
    }
}
