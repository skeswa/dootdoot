//! Release-automation workflow contract tests.

const CUT_RELEASE: &str = include_str!("../../.github/workflows/cut-release.yml");
const TAG_RELEASE: &str = include_str!("../../.github/workflows/tag-release.yml");

#[test]
fn cut_release_is_a_dispatch_that_opens_a_version_bump_pr() {
    for required in [
        "workflow_dispatch",
        // Bump level choice plus an explicit-version escape hatch.
        "options: [patch, minor, major]",
        "cargo update --workspace",
        // The bump lands via a PR (main is branch-protected).
        "gh pr create",
        "release: v",
        // Only ever run in the canonical repo.
        "github.repository == 'skeswa/dootdoot'",
    ] {
        assert!(
            CUT_RELEASE.contains(required),
            "cut-release workflow should contain {required}",
        );
    }
}

#[test]
fn tag_release_tags_version_bumps_with_a_pat() {
    for required in [
        // Fires when a bump lands on main.
        "branches: [main]",
        // Acts on an actual version change, not a brittle commit-message match.
        "HEAD~1:Cargo.toml",
        // The PAT is what lets the tag push trigger the dist release build.
        "RELEASE_TOKEN",
        "git push",
        "github.repository == 'skeswa/dootdoot'",
    ] {
        assert!(
            TAG_RELEASE.contains(required),
            "tag-release workflow should contain {required}",
        );
    }

    // A missing PAT must fail loudly rather than silently tagging without
    // triggering the release build.
    assert!(
        TAG_RELEASE.contains("RELEASE_TOKEN is unset"),
        "tag-release should error clearly when the PAT is missing",
    );
}
