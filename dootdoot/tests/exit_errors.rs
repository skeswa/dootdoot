//! CLI exit-code and error-message tests.

use std::{fs, path::PathBuf, process::Command};

#[test]
fn over_ceiling_input_exits_one_with_error_prefix() {
    let text = "hello ".repeat(8_001);
    let output = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .arg(text)
        .args(["-o", "ignored.wav"])
        .output()
        .expect("test harness provides the compiled dootdoot binary path");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.starts_with("error: input would render"));
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
