//! Ring modulation and envelope tests.

use dootdoot_core::{
    BASE_SYLLABLE_SECONDS, ENVELOPE_ATTACK_SECONDS, ENVELOPE_DECAY_SECONDS,
    ENVELOPE_RELEASE_SECONDS, ENVELOPE_SUSTAIN_LEVEL, RING_MOD_FREQUENCY_HZ, RING_MOD_MIX,
    amplitude_envelope, apply_amplitude_envelope, ring_modulate, sin,
};

#[test]
fn ring_modulation_uses_fixed_frequency_and_mix() {
    let input = 0.5;
    let quarter_cycle = 1.0 / (4.0 * RING_MOD_FREQUENCY_HZ);
    let expected_at_zero = input * (1.0 - RING_MOD_MIX);
    let expected_at_quarter =
        input * ((1.0 - RING_MOD_MIX) + (RING_MOD_MIX * sin(core::f64::consts::FRAC_PI_2)));

    assert_eq!(
        ring_modulate(input, 0.0).to_bits(),
        expected_at_zero.to_bits()
    );
    assert_eq!(
        ring_modulate(input, quarter_cycle).to_bits(),
        expected_at_quarter.to_bits(),
    );
}

#[test]
fn amplitude_envelope_has_droid_gesture_pulse_dip_and_tail() {
    assert_eq!(
        amplitude_envelope(0.0, BASE_SYLLABLE_SECONDS).to_bits(),
        0.0_f64.to_bits(),
    );
    assert_eq!(
        amplitude_envelope(ENVELOPE_ATTACK_SECONDS, BASE_SYLLABLE_SECONDS).to_bits(),
        1.0_f64.to_bits(),
    );

    let pulse = amplitude_envelope(ENVELOPE_ATTACK_SECONDS + 0.012, BASE_SYLLABLE_SECONDS);
    let dip = amplitude_envelope(
        ENVELOPE_ATTACK_SECONDS + ENVELOPE_DECAY_SECONDS,
        BASE_SYLLABLE_SECONDS,
    );
    let recovery = amplitude_envelope(
        ENVELOPE_ATTACK_SECONDS + ENVELOPE_DECAY_SECONDS + 0.026,
        BASE_SYLLABLE_SECONDS,
    );
    let tail = amplitude_envelope(
        BASE_SYLLABLE_SECONDS - (ENVELOPE_RELEASE_SECONDS * 0.5),
        BASE_SYLLABLE_SECONDS,
    );

    assert!(pulse > recovery);
    assert!(dip < recovery);
    assert!(tail > 0.0);
    assert!(tail < ENVELOPE_SUSTAIN_LEVEL);
    assert_eq!(
        amplitude_envelope(BASE_SYLLABLE_SECONDS, BASE_SYLLABLE_SECONDS).to_bits(),
        0.0_f64.to_bits(),
    );
}

#[test]
fn amplitude_envelope_scales_samples() {
    let sample = 0.25_f64;
    let enveloped =
        apply_amplitude_envelope(sample, ENVELOPE_ATTACK_SECONDS, BASE_SYLLABLE_SECONDS);

    assert_eq!(enveloped.to_bits(), sample.to_bits());
}
