//! Source model loading tests.

use xtask::load_source_model;

#[test]
fn source_model_loader_extracts_embeddings_and_weights() {
    let tokenizer = br#"{
        "version": "1.0",
        "truncation": null,
        "padding": null,
        "added_tokens": [],
        "normalizer": null,
        "pre_tokenizer": null,
        "post_processor": null,
        "decoder": null,
        "model": {
            "type": "WordLevel",
            "vocab": { "[UNK]": 0, "hello": 1 },
            "unk_token": "[UNK]"
        }
    }"#;
    let config = br#"{"normalize":true}"#;
    let model = safetensors_fixture();

    let source_model =
        load_source_model(tokenizer, &model, config).expect("fixture model should load");

    assert_eq!(source_model.token_count(), 2);
    assert_eq!(source_model.embedding_width(), 3);
    assert_eq!(source_model.embeddings(), &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert_eq!(source_model.weights(), &[0.25, 0.75]);
}

fn safetensors_fixture() -> Vec<u8> {
    let header = r#"{"embeddings":{"dtype":"F32","shape":[2,3],"data_offsets":[0,24]},"weights":{"dtype":"F32","shape":[2],"data_offsets":[24,32]}}"#;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u64::try_from(header.len())
            .expect("fixture header length should fit in u64")
            .to_le_bytes(),
    );
    for value in [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0, 0.25, 0.75] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.splice(8..8, header.bytes());
    bytes
}
