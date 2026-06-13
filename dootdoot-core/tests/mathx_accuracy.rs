//! Owned-math accuracy tests.

use core::f64::consts::{FRAC_PI_2, PI, TAU};

use dootdoot_core::{cos, exp, sin, tanh};

#[test]
fn trig_matches_std_within_audio_tolerance() {
    for sample in [
        -4.0 * TAU,
        -3.0 * PI,
        -PI,
        -FRAC_PI_2,
        -1.0,
        -0.25,
        0.0,
        0.25,
        1.0,
        FRAC_PI_2,
        PI,
        3.0 * PI,
        4.0 * TAU,
    ] {
        assert_close(sin(sample), sample.sin(), 0.001);
        assert_close(cos(sample), sample.cos(), 0.001);
    }
}

#[test]
fn exp_and_tanh_match_std_within_audio_tolerance() {
    for sample in [-10.0, -4.0, -1.0, -0.125, 0.0, 0.125, 1.0, 4.0, 10.0] {
        assert_relative_close(exp(sample), sample.exp(), 0.000_1);
    }

    for sample in [-4.0, -1.0, -0.125, 0.0, 0.125, 1.0, 4.0] {
        assert_close(tanh(sample), sample.tanh(), 0.000_001);
    }
}

#[test]
fn mathx_boundaries_stay_finite() {
    for sample in [-1000.0 * TAU, -TAU, 0.0, TAU, 1000.0 * TAU] {
        assert!(sin(sample).is_finite());
        assert!(cos(sample).is_finite());
    }

    assert!(exp(709.0).is_finite());
    assert!(exp(-745.0).is_finite());
    assert!(tanh(20.0).is_finite());
    assert!(tanh(-20.0).is_finite());
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();

    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}; delta was {delta}",
    );
}

fn assert_relative_close(actual: f64, expected: f64, tolerance: f64) {
    let scale = expected.abs().max(1.0);
    let relative_delta = (actual - expected).abs() / scale;

    assert!(
        relative_delta <= tolerance,
        "expected {actual} to be within relative tolerance {tolerance} of {expected}; relative delta was {relative_delta}",
    );
}
