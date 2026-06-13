//! Public facade tests for dootdoot-core.

use dootdoot_core::{FORMAT_V1, Format, Mapping, Mathx, Synth, Tokenizer, WavWriter};

#[test]
fn public_api_exports_core_stubs() {
    assert_eq!(FORMAT_V1, "FORMAT_V1");

    let stubs = [
        format!("{Format:?}"),
        format!("{Mapping:?}"),
        format!("{Mathx:?}"),
        format!("{Synth:?}"),
        format!("{Tokenizer:?}"),
        format!("{WavWriter:?}"),
    ];

    assert_eq!(
        stubs,
        [
            "Format",
            "Mapping",
            "Mathx",
            "Synth",
            "Tokenizer",
            "WavWriter"
        ],
    );
}
