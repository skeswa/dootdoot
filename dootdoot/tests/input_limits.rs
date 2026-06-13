//! Input limit tests.

use std::{fs, path::PathBuf, process::Command};

use dootdoot::{HARD_LIMIT_SAMPLES, InputLimitStatus, WARNING_LIMIT_SAMPLES, check_input_limits};

#[test]
fn input_limit_status_warns_above_warning_threshold() {
    let status = check_input_limits(WARNING_LIMIT_SAMPLES + 1);

    assert_eq!(
        status,
        InputLimitStatus::Warn {
            sample_count: WARNING_LIMIT_SAMPLES + 1,
            byte_count: (WARNING_LIMIT_SAMPLES + 1) * 2,
        },
    );
}

#[test]
fn input_limit_status_rejects_above_hard_threshold() {
    let status = check_input_limits(HARD_LIMIT_SAMPLES + 1);

    assert_eq!(
        status,
        InputLimitStatus::Reject {
            sample_count: HARD_LIMIT_SAMPLES + 1,
            byte_count: (HARD_LIMIT_SAMPLES + 1) * 2,
        },
    );
}

#[test]
fn binary_rejects_over_ceiling_input_without_writing_audio() {
    let path = unique_wav_path("limit");
    let _ = fs::remove_file(&path);
    let text = "hello ".repeat(8_001);
    let output = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .arg(text)
        .args(["-o"])
        .arg(&path)
        .output()
        .expect("test harness provides the compiled dootdoot binary path");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");

    assert!(!output.status.success());
    assert!(stderr.contains("exceeds the 30 minute"));
    assert!(!path.exists());
}

fn unique_wav_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "dootdoot-{label}-{}-{}.wav",
        std::process::id(),
        std::thread::current().name().unwrap_or("test"),
    ))
}
