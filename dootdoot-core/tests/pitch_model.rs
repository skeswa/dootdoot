//! Pitch model tests.

use dootdoot_core::{
    PITCH_REGISTER_BIAS_HZ, PITCH_SEMITONE_SPAN, PORTAMENTO_SECONDS, exp, pitch_center_hz,
    portamento_pitch_hz, portamento_progress,
};

#[test]
fn pitch_center_maps_knob_to_high_register_span() {
    let octave_ratio = exp(core::f64::consts::LN_2 * (PITCH_SEMITONE_SPAN / 12.0));

    assert_eq!(
        pitch_center_hz(0.0).to_bits(),
        PITCH_REGISTER_BIAS_HZ.to_bits()
    );
    assert_eq!(
        pitch_center_hz(1.0).to_bits(),
        (PITCH_REGISTER_BIAS_HZ * octave_ratio).to_bits(),
    );
    assert_eq!(
        pitch_center_hz(-1.0).to_bits(),
        (PITCH_REGISTER_BIAS_HZ / octave_ratio).to_bits(),
    );
}

#[test]
fn portamento_pitch_reaches_endpoints_at_glide_bounds() {
    let start = 660.0_f64;
    let target = 990.0_f64;

    assert_eq!(
        portamento_pitch_hz(start, target, 0.0, 0.0).to_bits(),
        start.to_bits(),
    );
    assert_eq!(
        portamento_pitch_hz(start, target, 0.0, PORTAMENTO_SECONDS).to_bits(),
        target.to_bits(),
    );
    assert_eq!(
        portamento_pitch_hz(start, target, 0.0, PORTAMENTO_SECONDS * 2.0).to_bits(),
        target.to_bits(),
    );
}

#[test]
fn contour_shapes_mid_glide_progress_without_escaping_bounds() {
    let elapsed = PORTAMENTO_SECONDS * 0.25;
    let falling = portamento_progress(elapsed, -1.0);
    let neutral = portamento_progress(elapsed, 0.0);
    let rising = portamento_progress(elapsed, 1.0);

    assert!((0.0..=1.0).contains(&falling));
    assert!((0.0..=1.0).contains(&neutral));
    assert!((0.0..=1.0).contains(&rising));
    assert!(falling > neutral);
    assert!(neutral > rising);
}
