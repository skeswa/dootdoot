//! Knob assembly tests.

use dootdoot_core::{
    KNOB_BOUNDS, KNOB_MODULATION_DEPTHS, SquashedVector, assemble_knob_sequence, assemble_knobs,
};

#[test]
fn single_token_knobs_equal_the_squashed_baseline() {
    let baseline = SquashedVector::new([0.125, -0.25, 0.5, -0.75]);
    let knobs = assemble_knobs(baseline, baseline);

    assert_eq!(
        knobs.axes().map(f64::to_bits),
        baseline.axes().map(f64::to_bits)
    );
    assert_eq!(knobs.pitch_center().to_bits(), 0.125_f64.to_bits());
    assert_eq!(knobs.vowel_position().to_bits(), (-0.25_f64).to_bits());
    assert_eq!(knobs.contour().to_bits(), 0.5_f64.to_bits());
    assert_eq!(knobs.warble_depth().to_bits(), (-0.75_f64).to_bits());
}

#[test]
fn knob_assembly_clamps_exaggerated_modulation_to_bounds() {
    let baseline = SquashedVector::new([-1.0; 4]);
    let token = SquashedVector::new([1.0; 4]);
    let knobs = assemble_knobs(baseline, token);
    let expected = std::array::from_fn(|index| {
        let bounds = KNOB_BOUNDS[index];
        (-1.0 + (KNOB_MODULATION_DEPTHS[index] * 2.0)).clamp(bounds.lower(), bounds.upper())
    });

    assert_eq!(knobs.axes().map(f64::to_bits), expected.map(f64::to_bits));
    assert_eq!(knobs.contour().to_bits(), 1.0_f64.to_bits());
    assert_eq!(knobs.warble_depth().to_bits(), 1.0_f64.to_bits());
}

#[test]
fn knob_sequence_assembles_one_row_per_syllable() {
    let baseline = SquashedVector::new([0.0; 4]);
    let rows = assemble_knob_sequence(
        baseline,
        &[
            SquashedVector::new([0.25, 0.0, 0.0, 0.0]),
            SquashedVector::new([0.0, 0.25, 0.0, 0.0]),
        ],
    );

    assert_eq!(rows.len(), 2);
    assert_ne!(
        rows[0].pitch_center().to_bits(),
        rows[1].pitch_center().to_bits()
    );
    assert_ne!(
        rows[0].vowel_position().to_bits(),
        rows[1].vowel_position().to_bits()
    );
}
