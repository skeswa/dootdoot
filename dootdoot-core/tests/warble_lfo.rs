//! Warble LFO tests.

use dootdoot_core::{
    WARBLE_DEPTH_CENTS, apply_warble_hz, apply_warble_hz_with_phase, compound_warble_offset_cents,
    exp, warble_depth_cents, warble_offset_cents, warble_phase_offset_for_syllable,
};

#[test]
fn warble_depth_maps_knob_to_positive_depth() {
    assert_eq!(warble_depth_cents(-1.0).to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        warble_depth_cents(0.0).to_bits(),
        (WARBLE_DEPTH_CENTS * 0.5).to_bits(),
    );
    assert_eq!(
        warble_depth_cents(1.0).to_bits(),
        WARBLE_DEPTH_CENTS.to_bits()
    );
}

#[test]
fn compound_warble_is_bounded_and_phase_offset() {
    let elapsed = 0.037;
    let zero_phase = compound_warble_offset_cents(1.0, elapsed, 0.0);
    let shifted_phase = compound_warble_offset_cents(1.0, elapsed, 0.37);
    let medium = compound_warble_offset_cents(0.0, elapsed, 0.37);

    assert_ne!(zero_phase.to_bits(), shifted_phase.to_bits());
    assert!(shifted_phase.abs() <= WARBLE_DEPTH_CENTS);
    assert!(medium.abs() < shifted_phase.abs());
    assert_eq!(
        compound_warble_offset_cents(-1.0, elapsed, 0.37).to_bits(),
        0.0_f64.to_bits(),
    );
}

#[test]
fn warble_phase_offsets_are_deterministic_per_syllable() {
    let first = warble_phase_offset_for_syllable(0);
    let second = warble_phase_offset_for_syllable(1);

    assert_eq!(
        first.to_bits(),
        warble_phase_offset_for_syllable(0).to_bits()
    );
    assert_ne!(first.to_bits(), second.to_bits());
    assert!((0.0..1.0).contains(&first));
    assert!((0.0..1.0).contains(&second));
}

#[test]
fn warble_applies_compound_cent_offset_to_pitch() {
    let pitch = 880.0;
    let elapsed = 0.037;
    let phase = 0.37;
    let cents = compound_warble_offset_cents(1.0, elapsed, phase);
    let expected = pitch * exp(core::f64::consts::LN_2 * (cents / 1_200.0));

    assert_eq!(
        warble_offset_cents(1.0, elapsed).to_bits(),
        compound_warble_offset_cents(1.0, elapsed, 0.0).to_bits(),
    );
    assert_eq!(
        apply_warble_hz_with_phase(pitch, 1.0, elapsed, phase).to_bits(),
        expected.to_bits()
    );
    assert_eq!(
        apply_warble_hz(pitch, 1.0, elapsed).to_bits(),
        apply_warble_hz_with_phase(pitch, 1.0, elapsed, 0.0).to_bits(),
    );
}
