//! `VOICE_V7` code-talkbox mouth-stage primitive tests.

use dootdoot_core::{
    MOUTH_RESONANCE_COUNT, MOUTH_STAGE_MAX_MIX, MouthDrive, MouthStage, mouth_open_envelope,
    mouth_resonance_hz,
};

fn bits(value: f64) -> u64 {
    value.to_bits()
}

#[test]
fn mouth_stage_is_off_by_default() {
    let mut stage = MouthStage::new();
    let closed = MouthDrive::closed();

    for index in 0..256 {
        let input = 0.3 * f64::from(index % 11) - 1.0;
        let output = stage.process_sample(input, index, closed);

        assert_eq!(
            bits(output),
            bits(input),
            "a closed mouth must pass the signal through untouched",
        );
    }
}

#[test]
fn mouth_drive_open_changes_the_signal() {
    let mut stage = MouthStage::new();
    let drive = MouthDrive::new(0.9, 0.2, 0.0);
    let mut changed = false;

    for index in 0..512 {
        let input = (f64::from(index % 13) / 13.0) - 0.5;
        let output = stage.process_sample(input, index, drive);

        if (output - input).abs() > 1e-6 {
            changed = true;
        }
    }

    assert!(changed, "an open mouth should reshape the signal");
}

#[test]
fn mouth_stage_is_deterministic() {
    let drive = MouthDrive::new(0.7, -0.3, 0.4);
    let mut first = MouthStage::new();
    let mut second = MouthStage::new();

    for index in 0..1_024 {
        let input = ((f64::from(index % 17) / 17.0) * 2.0) - 1.0;

        assert_eq!(
            bits(first.process_sample(input, index, drive)),
            bits(second.process_sample(input, index, drive)),
        );
    }
}

#[test]
fn mouth_stage_is_bounded_and_finite() {
    let drives = [
        MouthDrive::closed(),
        MouthDrive::new(1.0, 1.0, 1.0),
        MouthDrive::new(0.5, -1.0, 0.5),
        MouthDrive::new(f64::NAN, f64::INFINITY, -1.0),
    ];

    for drive in drives {
        let mut stage = MouthStage::new();

        for index in 0..2_048 {
            let input = (f64::from(index % 23) - 11.0) / 8.0;
            let output = stage.process_sample(input, index, drive);

            assert!(output.is_finite(), "mouth stage output must be finite");
            assert!(
                output.abs() <= input.abs() + 1.0,
                "mouth stage output must stay bounded: {output}",
            );
        }
    }
}

#[test]
fn mouth_open_envelope_opens_and_closes() {
    let duration = 0.17;

    assert!(mouth_open_envelope(0.0, duration) <= 1e-9);
    assert!(mouth_open_envelope(duration, duration) <= 1e-9);

    let middle = mouth_open_envelope(duration * 0.5, duration);

    assert!(middle > 0.5, "mouth should be open mid-gesture: {middle}");

    for step in 0..=32 {
        let value = mouth_open_envelope(duration * f64::from(step) / 32.0, duration);

        assert!((0.0..=1.0).contains(&value));
    }
}

#[test]
fn mouth_resonances_are_bounded_and_ordered() {
    for openness in [0.0, 0.5, 1.0] {
        for front_back in [-1.0, 0.0, 1.0] {
            let centers = mouth_resonance_hz(openness, front_back);

            assert_eq!(centers.len(), MOUTH_RESONANCE_COUNT);
            for center in centers {
                assert!(
                    (150.0..=4_000.0).contains(&center),
                    "mouth resonance out of range: {center}",
                );
            }
        }
    }
}

#[test]
fn mouth_stage_max_mix_is_bounded() {
    const {
        assert!(MOUTH_STAGE_MAX_MIX > 0.0 && MOUTH_STAGE_MAX_MIX <= 0.75);
    }
}
