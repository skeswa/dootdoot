//! CI workflow contract tests.

const CI_WORKFLOW: &str = include_str!("../../.github/workflows/ci.yml");

#[test]
fn ci_runs_build_lint_and_test_on_linux_and_macos() {
    for required_fragment in [
        "ubuntu-latest",
        "macos-latest",
        "actions/cache@v4",
        "cargo build --workspace --all-targets",
        "scripts/lint",
        "Golden WAV hashes",
        "cargo test -p dootdoot-core --test golden_wav_hashes",
        "cargo test",
    ] {
        assert!(
            CI_WORKFLOW.contains(required_fragment),
            "CI workflow should contain {required_fragment}",
        );
    }
}
