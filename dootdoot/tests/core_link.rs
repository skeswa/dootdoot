//! Binary crate linkage tests.

use std::process::Command;

use dootdoot_core::FORMAT_V1;

#[test]
fn binary_links_core_and_exits_successfully() {
    assert_eq!(FORMAT_V1, "FORMAT_V1");

    let status = Command::new(env!("CARGO_BIN_EXE_dootdoot"))
        .arg("--version")
        .status()
        .expect("test harness provides the compiled dootdoot binary path");

    assert!(status.success());
}
