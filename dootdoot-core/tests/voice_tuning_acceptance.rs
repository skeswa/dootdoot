//! Voice tuning acceptance documentation tests.

const TUNING: &str = include_str!("../../docs/validation/voice-tuning.md");
const VOICE_V2: &str = include_str!("../../docs/validation/voice-v2-expressiveness.md");
const VOICE_V3: &str = include_str!("../../docs/validation/voice-v3-smoothing.md");
const VOICE_V4: &str = include_str!("../../docs/validation/voice-v4-onset-smoothing.md");

#[test]
fn integrated_tuning_acceptance_records_phase_seven_decision() {
    for expected in [
        "Accepted for VOICE_V1",
        "BB-8 reference",
        "dootdoot corpus",
        "body",
        "upper-mid brightness",
        "gesture motion",
        "harmonicity",
        "phrase air",
        "2-5 kHz",
        ">6 kHz",
    ] {
        assert!(
            TUNING.contains(expected),
            "voice tuning note should mention {expected}",
        );
    }
}

#[test]
fn voice_v2_expressiveness_acceptance_records_freeze_decision() {
    for expected in [
        "Accepted for VOICE_V2",
        "dootdoot VOICE_V2",
        "contextual clips",
        "lost-friends-sad",
        "enemy-approaching-alarm",
        "phrase",
        "affect",
        "complexity",
        "archetype",
        "golden WAV hashes",
    ] {
        assert!(
            VOICE_V2.contains(expected),
            "VOICE_V2 acceptance note should mention {expected}",
        );
    }
}

#[test]
fn voice_v3_smoothing_acceptance_records_phrase_continuity_decision() {
    for expected in [
        "Accepted for VOICE_V3",
        "dootdoot VOICE_V3",
        "phrase-continuity",
        "transition bridges",
        "connected envelope",
        "hard zero runs",
        "active islands",
        "golden WAV hashes",
    ] {
        assert!(
            VOICE_V3.contains(expected),
            "VOICE_V3 acceptance note should mention {expected}",
        );
    }
}

#[test]
fn voice_v4_onset_smoothing_acceptance_records_repeated_subword_decision() {
    for expected in [
        "Accepted for VOICE_V4",
        "dootdoot VOICE_V4",
        "repeated-onset",
        "Connected syllables",
        "attack peak",
        "roughness ratio",
        "golden WAV hashes",
    ] {
        assert!(
            VOICE_V4.contains(expected),
            "VOICE_V4 acceptance note should mention {expected}",
        );
    }
}
