//! Golden WAV fixture shape tests.
//!
//! Every golden corpus case must have a committed `.wav` fixture, and each
//! fixture must be a real, non-empty 44.1 kHz / mono / 16-bit PCM WAVE — the
//! format the canonical buffer serializes to — so the committed goldens stay
//! inspectable and playable.

use std::path::PathBuf;

use dootdoot_core::SYNTH_SAMPLE_RATE_HZ;

const GOLDEN_CORPUS: &str = include_str!("fixtures/golden_corpus.tsv");
const WAV_HEADER_BYTES: usize = 44;

#[test]
fn every_corpus_case_has_a_committed_wav_fixture() {
    for label in corpus_labels() {
        let path = fixture_path(label);

        assert!(
            path.is_file(),
            "golden corpus case {label} should have a committed fixture at {}",
            path.display(),
        );
    }
}

#[test]
fn golden_fixtures_are_well_formed_pcm_wave() {
    for label in corpus_labels() {
        let bytes = std::fs::read(fixture_path(label))
            .unwrap_or_else(|error| panic!("golden fixture {label} should be readable: {error}"));

        assert!(
            bytes.len() > WAV_HEADER_BYTES,
            "fixture {label} should carry audio past the header, got {} bytes",
            bytes.len(),
        );
        assert_eq!(
            &bytes[0..4],
            b"RIFF",
            "fixture {label} should start with RIFF"
        );
        assert_eq!(&bytes[8..12], b"WAVE", "fixture {label} should be a WAVE");
        assert_eq!(le_u16(&bytes[22..24]), 1, "fixture {label} should be mono",);
        assert_eq!(
            le_u32(&bytes[24..28]),
            SYNTH_SAMPLE_RATE_HZ,
            "fixture {label} should be 44.1 kHz",
        );
        assert_eq!(
            le_u16(&bytes[34..36]),
            16,
            "fixture {label} should be 16-bit",
        );
    }
}

fn le_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn le_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn fixture_path(label: &str) -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/golden"
    ))
    .join(format!("{label}.wav"))
}

fn corpus_labels() -> Vec<&'static str> {
    GOLDEN_CORPUS
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split_once('\t')
                .expect("golden corpus rows should be tab-separated")
                .0
        })
        .collect()
}
