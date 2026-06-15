//! Math contract tests.

use dootdoot_core::{
    EXP_POLYNOMIAL_DEGREE, EXP_TABLE_BITS, EXP_TABLE_LEN, MATHX_VERSION, SIN_COS_POLYNOMIAL_DEGREE,
    SIN_COS_TABLE_BITS, SIN_COS_TABLE_LEN, TANH_EXP_CLAMP,
};

const MATHX_RATIONALE: &str = include_str!("../../docs/reference/mathx.md");

#[test]
fn mathx_contract_pins_table_sizes_and_polynomial_degrees() {
    assert_eq!(MATHX_VERSION, "mathx-v1");
    assert_eq!(SIN_COS_TABLE_BITS, 12);
    assert_eq!(SIN_COS_TABLE_LEN, 4096);
    assert_eq!(SIN_COS_POLYNOMIAL_DEGREE, 7);
    assert_eq!(EXP_TABLE_BITS, 10);
    assert_eq!(EXP_TABLE_LEN, 1024);
    assert_eq!(EXP_POLYNOMIAL_DEGREE, 5);
    assert_eq!(TANH_EXP_CLAMP.to_bits(), 20.0_f64.to_bits());

    assert!(MATHX_RATIONALE.contains("No libm transcendentals"));
    assert!(MATHX_RATIONALE.contains("round-half-to-even"));
}
