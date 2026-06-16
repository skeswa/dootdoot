//! CLI explanation table tests.

use std::{fs, path::PathBuf, process::Command};

use dootdoot::explain_table_for_text;

#[test]
fn explain_table_includes_semantic_and_control_rows() {
    let table = explain_table_for_text("hello?").expect("explain table should render");

    let header = table
        .lines()
        .next()
        .expect("explain table should have a header");
    assert!(header.starts_with("token") && header.contains("pitch") && header.contains("role"));
    assert!(
        table
            .lines()
            .any(|line| line.starts_with('?') && line.contains("control:question"))
    );
    assert!(
        table
            .lines()
            .any(|line| { line.starts_with("hello ") && !line.contains("control:") })
    );
}

#[test]
fn binary_writes_explain_table_to_stderr_only() {
    let path = unique_wav_path("explain");
    let _ = fs::remove_file(&path);
    let output = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .args(["hello?", "-o"])
        .arg(&path)
        .arg("--explain")
        .output()
        .expect("test harness provides the compiled dootdoot binary path");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");

    assert!(output.status.success());
    assert!(stdout.is_empty());
    assert!(stderr.starts_with("token"));
    assert!(
        stderr
            .lines()
            .any(|line| line.starts_with('?') && line.contains("control:question"))
    );
    assert!(path.exists());

    fs::remove_file(path).expect("temporary wav should be removable");
}

fn unique_wav_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "dootdoot-{label}-{}-{}.wav",
        std::process::id(),
        std::thread::current().name().unwrap_or("test"),
    ))
}
