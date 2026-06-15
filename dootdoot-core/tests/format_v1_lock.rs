//! `VOICE_V1` freeze and `format_v1.bin` artifact contract tests.

use dootdoot_core::{FORMAT_ARTIFACT_V1, FORMAT_VERSION_NUMBER, VOICE_V1, embedded_format_v1};

const FORMAT_DOC: &str = include_str!("../../docs/reference/format_v1.md");

#[test]
fn format_v1_identifier_and_documentation_are_locked() {
    let format = embedded_format_v1().expect("embedded format should parse");

    assert_eq!(VOICE_V1, "VOICE_V1");
    assert_eq!(FORMAT_VERSION_NUMBER, 1);
    assert_eq!(format.artifact_id(), FORMAT_ARTIFACT_V1);

    for expected in [
        "`VOICE_V1` is locked",
        "VOICE_V2",
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
