//! Single-syllable renderer tests.

use dootdoot_core::{
    BASE_SYLLABLE_SAMPLES, SquashedVector, assemble_knobs, pitch_center_hz, render_syllable,
};

#[test]
fn renderer_returns_fixed_finite_non_silent_syllable() {
    let baseline = SquashedVector::new([0.0, 0.0, 0.0, 0.0]);
    let token = SquashedVector::new([0.25, -0.40, 0.20, 0.50]);
    let knobs = assemble_knobs(baseline, token);
    let samples = render_syllable(knobs, pitch_center_hz(0.0));

    assert_eq!(
        samples.len(),
        usize::try_from(BASE_SYLLABLE_SAMPLES).expect("base syllable sample count fits usize"),
    );
    assert!(samples.iter().all(|sample| sample.is_finite()));
    assert!(samples.iter().any(|sample| sample.abs() > 0.000_001));
    assert_eq!(samples[0].to_bits(), 0.0_f64.to_bits());
}

#[test]
fn renderer_is_bit_exact_for_repeated_inputs() {
    let baseline = SquashedVector::new([0.10, -0.10, 0.20, -0.20]);
    let token = SquashedVector::new([-0.35, 0.45, -0.25, 0.75]);
    let knobs = assemble_knobs(baseline, token);
    let start_pitch_hz = pitch_center_hz(0.65);
    let first = render_syllable(knobs, start_pitch_hz);
    let second = render_syllable(knobs, start_pitch_hz);
    let first_bits: Vec<u64> = first.iter().map(|sample| sample.to_bits()).collect();
    let second_bits: Vec<u64> = second.iter().map(|sample| sample.to_bits()).collect();

    assert_eq!(first_bits, second_bits);
}

#[test]
fn renderer_uses_start_pitch_for_portamento() {
    let baseline = SquashedVector::new([0.0, 0.0, 0.0, 0.0]);
    let token = SquashedVector::new([0.15, 0.10, 0.75, 0.0]);
    let knobs = assemble_knobs(baseline, token);
    let from_low = render_syllable(knobs, pitch_center_hz(-1.0));
    let from_high = render_syllable(knobs, pitch_center_hz(1.0));
    let low_bits: Vec<u64> = from_low.iter().map(|sample| sample.to_bits()).collect();
    let high_bits: Vec<u64> = from_high.iter().map(|sample| sample.to_bits()).collect();

    assert_ne!(low_bits, high_bits);
}
