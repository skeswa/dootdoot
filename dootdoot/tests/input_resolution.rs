//! CLI input resolution tests.

use clap::Parser;
use dootdoot::{Cli, ResolvedInput, StdinInput, resolve_input};

#[test]
fn positional_text_wins_over_piped_stdin() {
    let cli = Cli::try_parse_from(["dootdoot", "hello"]).expect("text argument parses");

    assert_eq!(
        resolve_input(&cli, StdinInput::Piped("ignored")),
        ResolvedInput::Text("hello".to_owned()),
    );
}

#[test]
fn piped_stdin_is_used_when_text_is_absent() {
    let cli = Cli::try_parse_from(["dootdoot"]).expect("empty argument list parses");

    assert_eq!(
        resolve_input(&cli, StdinInput::Piped("hello from stdin\n")),
        ResolvedInput::Text("hello from stdin".to_owned()),
    );
}

#[test]
fn missing_or_whitespace_input_routes_to_empty_chirp() {
    let no_text = Cli::try_parse_from(["dootdoot"]).expect("empty argument list parses");
    let whitespace_text =
        Cli::try_parse_from(["dootdoot", "   \n"]).expect("whitespace argument parses");

    assert!(resolve_input(&no_text, StdinInput::Terminal).is_empty_chirp());
    assert!(resolve_input(&no_text, StdinInput::Piped(" \t\n")).is_empty_chirp());
    assert!(resolve_input(&whitespace_text, StdinInput::Piped("ignored")).is_empty_chirp());
}
