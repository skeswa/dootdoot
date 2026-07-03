//! Pins the committed tagged-counts snapshot to the source manifest (T-120).
//!
//! The `[pos]` manifest section and `assets/pos/tagged_counts.tsv` must agree
//! byte-for-byte, and the locked derivation policy must produce a sane class
//! table from the real snapshot — including the canonical policy outcomes on
//! coding vocabulary.

use std::{fs, path::Path};

use xtask::{
    PosPolicyConfig, PosSourceManifest, PosTableClass, derive_pos_class_table, parse_tagged_counts,
};

fn workspace_path(relative: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root exists")
        .join(relative)
        .display()
        .to_string()
}

#[test]
fn committed_snapshot_matches_its_manifest_pin() {
    let manifest_text =
        fs::read_to_string(workspace_path("assets/source_manifest.toml")).expect("manifest reads");
    let manifest = PosSourceManifest::parse(&manifest_text).expect("pos section parses");
    let snapshot =
        fs::read(workspace_path("assets/pos/tagged_counts.tsv")).expect("snapshot reads");

    manifest
        .validate_tagged_counts(&snapshot)
        .expect("committed snapshot hashes to its manifest pin");
}

#[test]
fn committed_snapshot_derives_the_expected_policy_outcomes() {
    let snapshot_text =
        fs::read_to_string(workspace_path("assets/pos/tagged_counts.tsv")).expect("snapshot reads");
    let snapshot = parse_tagged_counts(&snapshot_text).expect("committed snapshot parses");
    let table = derive_pos_class_table(&snapshot, &PosPolicyConfig::default());
    let class_of = |surface: &str| {
        table
            .iter()
            .find(|entry| entry.surface() == surface)
            .map(xtask::PosClassEntry::pos_class)
    };

    assert!(table.len() > 1_000, "table has {} entries", table.len());

    // Canonical coding vocabulary lands where the policy says it should.
    assert_eq!(class_of("bug"), Some(PosTableClass::Noun));
    assert_eq!(class_of("error"), Some(PosTableClass::Noun));

    // Surface expansion: an inflected form rides its lemma.
    assert_eq!(class_of("bugs"), Some(PosTableClass::Noun));

    // Conservative ambiguity fallback: `build` (42% minority use) stays out.
    assert_eq!(class_of("build"), None);

    // Closed-class words never enter the table.
    assert_eq!(class_of("the"), None);
    assert_eq!(class_of("can"), None);
}
