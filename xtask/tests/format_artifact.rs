//! Format artifact serialization tests.

use dootdoot_core::{FORMAT_HEADER_BYTES, FORMAT_MAGIC, FORMAT_TOKEN_RECORD_BYTES};
use xtask::{
    PcaProjection, SourceManifest, SourceModel, compute_squash_stats, dequantize_i16,
    quantize_symmetric_i16, serialize_format_artifact,
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
fn serialized_format_artifact_uses_pinned_layout() {
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

    let bytes = serialize_format_artifact(&source_model, &projection, &squash_stats, &manifest)
        .expect("artifact should serialize");

    assert_eq!(&bytes[..FORMAT_MAGIC.len()], FORMAT_MAGIC);
    assert_eq!(
        bytes.len(),
        FORMAT_HEADER_BYTES + (source_model.token_count() * FORMAT_TOKEN_RECORD_BYTES),
    );
}
