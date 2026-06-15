//! Pitch model tests.

use dootdoot_core::{
    BASE_SYLLABLE_SECONDS, INTERNAL_PITCH_ARCH_CENTS, INTERNAL_PITCH_SWEEP_CENTS,
    PITCH_REGISTER_BIAS_HZ, PITCH_SEMITONE_SPAN, PORTAMENTO_SECONDS, apply_internal_pitch_swoop_hz,
    exp, internal_pitch_offset_cents, pitch_center_hz, portamento_pitch_hz, portamento_progress,
    sin,
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

#[test]
fn internal_pitch_swoop_moves_single_syllables_inside_fixed_range() {
    let start = internal_pitch_offset_cents(-1.0, 0.0);
    let middle = internal_pitch_offset_cents(-1.0, 0.5 * BASE_SYLLABLE_SECONDS);
    let end = internal_pitch_offset_cents(-1.0, BASE_SYLLABLE_SECONDS);

    assert_eq!(start.to_bits(), (-INTERNAL_PITCH_SWEEP_CENTS).to_bits());
    assert_eq!(end.to_bits(), INTERNAL_PITCH_SWEEP_CENTS.to_bits());
    let expected_middle = INTERNAL_PITCH_ARCH_CENTS * sin(core::f64::consts::FRAC_PI_2);

    assert_eq!(middle.to_bits(), expected_middle.to_bits());

    for contour in [-1.0, 0.0, 1.0] {
        for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let elapsed = progress * BASE_SYLLABLE_SECONDS;
            let offset = internal_pitch_offset_cents(contour, elapsed);

            assert!(
                offset.abs() <= INTERNAL_PITCH_SWEEP_CENTS + INTERNAL_PITCH_ARCH_CENTS,
                "pitch offset {offset} escaped the fixed trajectory range",
            );
        }
    }
}

#[test]
fn internal_pitch_swoop_changes_pitch_without_neighbor_portamento() {
    let base = pitch_center_hz(0.0);
    let start = apply_internal_pitch_swoop_hz(base, 0.0, 0.0);
    let middle = apply_internal_pitch_swoop_hz(base, 0.0, 0.5 * BASE_SYLLABLE_SECONDS);
    let end = apply_internal_pitch_swoop_hz(base, 0.0, BASE_SYLLABLE_SECONDS);

    assert_eq!(start.to_bits(), base.to_bits());
    assert!(middle > base);
    assert_eq!(end.to_bits(), base.to_bits());
}
