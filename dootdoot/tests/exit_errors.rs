//! CLI exit-code and error-message tests.

use std::{fs, path::PathBuf, process::Command};

#[test]
fn over_ceiling_input_exits_one_with_error_prefix() {
    // VOICE_V8 shortens neutral word gaps (silent rests replace tonal bridges),
    // so the over-ceiling fixture needs more words to still exceed 30 minutes.
    let text = "hello ".repeat(12_000);
    // Write to a unique temp path, not a relative "ignored.wav", so a regression
    // that renders instead of rejecting cannot litter the working tree.
    let path = missing_output_path("over-ceiling").with_file_name("ignored.wav");
    let _ = fs::remove_file(&path);
    let output = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .arg(text)
        .arg("-o")
        .arg(&path)
        .output()
        .expect("test harness provides the compiled dootdoot binary path");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.starts_with("error: input would render"));
    assert!(!path.exists(), "rejected input must not write audio");
}

#[test]
fn failed_wav_write_exits_one_with_output_path_context() {
    let path = missing_output_path("write-error");
    let _ = fs::remove_file(&path);
    let output = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .args(["hello", "-o"])
        .arg(&path)
        .output()
        .expect("test harness provides the compiled dootdoot binary path");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.starts_with("error: failed to write WAV output"));
    assert!(stderr.contains(path.to_string_lossy().as_ref()));
    assert!(!path.exists());
}

fn missing_output_path(label: &str) -> PathBuf {
    std::env::temp_dir()
        .join(format!("dootdoot-{label}-{}", std::process::id()))
        .join("out.wav")
}
