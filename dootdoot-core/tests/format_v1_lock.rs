//! `FORMAT_V1` freeze contract tests.

use dootdoot_core::{FORMAT_V1, FORMAT_VERSION_NUMBER, embedded_format_v1};

const FORMAT_DOC: &str = include_str!("../../docs/format_v1.md");

#[test]
fn format_v1_identifier_and_documentation_are_locked() {
    let format = embedded_format_v1().expect("embedded format should parse");

    assert_eq!(FORMAT_V1, "FORMAT_V1");
    assert_eq!(FORMAT_VERSION_NUMBER, 1);
    assert_eq!(format.format_id(), FORMAT_V1);

    for expected in [
        "`FORMAT_V1` is locked",
        "FORMAT_V2",
        "voice tuning accepted",
        "squash finalized",
        "learnability spread validated",
    ] {
        assert!(
            FORMAT_DOC.contains(expected),
            "format doc should mention {expected}",
        );
    }
}
