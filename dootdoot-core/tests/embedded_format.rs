//! Embedded format artifact tests.

use std::num::IntErrorKind;

use dootdoot_core::{
    FORMAT_AXIS_COUNT, FORMAT_HEADER_BYTES, FORMAT_TOKEN_RECORD_BYTES, FORMAT_V1, FormatArtifact,
    embedded_format_v1,
};

#[test]
fn embedded_format_v1_parses_and_exposes_header() {
    let artifact = embedded_format_v1().expect("embedded artifact should parse");

    assert_eq!(artifact.format_id(), FORMAT_V1);
    assert_eq!(artifact.header_byte_len(), FORMAT_HEADER_BYTES);
    assert_eq!(artifact.token_count(), 29_528);
    assert_eq!(artifact.axis_scales().len(), FORMAT_AXIS_COUNT);
    assert_eq!(artifact.record_bytes().len(), 295_280);
    assert_eq!(
        artifact.record_bytes().len(),
        artifact.token_count() * FORMAT_TOKEN_RECORD_BYTES,
    );
    assert!(
        artifact
            .axis_scales()
            .iter()
            .all(|scale| scale.is_finite() && *scale > 0.0)
    );
    assert!(artifact.weight_scale().is_finite());
    assert!(artifact.weight_scale() > 0.0);
    assert_eq!(
        artifact.model_hash(),
        hex32("f65d0f325faadc1e121c319e2faa41170d3fa07d8c89abd48ca5358d9a223de2"),
    );
    assert_eq!(
        artifact.tokenizer_hash(),
        hex32("e67e803f624fb4d67dea1c730d06e1067e1b14d830e2c2202569e3ef0f70bb50"),
    );
    assert!(artifact.pca_hash().iter().any(|byte| *byte != 0));
    assert_eq!(artifact.squash_stats().len(), FORMAT_AXIS_COUNT);
    assert!(artifact.squash_stats().iter().all(|axis| {
        axis.mean().is_finite()
            && axis.standard_deviation().is_finite()
            && axis.standard_deviation() > 0.0
    }));
}

#[test]
fn format_parser_rejects_invalid_magic() {
    let mut bytes = include_bytes!("../../assets/format_v1.bin").to_vec();
    bytes[0] = b'X';

    let error = FormatArtifact::parse(&bytes).expect_err("invalid magic should fail");

    assert!(
        error.to_string().contains("magic"),
        "unexpected error: {error}",
    );
}

fn hex32(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64);

    let mut bytes = [0_u8; 32];

    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        let end = start + 2;
        *byte = u8::from_str_radix(&hex[start..end], 16).unwrap_or_else(|error| {
            assert_eq!(*error.kind(), IntErrorKind::InvalidDigit);
            panic!("invalid hex fixture at byte {index}: {error}");
        });
    }

    bytes
}
