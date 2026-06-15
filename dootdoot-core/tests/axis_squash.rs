//! Axis squash tests.

use dootdoot_core::{
    DootAsset, TokenVector, embedded_doot_asset, embedded_mapping, pool_sequence, tanh,
};

#[test]
fn mapping_squashes_token_axes_with_frozen_tanh_zscore_stats() {
    let asset = embedded_doot_asset().expect("asset should parse");
    let mapping = embedded_mapping().expect("mapping should load");
    let token = TokenVector::new(fixture_axes(&asset), 1.0);
    let squashed = mapping.squash_token(token);

    assert_eq!(squashed.axes().map(f64::to_bits), expected_bits());
}

#[test]
fn mapping_squashes_pooled_baseline_axes_with_same_stats() {
    let asset = embedded_doot_asset().expect("asset should parse");
    let mapping = embedded_mapping().expect("mapping should load");
    let pooled = pool_sequence(&[TokenVector::new(fixture_axes(&asset), 1.0)])
        .expect("single token should pool");
    let squashed = mapping.squash_pooled(pooled);

    assert_eq!(squashed.axes().map(f64::to_bits), expected_bits());
}

fn fixture_axes(asset: &DootAsset) -> [f64; 4] {
    let stats = asset.squash_stats();

    [
        stats[0].mean(),
        stats[1].mean() + stats[1].standard_deviation(),
        stats[2].mean() - stats[2].standard_deviation(),
        stats[3].mean() + (2.0 * stats[3].standard_deviation()),
    ]
}

fn expected_bits() -> [u64; 4] {
    [0.0, 1.0, -1.0, 2.0].map(tanh).map(f64::to_bits)
}
