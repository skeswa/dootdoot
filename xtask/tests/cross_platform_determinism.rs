//! Cross-platform determinism CI contract tests.

const CI_WORKFLOW: &str = include_str!("../../.github/workflows/ci.yml");
const CROSS_PLATFORM: &str = include_str!("../../docs/reference/cross-platform-determinism.md");

#[test]
fn ci_verifies_golden_hashes_on_linux_and_macos() {
    for expected in [
        "ubuntu-latest",
        "macos-latest",
        "Cross-platform Golden WAV hashes",
        "cargo test -p dootdoot-core --test golden_wav_hashes",
    ] {
        assert!(
            CI_WORKFLOW.contains(expected),
            "CI workflow should mention {expected}",
        );
    }
}

#[test]
fn docs_record_verified_platform_scope() {
    for expected in [
        "active voice contract",
        "macOS",
        "Linux",
        "golden_wav_hashes",
        "identical hashes",
    ] {
        assert!(
            CROSS_PLATFORM.contains(expected),
            "cross-platform doc should mention {expected}",
        );
    }
}
