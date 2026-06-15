//! Golden WAV hash contract tests.

use std::collections::BTreeMap;

use dootdoot_core::{render_text_canonical_buffer, wav_bytes};
use sha2::{Digest, Sha256};

const GOLDEN_CORPUS: &str = include_str!("fixtures/golden_corpus.tsv");
const GOLDEN_HASHES: &str = include_str!("fixtures/golden_wav_hashes.tsv");

#[test]
fn golden_corpus_wav_hashes_match_committed_fixture() {
    let hashes = golden_hashes();

    for case in golden_cases() {
        let buffer =
            render_text_canonical_buffer(case.text).expect("golden corpus case should render");
        let bytes = wav_bytes(&buffer).expect("golden corpus WAV should serialize");
        let actual = sha256_hex(&bytes);
        let expected = hashes
            .get(case.label)
            .expect("golden corpus label should have a committed hash");

        assert_eq!(
            &actual, expected,
            "golden WAV hash changed for {}",
            case.label
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct GoldenCase<'a> {
    label: &'a str,
    text: &'a str,
}

fn golden_cases() -> Vec<GoldenCase<'static>> {
    data_lines(GOLDEN_CORPUS)
        .map(|line| {
            let (label, text) = line
                .split_once('\t')
                .expect("golden corpus rows should be tab-separated");

            GoldenCase { label, text }
        })
        .collect()
}

fn golden_hashes() -> BTreeMap<&'static str, &'static str> {
    data_lines(GOLDEN_HASHES)
        .map(|line| {
            line.split_once('\t')
                .expect("golden hash rows should be tab-separated")
        })
        .collect()
}

fn data_lines(tsv: &'static str) -> impl Iterator<Item = &'static str> {
    tsv.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);

    for byte in digest {
        output.push(nibble_to_hex(byte >> 4));
        output.push(nibble_to_hex(byte & 0x0f));
    }

    output
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + (nibble - 10)),
        _ => '?',
    }
}
