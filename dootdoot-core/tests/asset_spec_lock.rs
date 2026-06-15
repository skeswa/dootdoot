//! `VOICE_V1` freeze and dootdoot asset spec contract tests.

use dootdoot_core::{DOOT_ASSET_FILE_V1, DOOT_ASSET_SPEC_VERSION, VOICE_V1, embedded_doot_asset};

const ASSET_SPEC_DOC: &str = include_str!("../../docs/reference/dootdoot_asset_spec.md");

#[test]
fn doot_asset_identifier_and_documentation_are_locked() {
    let asset = embedded_doot_asset().expect("embedded asset should parse");

    assert_eq!(VOICE_V1, "VOICE_V1");
    assert_eq!(DOOT_ASSET_SPEC_VERSION, 1);
    assert_eq!(asset.file_name(), DOOT_ASSET_FILE_V1);

    for expected in [
        "`VOICE_V1` is locked",
        "VOICE_V2",
        "voice tuning accepted",
        "squash finalized",
        "learnability spread validated",
    ] {
        assert!(
            ASSET_SPEC_DOC.contains(expected),
            "asset spec doc should mention {expected}",
        );
    }
}
