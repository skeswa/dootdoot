//! Dootdoot asset spec layout tests.

use dootdoot_core::{
    DOOT_ASSET_AXIS_COUNT, DOOT_ASSET_HASH_BYTES, DOOT_ASSET_SCALE_COUNT, DOOT_ASSET_SPEC_VERSION,
    DOOT_ASSET_SQUASH_STATS_PER_AXIS, DOOT_ASSET_TOKEN_RECORD_BYTES,
};

const ASSET_SPEC: &str = include_str!("../../docs/reference/dootdoot_asset_spec.md");

#[test]
fn dootdoot_asset_spec_layout_constants_are_pinned() {
    assert_eq!(DOOT_ASSET_SPEC_VERSION, 1);
    assert_eq!(DOOT_ASSET_AXIS_COUNT, 4);
    assert_eq!(DOOT_ASSET_SCALE_COUNT, 5);
    assert_eq!(DOOT_ASSET_HASH_BYTES, 32);
    assert_eq!(DOOT_ASSET_SQUASH_STATS_PER_AXIS, 2);
    assert_eq!(DOOT_ASSET_TOKEN_RECORD_BYTES, 10);

    assert!(ASSET_SPEC.contains("Protocol Buffers"));
    assert!(ASSET_SPEC.contains("token_records"));
    assert!(ASSET_SPEC.contains("Per-token record"));
}
