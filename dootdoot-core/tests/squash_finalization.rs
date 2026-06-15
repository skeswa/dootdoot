//! `FORMAT_V1` squash finalization tests.

use dootdoot_core::{FormatSquashFunction, embedded_format_v1};

const SQUASH_DOC: &str = include_str!("../../docs/validation/squash.md");

#[test]
fn format_v1_squash_is_finalized_without_artifact_regeneration() {
    let format = embedded_format_v1().expect("embedded format should parse");

    assert_eq!(format.squash_function(), FormatSquashFunction::TanhZScore);

    for expected in [
        "Finalized for FORMAT_V1",
        "tanh z-score",
        "T-52",
        "No artifact regeneration",
        "was needed",
    ] {
        assert!(
            SQUASH_DOC.contains(expected),
            "squash doc should mention {expected}",
        );
    }
}
