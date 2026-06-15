//! Committed runtime asset tests.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dootdoot_core::{DOOT_ASSET_FILE_V1, DOOT_ASSET_TOKEN_RECORD_BYTES, DootAsset};
use sha2::{Digest, Sha256};
use xtask::SourceManifest;

const SOURCE_MANIFEST: &str = include_str!("../../assets/source_manifest.toml");

#[test]
fn committed_runtime_asset_matches_manifest_and_asset_spec() {
    let root = workspace_root();
    let manifest = SourceManifest::parse(SOURCE_MANIFEST).expect("source manifest should parse");
    let asset_bytes = fs::read(root.join("assets").join(DOOT_ASSET_FILE_V1))
        .expect("assets/dootdoot_asset_v1.doot should be committed");
    let asset = DootAsset::parse(&asset_bytes).expect("committed .doot asset should parse");

    assert_eq!(
        sha256_hex(asset.tokenizer_json()),
        manifest.tokenizer_sha256()
    );
    assert_eq!(asset.model_hash(), hex32(manifest.model_sha256()));
    assert_eq!(asset.tokenizer_hash(), hex32(manifest.tokenizer_sha256()));
    assert!(
        (900_000..=1_100_000).contains(&asset_bytes.len()),
        "dootdoot asset should be roughly 1 MB, got {} bytes",
        asset_bytes.len(),
    );
    assert_eq!(
        asset.record_bytes().len(),
        asset.token_count() * DOOT_ASSET_TOKEN_RECORD_BYTES,
    );
    assert!(!root.join("assets/format_v1.bin").exists());
    assert!(!root.join("assets/tokenizer.json").exists());
}

#[test]
fn asset_readme_documents_regeneration() {
    let readme = fs::read_to_string(workspace_root().join("assets/README.md"))
        .expect("asset README should be committed");

    for required in [
        "assets/dootdoot_asset_v1.doot",
        "target/generated/dootdoot_asset_v1.doot",
        "Protocol Buffers",
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

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);

    hex::encode(digest)
}

fn hex32(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64);

    let mut bytes = [0_u8; 32];
    hex::decode_to_slice(hex, &mut bytes).expect("manifest hash should decode");

    bytes
}
