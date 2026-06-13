//! Formant filter bank tests.

use dootdoot_core::{
    FORMANT_AH_HZ, FORMANT_EE_HZ, FORMANT_OO_HZ, FormantFilterBank, formant_frequencies,
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
            500.0_f64.to_bits(),
            1_690.0_f64.to_bits(),
            2_725.0_f64.to_bits(),
        ],
    );
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
