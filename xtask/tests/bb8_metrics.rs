//! BB-8 tuning metrics workflow tests.

use std::fs;

const SCRIPT: &str = include_str!("../../scripts/bb8-metrics");
const ANALYSIS: &str = include_str!("../../docs/research/bb8-sound-signature-analysis.md");

/// Docs whose code blocks invoke `scripts/bb8-metrics`. The script makes the
/// reference-recordings directory mandatory, so a bare `scripts/bb8-metrics`
/// command line in any of these would exit immediately with the usage error,
/// breaking the documented reproduction path.
const DOCS_WITH_INVOCATIONS: &[(&str, &str)] = &[
    (
        "corpus-timbre-texture",
        include_str!("../../docs/research/bb8-corpus-timbre-texture-analysis.md"),
    ),
    (
        "voice-v8-semantic-engagement",
        include_str!("../../docs/validation/voice-v8-semantic-engagement.md"),
    ),
    (
        "voice-v2-expressiveness",
        include_str!("../../docs/validation/voice-v2-expressiveness.md"),
    ),
    (
        "voice-tuning",
        include_str!("../../docs/validation/voice-tuning.md"),
    ),
];

#[test]
fn bb8_metrics_script_decodes_references_and_runs_xtask_report() {
    for expected in [
        "ffmpeg",
        "-ac 1",
        "-ar 44100",
        "bb8-metrics",
        "bb8-clips",
        "contextual-wav",
        "contextual",
        "reference-recordings-dir",
    ] {
        assert!(
            SCRIPT.contains(expected),
            "metrics script should mention {expected}",
        );
    }
}

#[test]
fn bb8_metrics_taxonomy_step_guards_empty_wav_directories() {
    // The taxonomy step must not pass an unexpanded glob to the analyzer when a
    // decoded directory is empty; it collects matches first and skips with a
    // warning when there are none, so an empty/mispointed corpus cannot abort
    // the whole `set -e` workflow.
    for expected in ["sound_taxonomy.py", "no WAVs", "run_taxonomy"] {
        assert!(
            SCRIPT.contains(expected),
            "metrics script should mention {expected}",
        );
    }
    assert!(
        !SCRIPT.contains("\"${wav_dir}\"/*.wav"),
        "taxonomy step must not pass an unguarded glob to the analyzer",
    );
}

#[test]
fn docs_invoke_bb8_metrics_with_a_reference_dir() {
    for (name, doc) in DOCS_WITH_INVOCATIONS {
        for line in doc.lines() {
            // A runnable command line begins (after indentation) with the bare
            // script path; inline prose mentions are wrapped in backticks or
            // start with markdown markers, so they do not match here.
            let Some(rest) = line.trim_start().strip_prefix("scripts/bb8-metrics") else {
                continue;
            };
            let rest = rest.trim_start();
            assert!(
                !rest.is_empty() && !rest.starts_with('#'),
                "{name}: runnable `scripts/bb8-metrics` line must pass a \
                 reference-recordings dir, found bare invocation: {line:?}",
            );
        }
    }
}

#[test]
fn bb8_metrics_report_includes_phase_seven_directional_fields() {
    let temp = std::env::temp_dir().join(format!("dootdoot-bb8-metrics-{}", std::process::id(),));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).expect("temporary metrics directory should be creatable");

    let report = xtask::run_with_args([
        "bb8-metrics",
        "--reference-wav-dir",
        temp.to_str().expect("temporary path should be utf8"),
        "--output-dir",
        temp.to_str().expect("temporary path should be utf8"),
    ])
    .expect("metrics command should run with an empty reference set");

    for expected in [
        "active fraction",
        "active island median ms",
        "magnitude centroid hz",
        "magnitude rolloff 85 hz",
        "dominant peak motion hz",
        "harmonicity",
        "sub-500 hz power",
        "500-2000 hz power",
        "2000-5000 hz power",
        "over-6000 hz power",
    ] {
        assert!(
            report.contains(expected),
            "metrics report should include {expected}",
        );
    }

    fs::remove_dir_all(temp).expect("temporary metrics directory should be removable");
}

#[test]
fn analysis_documents_metrics_caveats_and_upper_mid_target() {
    for expected in [
        "active-island duration",
        "gate-dependent",
        "magnitude spectrum",
        "2-5 kHz",
        "not above 6 kHz",
    ] {
        assert!(
            ANALYSIS.contains(expected),
            "analysis doc should mention {expected}",
        );
    }
}
