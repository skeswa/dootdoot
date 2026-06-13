//! Sequence pooling tests.

use dootdoot_core::{TokenVector, pool_sequence};

#[test]
fn sequence_pooling_uses_token_weighted_mean_without_l2_normalization() {
    let pooled = pool_sequence(&[
        TokenVector::new([1.0, 2.0, 3.0, 4.0], 0.5),
        TokenVector::new([3.0, 4.0, 5.0, 6.0], 1.5),
    ])
    .expect("non-empty sequence should pool");

    assert_eq!(
        pooled.axes().map(f64::to_bits),
        [
            2.5_f64.to_bits(),
            3.5_f64.to_bits(),
            4.5_f64.to_bits(),
            5.5_f64.to_bits(),
        ],
    );

    let squared_norm = pooled.axes().iter().map(|axis| axis * axis).sum::<f64>();
    assert_ne!(squared_norm.to_bits(), 1.0_f64.to_bits());
}

#[test]
fn sequence_pooling_rejects_empty_sequences() {
    let error = pool_sequence(&[]).expect_err("empty sequence should not pool");

    assert!(
        error.to_string().contains("empty token sequence"),
        "unexpected error: {error}",
    );
}
