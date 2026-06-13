//! Committed runtime asset tests.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dootdoot_core::{
    FORMAT_HEADER_BYTES, FORMAT_MAGIC, FORMAT_TOKEN_RECORD_BYTES, FORMAT_VERSION_NUMBER,
};
use sha2::{Digest, Sha256};
use xtask::SourceManifest;

const SOURCE_MANIFEST: &str = include_str!("../../assets/source_manifest.toml");

#[test]
fn committed_runtime_assets_match_manifest_and_format_layout() {
    let root = workspace_root();
    let manifest = SourceManifest::parse(SOURCE_MANIFEST).expect("source manifest should parse");
    let tokenizer = fs::read(root.join("assets/tokenizer.json"))
        .expect("assets/tokenizer.json should be committed");
    let artifact = fs::read(root.join("assets/format_v1.bin"))
        .expect("assets/format_v1.bin should be committed");

    assert_eq!(sha256_hex(&tokenizer), manifest.tokenizer_sha256());
    assert!(
        (250_000..=350_000).contains(&artifact.len()),
        "format artifact should be roughly 300 KB, got {} bytes",
        artifact.len(),
    );
    assert_eq!(&artifact[..FORMAT_MAGIC.len()], FORMAT_MAGIC);
    assert_eq!(
        read_u32(&artifact, 8),
        u32::try_from(FORMAT_HEADER_BYTES).expect("header size should fit u32"),
    );
    assert_eq!(read_u32(&artifact, 12), FORMAT_VERSION_NUMBER);
    assert_eq!(read_u32(&artifact, 20), 4);
    assert_eq!(
        &artifact[112..144],
        hex::decode(manifest.model_sha256())
            .expect("model hash should decode")
            .as_slice(),
    );
    assert_eq!(
        &artifact[144..176],
        hex::decode(manifest.tokenizer_sha256())
            .expect("tokenizer hash should decode")
            .as_slice(),
    );

    let vocab_size = usize::try_from(read_u32(&artifact, 16)).expect("vocab size should fit usize");
    assert_eq!(
        artifact.len(),
        FORMAT_HEADER_BYTES + (vocab_size * FORMAT_TOKEN_RECORD_BYTES),
    );
}

#[test]
fn asset_readme_documents_regeneration() {
    let readme = fs::read_to_string(workspace_root().join("assets/README.md"))
        .expect("asset README should be committed");

    for required in [
        "assets/format_v1.bin",
        "assets/tokenizer.json",
        "target/generated/format_v1.bin",
        "cargo run -p xtask",
    ] {
        assert!(
            readme.contains(required),
            "asset README should mention {required}",
        );
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask should live under the workspace")
        .to_path_buf()
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("u32 range should be in-bounds"),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);

    hex::encode(digest)
}
