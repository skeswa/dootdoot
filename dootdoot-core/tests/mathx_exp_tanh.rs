//! Exponential owned-math tests.

use core::f64::consts::E;

use dootdoot_core::{exp, tanh};

#[test]
fn exp_and_tanh_match_known_values() {
    assert_close(exp(0.0), 1.0, 0.0);
    assert_close(exp(1.0), E, 0.000_000_1);
    assert_close(exp(-1.0), 1.0 / E, 0.000_000_1);

    assert_close(tanh(0.0), 0.0, 0.0);
    assert_close(tanh(1.0), 0.761_594_155_955_764_9, 0.000_001);
    assert_close(tanh(-1.0), -0.761_594_155_955_764_9, 0.000_001);
    assert_close(tanh(20.0), 1.0, 0.0);
    assert_close(tanh(-20.0), -1.0, 0.0);
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();

    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}; delta was {delta}",
    );
}
