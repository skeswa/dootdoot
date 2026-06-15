//! `VOICE_V1` squash finalization tests.

use dootdoot_core::{DootAssetSquashFunction, embedded_doot_asset};

const SQUASH_DOC: &str = include_str!("../../docs/validation/squash.md");

#[test]
fn doot_asset_squash_is_finalized_without_asset_regeneration() {
    let asset = embedded_doot_asset().expect("embedded asset should parse");

    assert_eq!(asset.squash_function(), DootAssetSquashFunction::TanhZScore);

    for expected in [
        "Finalized for VOICE_V1",
        "tanh z-score",
        "T-52",
        "No asset regeneration",
        "remain the VOICE_V1 contract",
    ] {
        assert!(
            SQUASH_DOC.contains(expected),
            "squash doc should mention {expected}",
        );
    }
}
