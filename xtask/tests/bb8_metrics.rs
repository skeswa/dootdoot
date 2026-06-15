//! BB-8 tuning metrics workflow tests.

use std::fs;

const SCRIPT: &str = include_str!("../../scripts/bb8-metrics");
const ANALYSIS: &str = include_str!("../../docs/research/bb8-sound-signature-analysis.md");

#[test]
fn bb8_metrics_script_decodes_references_and_runs_xtask_report() {
    for expected in [
        "ffmpeg",
        "-ac 1",
        "-ar 44100",
        "bb8-metrics",
        "/Users/skeswa/repos/anddav87/bb8-sounds",
    ] {
        assert!(
            SCRIPT.contains(expected),
            "metrics script should mention {expected}",
        );
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
