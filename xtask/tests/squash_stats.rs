//! Squash-statistics tests.

use xtask::{PcaProjection, SourceModel, SquashFunction, compute_squash_stats};

#[test]
fn squash_stats_use_tanh_zscore_over_projected_values() {
    let source_model = SourceModel::from_parts(
        vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0],
        vec![1.0, 1.0, 1.0],
        3,
        2,
    )
    .expect("fixture source model should be valid");
    let projection = PcaProjection::from_parts(vec![1.0, 0.0], vec![1.0], vec![2.0, 0.0], 2, 1)
        .expect("fixture projection should be valid");

    let stats = compute_squash_stats(&source_model, &projection).expect("stats should compute");

    assert_eq!(stats.function(), SquashFunction::TanhZScore);
    assert_eq!(stats.axis_count(), 1);
    assert_close(stats.axes()[0].mean(), 0.0, 0.0);
    assert_close(
        stats.axes()[0].standard_deviation(),
        0.816_496_580_927_726,
        0.000_000_000_001,
    );
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();

    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}; delta was {delta}",
    );
}
