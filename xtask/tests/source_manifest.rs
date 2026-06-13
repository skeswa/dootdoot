//! Source manifest validation tests.

use sha2::{Digest, Sha256};
use xtask::{SourceFiles, SourceManifest};

const SOURCE_MANIFEST: &str = include_str!("../../assets/source_manifest.toml");

#[test]
fn source_manifest_pins_potion_base_8m() {
    let manifest = SourceManifest::parse(SOURCE_MANIFEST).expect("source manifest should parse");

    assert_eq!(manifest.hf_repo(), "minishlab/potion-base-8M");
    assert_eq!(
        manifest.revision(),
        "bf8b056651a2c21b8d2565580b8569da283cab23",
    );
    assert_eq!(
        manifest.model_sha256(),
        "f65d0f325faadc1e121c319e2faa41170d3fa07d8c89abd48ca5358d9a223de2",
    );
    assert_eq!(
        manifest.tokenizer_sha256(),
        "e67e803f624fb4d67dea1c730d06e1067e1b14d830e2c2202569e3ef0f70bb50",
    );
    assert_eq!(
        manifest.config_sha256(),
        "2a6ac0e9aaa356a68a5688070db78fc3a464fefe85d2f06a1905ce3718687553",
    );
    assert_eq!(manifest.hidden_dim(), 256);
    assert!(manifest.normalize());
    assert_eq!(manifest.dtype(), "F32");
    assert_eq!(manifest.acquisition(), "download-to-build-cache");
}

#[test]
fn source_validation_rejects_mismatched_hashes() {
    let model = safetensors_fixture("F32");
    let tokenizer = br#"{"tokenizer":"fixture"}"#;
    let config = br#"{"hidden_dim":256,"normalize":true}"#;
    let manifest = manifest_for_fixture(&model, tokenizer, config);

    manifest
        .validate_sources(SourceFiles::new(
            "fixture-revision",
            &model,
            tokenizer,
            config,
        ))
        .expect("matching fixture sources should validate");

    let error = manifest
        .validate_sources(SourceFiles::new(
            "fixture-revision",
            b"different-model",
            tokenizer,
            config,
        ))
        .expect_err("changed model bytes should be rejected");

    assert!(
        error
            .to_string()
            .contains("model.safetensors sha256 mismatch"),
        "unexpected validation error: {error}",
    );
}

fn manifest_for_fixture(model: &[u8], tokenizer: &[u8], config: &[u8]) -> SourceManifest {
    SourceManifest::parse(&format!(
        r#"
hf_repo = "fixture/repo"
revision = "fixture-revision"
model_sha256 = "{}"
tokenizer_sha256 = "{}"
config_sha256 = "{}"
hidden_dim = 256
normalize = true
dtype = "F32"
acquisition = "download-to-build-cache"
"#,
        sha256_hex(model),
        sha256_hex(tokenizer),
        sha256_hex(config),
    ))
    .expect("fixture manifest should parse")
}

fn safetensors_fixture(dtype: &str) -> Vec<u8> {
    let header =
        format!(r#"{{"embedding":{{"dtype":"{dtype}","shape":[1,256],"data_offsets":[0,4]}}}}"#);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u64::try_from(header.len())
            .expect("fixture header length should fit in u64")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(header.as_bytes());
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);

    hex::encode(digest)
}
