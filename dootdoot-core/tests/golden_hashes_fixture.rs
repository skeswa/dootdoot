//! Golden WAV hash fixture shape tests.

use std::collections::BTreeSet;

const GOLDEN_CORPUS: &str = include_str!("fixtures/golden_corpus.tsv");
const GOLDEN_HASHES: &str = include_str!("fixtures/golden_wav_hashes.tsv");

#[test]
fn golden_hash_fixture_covers_every_corpus_case() {
    let corpus_labels = labels_from_tsv(GOLDEN_CORPUS);
    let hash_labels = labels_from_tsv(GOLDEN_HASHES);

    assert_eq!(hash_labels, corpus_labels);
}

#[test]
fn golden_hash_fixture_contains_sha256_hex_values() {
    for line in data_lines(GOLDEN_HASHES) {
        let (label, hash) = line
            .split_once('\t')
            .expect("golden hash rows should be tab-separated");

        assert_eq!(hash.len(), 64, "hash for {label} should be 64 hex chars");
        assert!(
            hash.chars().all(|character| character.is_ascii_hexdigit()),
            "hash for {label} should be hex",
        );
    }
}

fn labels_from_tsv(tsv: &str) -> BTreeSet<&str> {
    data_lines(tsv)
        .map(|line| {
            line.split_once('\t')
                .expect("fixture rows should be tab-separated")
                .0
        })
        .collect()
}

fn data_lines(tsv: &str) -> impl Iterator<Item = &str> {
    tsv.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}
