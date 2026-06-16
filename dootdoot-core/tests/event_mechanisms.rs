//! `VOICE_V7` event-based mechanisms and imperfection tests.

use dootdoot_core::{
    imperfection_detune_cents, render_text_canonical_buffer, sparkle_event_gain,
    warble_phase_offset_for_syllable,
};

const DURATION: f64 = 0.170;

#[test]
fn neutral_brightness_keeps_the_v6_constant_sparkle() {
    for step in 0..=16 {
        let elapsed = DURATION * f64::from(step) / 16.0;

        assert_eq!(
            sparkle_event_gain(0.0, elapsed, DURATION).to_bits(),
            1.0_f64.to_bits()
        );
    }
}

#[test]
fn bright_sparkle_is_event_shaped() {
    assert!(
        sparkle_event_gain(0.6, 0.0, DURATION) <= 1e-9,
        "closed at the gesture start"
    );
    assert!(
        sparkle_event_gain(0.6, DURATION, DURATION) <= 1e-9,
        "closed at the gesture end",
    );

    let mid = sparkle_event_gain(0.6, DURATION * 0.5, DURATION);

    assert!(mid > 0.0, "open mid-gesture: {mid}");
}

#[test]
fn flourish_sparkle_peaks_above_ordinary() {
    let ordinary = sparkle_event_gain(0.45, DURATION * 0.5, DURATION);
    let flourish = sparkle_event_gain(0.85, DURATION * 0.5, DURATION);

    assert!(
        flourish > ordinary,
        "brighter gestures should reserve more sparkle: {flourish} <= {ordinary}",
    );
}

#[test]
fn sparkle_event_gain_is_bounded_and_finite() {
    for brightness in [-1.0, 0.0, 0.3, 1.0, 4.0, f64::NAN, f64::INFINITY] {
        for step in 0..=32 {
            let elapsed = DURATION * f64::from(step) / 32.0;
            let value = sparkle_event_gain(brightness, elapsed, DURATION);

            assert!(value.is_finite());
            assert!(
                (0.0..=1.8).contains(&value),
                "sparkle gain out of range: {value}"
            );
        }
    }
}

#[test]
fn neutral_tension_has_no_imperfection() {
    assert_eq!(
        imperfection_detune_cents(0.0, warble_phase_offset_for_syllable(3)).to_bits(),
        0.0_f64.to_bits(),
    );
}

#[test]
fn imperfection_is_bounded_and_varies_by_syllable() {
    let first = imperfection_detune_cents(0.8, warble_phase_offset_for_syllable(1));
    let second = imperfection_detune_cents(0.8, warble_phase_offset_for_syllable(2));

    assert!(first.abs() <= 6.0 && second.abs() <= 6.0);
    assert!(
        (first - second).abs() > 1e-9,
        "imperfection should vary between syllables",
    );
}

#[test]
fn imperfection_is_bounded_and_finite_over_a_grid() {
    for tension in [-1.0, 0.0, 0.5, 1.0, 2.0, f64::NAN] {
        for index in 0..64 {
            let value = imperfection_detune_cents(tension, warble_phase_offset_for_syllable(index));

            assert!(value.is_finite());
            assert!(value.abs() <= 6.0);
        }
    }
}

#[test]
fn rendering_stays_deterministic() {
    let text = "Hello - good morning Sandile. What are you doing today?!";

    assert_eq!(
        render_text_canonical_buffer(text).expect("first render"),
        render_text_canonical_buffer(text).expect("second render"),
    );
}
