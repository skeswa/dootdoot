//! Trigonometric owned-math tests.

use core::f64::consts::{FRAC_PI_2, PI, TAU};

use dootdoot_core::{cos, sin};

#[test]
fn sin_and_cos_match_known_angles() {
    assert_close(sin(0.0), 0.0, 0.0);
    assert_close(cos(0.0), 1.0, 0.0);
    assert_close(sin(FRAC_PI_2), 1.0, 0.000_2);
    assert_close(cos(PI), -1.0, 0.0);
    assert_close(sin(TAU), 0.0, 0.000_000_000_001);
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();

    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}; delta was {delta}",
    );
}
