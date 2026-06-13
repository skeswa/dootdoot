//! PCA sign canonicalization tests.

use xtask::PcaProjection;

#[test]
fn pca_sign_canonicalization_makes_largest_loading_positive() {
    let mut projection = PcaProjection::from_parts(
        vec![-0.2, -0.9, 0.1, 0.3, -0.1, -0.8],
        vec![4.0, 2.0],
        vec![0.0, 0.0, 0.0],
        3,
        2,
    )
    .expect("fixture projection should be valid");

    projection.canonicalize_component_signs();
    let once = bits(projection.components());
    projection.canonicalize_component_signs();

    assert_eq!(bits(projection.components()), once);
    assert_eq!(
        bits(projection.components()),
        bits(&[0.2, 0.9, -0.1, -0.3, 0.1, 0.8]),
    );
}

fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}
