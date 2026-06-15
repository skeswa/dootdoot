//! `VOICE_V7` swept-oscillator whistle gesture and wider pitch span tests.

use dootdoot_core::{
    PITCH_SEMITONE_SPAN, WHISTLE_PITCH_CEILING_HZ, WHISTLE_TARGET_HZ,
    WIDE_GESTURE_PITCH_SPAN_SEMITONES, pitch_center_hz, pitch_center_hz_with_span,
    whistle_sweep_pitch_hz,
};

fn bits(value: f64) -> u64 {
    value.to_bits()
}

#[test]
fn whistle_constants_target_the_whistle_band() {
    const {
        assert!(WHISTLE_TARGET_HZ >= 2_000.0 && WHISTLE_TARGET_HZ <= WHISTLE_PITCH_CEILING_HZ);
        assert!(WHISTLE_PITCH_CEILING_HZ >= 4_000.0 && WHISTLE_PITCH_CEILING_HZ <= 5_000.0);
        assert!(WIDE_GESTURE_PITCH_SPAN_SEMITONES > PITCH_SEMITONE_SPAN);
    }
}

#[test]
fn whistle_sweep_starts_at_the_starting_pitch() {
    assert_eq!(bits(whistle_sweep_pitch_hz(760.0, 1.0, 0.0)), bits(760.0));
}

#[test]
fn whistle_sweep_climbs_into_the_whistle_band_at_full_amount() {
    let end = whistle_sweep_pitch_hz(760.0, 1.0, 1.0);

    assert_eq!(bits(end), bits(WHISTLE_PITCH_CEILING_HZ));
    assert!(end >= 2_000.0);
}

#[test]
fn whistle_sweep_is_monotonic_in_progress() {
    let mut previous = whistle_sweep_pitch_hz(700.0, 0.8, 0.0);

    for step in 1..=64 {
        let progress = f64::from(step) / 64.0;
        let current = whistle_sweep_pitch_hz(700.0, 0.8, progress);

        assert!(
            current >= previous - 1e-9,
            "whistle sweep should rise monotonically: {current} < {previous} at {progress}",
        );
        previous = current;
    }
}

#[test]
fn whistle_sweep_amount_zero_is_a_no_op() {
    for step in 0..=16 {
        let progress = f64::from(step) / 16.0;

        assert_eq!(
            bits(whistle_sweep_pitch_hz(640.0, 0.0, progress)),
            bits(640.0)
        );
    }
}

#[test]
fn whistle_sweep_is_bounded_and_finite_over_a_grid() {
    let starts = [
        -100.0,
        0.0,
        f64::NAN,
        f64::INFINITY,
        320.0,
        760.0,
        1_135.0,
        9_000.0,
    ];
    let amounts = [-1.0, 0.0, 0.5, 1.0, 2.0, f64::NAN];
    let progresses = [-1.0, 0.0, 0.37, 1.0, 4.0, f64::NAN];

    for start in starts {
        for amount in amounts {
            for progress in progresses {
                let value = whistle_sweep_pitch_hz(start, amount, progress);

                assert!(value.is_finite(), "whistle sweep must be finite");
                assert!(value > 0.0, "whistle sweep must stay audible/positive");
                assert!(
                    value <= WHISTLE_PITCH_CEILING_HZ,
                    "whistle sweep must respect the ceiling: {value}",
                );
            }
        }
    }
}

#[test]
fn whistle_sweep_is_deterministic() {
    let first = whistle_sweep_pitch_hz(812.5, 0.63, 0.41);
    let second = whistle_sweep_pitch_hz(812.5, 0.63, 0.41);

    assert_eq!(first.to_bits(), second.to_bits());
}

#[test]
fn wide_pitch_span_leaves_the_default_band() {
    let default_ceiling = pitch_center_hz(1.0);
    let wide_ceiling = pitch_center_hz_with_span(1.0, WIDE_GESTURE_PITCH_SPAN_SEMITONES);

    assert!(
        wide_ceiling > default_ceiling,
        "wide span should reach higher than the default span: {wide_ceiling} <= {default_ceiling}",
    );
    assert!(
        wide_ceiling > 1_800.0,
        "wide span should leave the established register: {wide_ceiling}",
    );
    assert!(wide_ceiling < WHISTLE_PITCH_CEILING_HZ);
}

#[test]
fn wide_pitch_span_matches_default_span_for_default_value() {
    assert_eq!(
        pitch_center_hz_with_span(0.5, PITCH_SEMITONE_SPAN).to_bits(),
        pitch_center_hz(0.5).to_bits(),
    );
}

#[test]
fn wide_pitch_span_is_bounded_and_finite() {
    for raw in [-4.0, -1.0, 0.0, 0.5, 1.0, 4.0, f64::NAN, f64::INFINITY] {
        let value = pitch_center_hz_with_span(raw, WIDE_GESTURE_PITCH_SPAN_SEMITONES);

        assert!(value.is_finite() && value > 0.0);
    }
}
