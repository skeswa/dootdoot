//! PCA projection tests.

use xtask::{SourceModel, compute_pca_projection};

#[test]
fn pca_projection_finds_top_variance_axis() {
    let source_model = SourceModel::from_parts(
        vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0],
        vec![1.0, 1.0, 1.0],
        3,
        2,
    )
    .expect("fixture source model should be valid");

    let projection = compute_pca_projection(&source_model, 1).expect("fixture PCA should compute");

    assert_eq!(projection.source_width(), 2);
    assert_eq!(projection.axis_count(), 1);
    assert_close(projection.means()[0], 2.0, 0.0);
    assert_close(projection.means()[1], 0.0, 0.0);
    assert_close(projection.eigenvalues()[0], 1.0, 0.000_000_001);
    assert_close(projection.components()[0].abs(), 1.0, 0.000_000_001);
    assert_close(projection.components()[1].abs(), 0.0, 0.000_000_001);
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();

    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}; delta was {delta}",
    );
}
