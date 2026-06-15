//! Embedded dootdoot asset spec tests.

use std::num::IntErrorKind;

use dootdoot_core::{
    DOOT_ASSET_AXIS_COUNT, DOOT_ASSET_FILE_V1, DOOT_ASSET_HASH_BYTES, DOOT_ASSET_SPEC_VERSION,
    DOOT_ASSET_TOKEN_RECORD_BYTES, DootAsset, Mapping, Tokenizer, embedded_doot_asset,
};

#[test]
fn embedded_doot_asset_parses_and_exposes_runtime_payloads() {
    let asset = embedded_doot_asset().expect("embedded .doot asset should parse");

    assert_eq!(asset.file_name(), DOOT_ASSET_FILE_V1);
    assert_eq!(asset.spec_version(), DOOT_ASSET_SPEC_VERSION);
    assert_eq!(asset.token_count(), 29_528);
    assert_eq!(asset.axis_scales().len(), DOOT_ASSET_AXIS_COUNT);
    assert_eq!(asset.model_hash().len(), DOOT_ASSET_HASH_BYTES);
    assert_eq!(asset.tokenizer_hash().len(), DOOT_ASSET_HASH_BYTES);
    assert_eq!(
        asset.tokenizer_hash(),
        hex32("e67e803f624fb4d67dea1c730d06e1067e1b14d830e2c2202569e3ef0f70bb50"),
    );
    assert!(asset.tokenizer_json().starts_with(b"{"));
    assert!(
        asset
            .tokenizer_json()
            .windows(7)
            .any(|bytes| bytes == b"\"hello\"")
    );
    assert_eq!(
        asset.record_bytes().len(),
        asset.token_count() * DOOT_ASSET_TOKEN_RECORD_BYTES,
    );
    assert!(asset.pca_hash().iter().any(|byte| *byte != 0));
}

#[test]
fn doot_asset_parser_rejects_invalid_protobuf() {
    let error =
        DootAsset::parse(b"not a protobuf asset").expect_err("invalid protobuf should fail");

    assert!(
        error.to_string().contains("protobuf"),
        "unexpected error: {error}",
    );
}

#[test]
fn tokenizer_and_mapping_load_from_the_same_doot_asset() {
    let asset = embedded_doot_asset().expect("embedded .doot asset should parse");
    let tokenizer = Tokenizer::from_asset(&asset).expect("tokenizer should load from asset");
    let mapping = Mapping::from_asset(asset);
    let output = tokenizer.tokenize("hello").expect("hello should tokenize");
    let vector = mapping
        .lookup(output.tokens()[0].id())
        .expect("hello token should map");

    assert_eq!(output.tokens()[0].id(), 6_598);
    assert!(vector.axes().iter().all(|axis| axis.is_finite()));
    assert!(vector.weight().is_finite());
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
