//! `VOICE_V2` affect asset tests.

use std::{fs, path::Path};

const VADER_VALENCE: &str = include_str!("../../assets/affect/vader_valence.tsv");
const AROUSAL_SIGNALS: &str = include_str!("../../assets/affect/arousal_signals.toml");
const AFFECT_README: &str = include_str!("../../assets/affect/README.md");

#[test]
fn vader_valence_asset_contains_pinned_mit_scores() {
    assert!(AFFECT_README.contains("MIT"));
    assert!(AFFECT_README.contains("cjhutto/vaderSentiment"));
    assert!(VADER_VALENCE.lines().any(|line| line == "happy\t2.7"));
    assert!(VADER_VALENCE.lines().any(|line| line == "excellent\t2.7"));
    assert!(VADER_VALENCE.lines().any(|line| line == "bad\t-2.5"));
    assert!(VADER_VALENCE.lines().any(|line| line == "terrible\t-2.1"));
}

#[test]
fn arousal_signal_asset_defines_only_owned_proxies() {
    for expected in [
        "[punctuation]",
        "[repeated_markers]",
        "[all_caps]",
        "[intensifiers]",
        "[token_count]",
        "[complexity]",
    ] {
        assert!(
            AROUSAL_SIGNALS.contains(expected),
            "arousal signal asset should contain {expected}",
        );
    }

    for intensifier in ["very", "really", "extremely", "barely"] {
        assert!(
            AROUSAL_SIGNALS.contains(intensifier),
            "owned intensifier list should contain {intensifier}",
        );
    }
}

#[test]
fn affect_assets_do_not_commit_disallowed_lexicons() {
    let affect_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/affect");
    let disallowed = [
        "afinn",
        "sentiwordnet",
        "subtlex",
        "nrc-vad",
        "warriner",
        "zipf",
    ];

    for entry in fs::read_dir(affect_dir).expect("affect asset directory should be readable") {
        let entry = entry.expect("affect asset entry should be readable");
        let filename = entry.file_name().to_string_lossy().to_lowercase();

        assert!(
            !disallowed
                .iter()
                .any(|disallowed_name| filename.contains(disallowed_name)),
            "disallowed affect asset committed: {filename}",
        );
    }
}
