//! CLI argument model tests.

use std::{path::PathBuf, process::Command};

use clap::Parser;
use dootdoot::Cli;
use dootdoot_core::ACTIVE_FORMAT;

#[test]
fn cli_parses_text_output_play_and_explain_flags() {
    let cli = Cli::try_parse_from([
        "dootdoot",
        "hello there",
        "-o",
        "hello.wav",
        "--play",
        "--explain",
    ])
    .expect("valid CLI arguments should parse");

    assert_eq!(cli.text, Some("hello there".to_owned()));
    assert_eq!(cli.output, Some(PathBuf::from("hello.wav")));
    assert!(cli.play);
    assert!(cli.explain);
}

#[test]
fn binary_version_surfaces_active_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .arg("--version")
        .output()
        .expect("test harness provides the compiled dootdoot binary path");
    let stdout = String::from_utf8(output.stdout).expect("version output is utf8");

    assert!(output.status.success());
    assert!(stdout.contains(ACTIVE_FORMAT));
}

#[test]
fn binary_help_lists_supported_arguments() {
    let output = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .arg("--help")
        .output()
        .expect("test harness provides the compiled dootdoot binary path");
    let stdout = String::from_utf8(output.stdout).expect("help output is utf8");

    assert!(output.status.success());
    assert!(stdout.contains("TEXT"));
    assert!(stdout.contains("-o"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--play"));
    assert!(stdout.contains("--explain"));
}
