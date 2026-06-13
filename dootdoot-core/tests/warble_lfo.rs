//! Warble LFO tests.

use dootdoot_core::{
    WARBLE_DEPTH_CENTS, WARBLE_RATE_HZ, apply_warble_hz, exp, sin, warble_depth_cents,
    warble_offset_cents,
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
fn warble_offset_uses_fixed_rate_lfo() {
    let elapsed = 1.0 / (4.0 * WARBLE_RATE_HZ);
    let expected = WARBLE_DEPTH_CENTS * sin(core::f64::consts::FRAC_PI_2);

    assert_eq!(
        warble_offset_cents(1.0, elapsed).to_bits(),
        expected.to_bits()
    );
    assert_eq!(
        warble_offset_cents(-1.0, elapsed).to_bits(),
        0.0_f64.to_bits()
    );
}

#[test]
fn warble_applies_cent_offset_to_pitch() {
    let pitch = 880.0;
    let elapsed = 1.0 / (4.0 * WARBLE_RATE_HZ);
    let cents = warble_offset_cents(1.0, elapsed);
    let expected = pitch * exp(core::f64::consts::LN_2 * (cents / 1_200.0));

    assert_eq!(
        apply_warble_hz(pitch, 1.0, elapsed).to_bits(),
        expected.to_bits()
    );
}
