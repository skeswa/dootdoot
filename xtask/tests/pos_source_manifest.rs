//! Tests for the pinned `[pos]` source-manifest section (T-120, FR-114).

use xtask::PosSourceManifest;

const MANIFEST: &str = r#"
hf_repo = "minishlab/potion-base-8M"

[pos]
corpus_hf_repo = "JetBrains-Research/commit-chronicle"
corpus_revision = "5fd076e67b812a9f3d1999e5e40f71715f84bb51"
corpus_file = "data/test-00000-of-00012-2085aa4b49c438e4.parquet"
corpus_sha256 = "b05b4ab34973c358d18475173e301ba0534cb0f23a15f91ba07b3af3fbf0d988"
tagger = "spacy/en_core_web_sm"
tagger_version = "3.7.1"
tagged_counts_sha256 = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
derivation = "uv run scripts/derive_pos_table.py"
"#;

#[test]
fn parses_the_pos_section() {
    let manifest = PosSourceManifest::parse(MANIFEST).expect("pos section parses");

    assert_eq!(
        manifest.corpus_hf_repo(),
        "JetBrains-Research/commit-chronicle"
    );
    assert_eq!(
        manifest.corpus_revision(),
        "5fd076e67b812a9f3d1999e5e40f71715f84bb51"
    );
    assert_eq!(manifest.tagger(), "spacy/en_core_web_sm");
    assert_eq!(manifest.tagger_version(), "3.7.1");
}

#[test]
fn missing_pos_section_is_an_error() {
    assert!(PosSourceManifest::parse("hf_repo = \"x\"").is_err());
}

#[test]
fn validates_the_tagged_counts_snapshot_hash() {
    let manifest = PosSourceManifest::parse(MANIFEST).expect("pos section parses");

    // The pinned hash above is sha256("test").
    assert!(manifest.validate_tagged_counts(b"test").is_ok());
    assert!(manifest.validate_tagged_counts(b"tampered").is_err());
}
