//! Voice tuning acceptance documentation tests.

const TUNING: &str = include_str!("../../docs/validation/voice-tuning.md");

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
