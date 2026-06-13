//! CLI output routing tests.

use std::{fs, path::PathBuf, process::Command};

use clap::Parser;
use dootdoot::{Cli, OutputRoute, output_route};

#[test]
fn route_plays_when_no_output_path_is_set() {
    let cli = Cli::try_parse_from(["dootdoot", "hello"]).expect("arguments parse");

    assert_eq!(
        output_route(&cli),
        OutputRoute {
            output: None,
            play: true,
        },
    );
}

#[test]
fn route_writes_without_playback_when_output_path_is_set() {
    let cli =
        Cli::try_parse_from(["dootdoot", "hello", "-o", "hello.wav"]).expect("arguments parse");

    assert_eq!(
        output_route(&cli),
        OutputRoute {
            output: Some(PathBuf::from("hello.wav")),
            play: false,
        },
    );
}

#[test]
fn route_writes_and_plays_when_output_and_play_are_set() {
    let cli = Cli::try_parse_from(["dootdoot", "hello", "-o", "hello.wav", "--play"])
        .expect("arguments parse");

    assert_eq!(
        output_route(&cli),
        OutputRoute {
            output: Some(PathBuf::from("hello.wav")),
            play: true,
        },
    );
}

#[test]
fn binary_writes_wav_when_output_path_is_set() {
    let path = unique_wav_path("write-output");
    let _ = fs::remove_file(&path);
    let output = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .args(["?", "-o"])
        .arg(&path)
        .output()
        .expect("test harness provides the compiled dootdoot binary path");
    let bytes = fs::read(&path).expect("output wav should be written");

    assert!(output.status.success());
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");

    fs::remove_file(path).expect("temporary wav should be removable");
}

fn unique_wav_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "dootdoot-{label}-{}-{}.wav",
        std::process::id(),
        std::thread::current().name().unwrap_or("test"),
    ))
}
