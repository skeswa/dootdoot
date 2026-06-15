//! Dootdoot asset serialization tests.

use dootdoot_core::{DOOT_ASSET_SPEC_VERSION, DOOT_ASSET_TOKEN_RECORD_BYTES, DootAsset};
use xtask::{
    PcaProjection, SourceManifest, SourceModel, compute_squash_stats, dequantize_i16,
    quantize_symmetric_i16, serialize_doot_asset,
};

#[test]
fn quantization_uses_symmetric_half_even_rounding() {
    assert_eq!(quantize_symmetric_i16(0.5, 1.0).expect("quantize"), 0);
    assert_eq!(quantize_symmetric_i16(1.5, 1.0).expect("quantize"), 2);
    assert_eq!(quantize_symmetric_i16(2.5, 1.0).expect("quantize"), 2);
    assert_eq!(quantize_symmetric_i16(-0.5, 1.0).expect("quantize"), 0);
    assert_eq!(quantize_symmetric_i16(-1.5, 1.0).expect("quantize"), -2);
    assert_eq!(quantize_symmetric_i16(-2.5, 1.0).expect("quantize"), -2);
    let dequantized = dequantize_i16(quantize_symmetric_i16(123.0, 1.0).expect("quantize"), 1.0);
    assert_eq!(dequantized.to_bits(), 123.0_f64.to_bits());
}

#[test]
fn serialized_doot_asset_uses_protobuf_asset_spec() {
    let source_model = SourceModel::from_parts(vec![1.0, 2.0, 3.0, 4.0], vec![0.25, 0.75], 2, 2)
        .expect("source model should be valid");
    let projection = PcaProjection::from_parts(
        vec![1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0],
        vec![1.0, 0.5, 0.25, 0.125],
        vec![0.0, 0.0],
        2,
        4,
    )
    .expect("projection should be valid");
    let squash_stats =
        compute_squash_stats(&source_model, &projection).expect("squash stats should compute");
    let manifest = SourceManifest::parse(
        r#"
hf_repo = "fixture/repo"
revision = "fixture"
model_sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
tokenizer_sha256 = "1111111111111111111111111111111111111111111111111111111111111111"
config_sha256 = "2222222222222222222222222222222222222222222222222222222222222222"
hidden_dim = 2
normalize = true
dtype = "F32"
acquisition = "download-to-build-cache"
"#,
    )
    .expect("manifest should parse");

    let bytes = serialize_doot_asset(
        &source_model,
        &projection,
        &squash_stats,
        &manifest,
        br#"{"version":"1.0"}"#,
    )
    .expect("asset should serialize");
    let asset = DootAsset::parse(&bytes).expect("serialized asset should parse");

    assert_eq!(asset.spec_version(), DOOT_ASSET_SPEC_VERSION);
    assert_eq!(asset.token_count(), source_model.token_count());
    assert_eq!(asset.tokenizer_json(), br#"{"version":"1.0"}"#);
    assert_eq!(
        asset.record_bytes().len(),
        source_model.token_count() * DOOT_ASSET_TOKEN_RECORD_BYTES,
    );
}
