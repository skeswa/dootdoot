//! Formant filter bank tests.

use dootdoot_core::{
    BASE_SYLLABLE_SECONDS, FORMANT_AH_HZ, FORMANT_EE_HZ, FORMANT_OO_HZ, FormantFilterBank,
    VOWEL_TRAJECTORY_BLOOM, VOWEL_TRAJECTORY_SWEEP, formant_frequencies, sin,
    vowel_trajectory_position,
};

#[test]
fn formant_frequencies_interpolate_between_vowel_loci() {
    assert_eq!(
        formant_frequencies(-1.0).map(f64::to_bits),
        FORMANT_EE_HZ.map(f64::to_bits)
    );
    assert_eq!(
        formant_frequencies(0.0).map(f64::to_bits),
        FORMANT_AH_HZ.map(f64::to_bits)
    );
    assert_eq!(
        formant_frequencies(1.0).map(f64::to_bits),
        FORMANT_OO_HZ.map(f64::to_bits)
    );
    assert_eq!(
        formant_frequencies(-0.5).map(f64::to_bits),
        [
            460.0_f64.to_bits(),
            1_820.0_f64.to_bits(),
            2_980.0_f64.to_bits(),
        ],
    );
}

#[test]
fn vowel_trajectory_moves_around_semantic_target_inside_bounds() {
    let start = vowel_trajectory_position(0.0, 1.0, 0.0);
    let middle = vowel_trajectory_position(0.0, 1.0, 0.5 * BASE_SYLLABLE_SECONDS);
    let end = vowel_trajectory_position(0.0, 1.0, BASE_SYLLABLE_SECONDS);

    assert_eq!(start.to_bits(), (-VOWEL_TRAJECTORY_SWEEP).to_bits());
    assert_eq!(end.to_bits(), VOWEL_TRAJECTORY_SWEEP.to_bits());
    let expected_middle = VOWEL_TRAJECTORY_BLOOM * sin(core::f64::consts::FRAC_PI_2);

    assert_eq!(middle.to_bits(), expected_middle.to_bits());

    for target in [-1.0, -0.5, 0.0, 0.5, 1.0] {
        for contour in [-1.0, 0.0, 1.0] {
            for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let position =
                    vowel_trajectory_position(target, contour, progress * BASE_SYLLABLE_SECONDS);

                assert!(
                    (-1.0..=1.0).contains(&position),
                    "vowel position {position} escaped the fixed droid range",
                );
            }
        }
    }
}

#[test]
fn formant_filter_bank_outputs_finite_impulse_response() {
    let mut bank = FormantFilterBank::new();
    let mut nonzero_seen = false;

    for index in 0..128 {
        let input = if index == 0 { 1.0 } else { 0.0 };
        let output = bank.process_sample(input, 0.0);

        assert!(output.is_finite());
        assert!(
            output.abs() <= 2.0,
            "formant output {output} exceeded test bound"
        );
        nonzero_seen |= output.to_bits() != 0.0_f64.to_bits();
    }

    assert!(nonzero_seen);
}

#[test]
fn formant_filter_bank_reset_restores_deterministic_state() {
    let mut bank = FormantFilterBank::new();
    let first = impulse_response(&mut bank);

    bank.reset();
    let second = impulse_response(&mut bank);

    assert_eq!(first, second);
}

fn impulse_response(bank: &mut FormantFilterBank) -> Vec<u64> {
    (0..32)
        .map(|index| {
            let input = if index == 0 { 1.0 } else { 0.0 };

            bank.process_sample(input, -0.25).to_bits()
        })
        .collect()
}
