//! Voice tuning acceptance documentation tests.

const TUNING: &str = include_str!("../../docs/validation/voice-tuning.md");
const FORMAT_V2: &str = include_str!("../../docs/validation/format-v2-expressiveness.md");

#[test]
fn integrated_tuning_acceptance_records_phase_seven_decision() {
    for expected in [
        "Accepted for FORMAT_V1",
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
fn format_v2_expressiveness_acceptance_records_freeze_decision() {
    for expected in [
        "Accepted for FORMAT_V2",
        "dootdoot FORMAT_V2",
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
            FORMAT_V2.contains(expected),
            "FORMAT_V2 acceptance note should mention {expected}",
        );
    }
}
