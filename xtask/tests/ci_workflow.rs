//! CI workflow contract tests.

const CI_WORKFLOW: &str = include_str!("../../.github/workflows/ci.yml");

#[test]
fn ci_runs_build_lint_and_test_on_linux_and_macos() {
    for required_fragment in [
        // Linux stays GitHub-hosted; the macOS leg runs on the self-hosted Mac.
        "ubuntu-latest",
        "[self-hosted, macOS, ARM64]",
        "cargo build --workspace --all-targets",
        // rodio links ALSA on Linux; the dev headers must be installed first.
        "libasound2-dev",
        "scripts/lint",
        "Golden WAV determinism",
        "cargo test -p dootdoot-core --test golden_wav",
        "cargo test",
    ] {
        assert!(
            CI_WORKFLOW.contains(required_fragment),
            "CI workflow should contain {required_fragment}",
        );
    }
}

#[test]
fn ci_is_hardened_for_the_self_hosted_runner() {
    for required_fragment in [
        // Least-privilege: no token scope by default, read-only per job.
        "permissions: {}",
        "contents: read",
        // Don't leave the token in .git/config on the persistent runner.
        "persist-credentials: false",
        // Only ever run in the canonical repo, so fork code can't reach the Mac.
        "github.repository == 'skeswa/dootdoot'",
        // Actions pinned to commit SHAs, not movable tags (supply chain).
        "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0",
        "actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9",
        // Denial-of-service guards on the shared runner.
        "timeout-minutes:",
        "concurrency:",
    ] {
        assert!(
            CI_WORKFLOW.contains(required_fragment),
            "hardened CI workflow should contain {required_fragment}",
        );
    }

    // No fork-reachable trigger while CI runs on the self-hosted Mac. Match the
    // trigger keys specifically, so prose mentioning pull requests is fine.
    for fork_trigger in ["pull_request:", "pull_request_target:"] {
        assert!(
            !CI_WORKFLOW.contains(fork_trigger),
            "CI must not be {fork_trigger}-triggered while it runs on a self-hosted runner",
        );
    }
    // The macOS leg must be the self-hosted runner, never a GitHub-hosted one.
    assert!(
        !CI_WORKFLOW.contains("macos-latest"),
        "the macOS leg should run on the self-hosted runner, not macos-latest",
    );
    // No bare, movable action tags (any `@v<N>` ref, regardless of version).
    for floating_tag in ["actions/checkout@v", "actions/cache@v"] {
        assert!(
            !CI_WORKFLOW.contains(floating_tag),
            "actions must be pinned to a commit SHA, not the floating {floating_tag}",
        );
    }
}
