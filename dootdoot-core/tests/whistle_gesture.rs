//! `VOICE_V7` swept-oscillator whistle gesture and wider pitch span tests.

use dootdoot_core::{
    PITCH_SEMITONE_SPAN, PhraseRole, WHISTLE_FLOOR_HZ, WHISTLE_PITCH_CEILING_HZ, WHISTLE_TARGET_HZ,
    WIDE_GESTURE_PITCH_SPAN_SEMITONES, apply_whistle_sweep_hz, gesture_pitch_span_semitones,
    pitch_center_hz, pitch_center_hz_with_span, whistle_sweep_amount, whistle_sweep_pitch_hz,
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
fn whistle_floor_sits_below_the_register_and_target() {
    const {
        assert!(WHISTLE_FLOOR_HZ >= 200.0 && WHISTLE_FLOOR_HZ <= 500.0);
        assert!(WHISTLE_FLOOR_HZ < WHISTLE_TARGET_HZ);
    }
}

#[test]
fn whistle_sweep_descends_toward_the_floor_at_negative_amount() {
    let start = 900.0;
    let end = whistle_sweep_pitch_hz(start, -1.0, 1.0);

    assert!(end < start, "negative amount should descend: {end} >= {start}");
    assert!(
        (end - WHISTLE_FLOOR_HZ).abs() <= 1.0,
        "full negative sweep should land at the floor: {end} vs {WHISTLE_FLOOR_HZ}",
    );
}

#[test]
fn whistle_sweep_descent_is_monotonic_in_progress() {
    let mut previous = whistle_sweep_pitch_hz(820.0, -0.8, 0.0);

    for step in 1..=64 {
        let progress = f64::from(step) / 64.0;
        let current = whistle_sweep_pitch_hz(820.0, -0.8, progress);

        assert!(
            current <= previous + 1e-9,
            "descending whistle should fall monotonically: {current} > {previous} at {progress}",
        );
        previous = current;
    }
}

#[test]
fn whistle_sweep_descent_starts_at_the_starting_pitch() {
    assert_eq!(bits(whistle_sweep_pitch_hz(760.0, -1.0, 0.0)), bits(760.0));
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
fn body_accent_whistles_hard_enough_to_leave_the_register() {
    // VOICE_V10: once a body accent engages, it must sweep substantially (not the
    // near-zero V8 ramp) so the dominant peak actually rides into the whistle
    // band — the taxonomy gap was that engaged accents barely whistled.
    let amount = whistle_sweep_amount(PhraseRole::ChattyReply, 0.85, 1.0);

    assert!(
        amount >= 0.5,
        "an engaged body accent should sweep hard, got {amount}",
    );
}

#[test]
fn non_accent_body_syllable_does_not_whistle() {
    // The promoted accent reaches ~0.80+ tension; non-accent body syllables top
    // out around 0.75, so they must stay below the gate (no shrill multi-whistle).
    assert!(whistle_sweep_amount(PhraseRole::ChattyReply, 0.75, 1.0).abs() < 1e-12);
    assert!(whistle_sweep_amount(PhraseRole::Probe, 0.75, 1.0).abs() < 1e-12);
}

#[test]
fn whistle_begins_earlier_in_the_syllable() {
    // VOICE_V10: the sweep starts earlier so more frames ride high. By 35% of the
    // syllable it has already begun moving off the starting pitch.
    let start = 760.0;
    let moved = apply_whistle_sweep_hz(start, 1.0, 0.35, 1.0);

    assert!(
        moved > start + 1.0,
        "sweep should have begun by 35% progress, got {moved}",
    );
}

#[test]
fn flourish_amount_follows_pitch_velocity_sign() {
    assert!(whistle_sweep_amount(PhraseRole::TerminalFlourish, 0.9, -1.0) < 0.0);
    assert!(whistle_sweep_amount(PhraseRole::TerminalFlourish, 0.9, 1.0) > 0.0);
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
fn body_accent_uses_a_wider_span_than_the_flourish() {
    // VOICE_V10: the one promoted body accent swoops widest (toward BB-8's
    // multi-octave gestures); the terminal flourish stays at the wide span; a
    // non-whistling syllable keeps the default span.
    let accent = gesture_pitch_span_semitones(PhraseRole::ChattyReply, 0.6);
    let flourish = gesture_pitch_span_semitones(PhraseRole::TerminalFlourish, 0.6);
    let plain = gesture_pitch_span_semitones(PhraseRole::ChattyReply, 0.0);

    assert!(
        accent > flourish,
        "the body accent span ({accent}) should exceed the flourish span ({flourish})",
    );
    assert!(
        accent <= 36.0,
        "the accent span ({accent}) must stay bounded inside the droid range",
    );
    assert_eq!(bits(flourish), bits(WIDE_GESTURE_PITCH_SPAN_SEMITONES));
    assert_eq!(bits(plain), bits(PITCH_SEMITONE_SPAN));
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
