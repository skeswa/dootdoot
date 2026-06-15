//! Double-run determinism tests.

use dootdoot_core::{render_text_canonical_buffer, wav_bytes};

const GOLDEN_CORPUS: &str = include_str!("fixtures/golden_corpus.tsv");

#[test]
fn golden_corpus_renders_identical_buffers_and_wavs_twice() {
    for case in golden_cases() {
        let first_buffer =
            render_text_canonical_buffer(case.text).expect("first render should succeed");
        let second_buffer =
            render_text_canonical_buffer(case.text).expect("second render should succeed");

        assert_eq!(
            first_buffer, second_buffer,
            "canonical buffer changed between runs for {}",
            case.label,
        );

        let first_wav = wav_bytes(&first_buffer).expect("first WAV should serialize");
        let second_wav = wav_bytes(&second_buffer).expect("second WAV should serialize");

        assert_eq!(
            first_wav, second_wav,
            "WAV bytes changed between runs for {}",
            case.label,
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct GoldenCase<'a> {
    label: &'a str,
    text: &'a str,
}

fn golden_cases() -> Vec<GoldenCase<'static>> {
    GOLDEN_CORPUS
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (label, text) = line
                .split_once('\t')
                .expect("golden corpus rows should be tab-separated");

            GoldenCase { label, text }
        })
        .collect()
}
