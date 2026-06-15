//! Golden corpus fixture tests.

use std::collections::BTreeSet;

use dootdoot_core::render_text_canonical_buffer;

const GOLDEN_CORPUS: &str = include_str!("fixtures/golden_corpus.tsv");

#[test]
fn golden_corpus_covers_voice_v1_contract_cases() {
    let cases = golden_cases();
    let labels = cases.iter().map(|case| case.label).collect::<BTreeSet<_>>();

    for required in [
        "empty",
        "hello",
        "hello_there",
        "playing",
        "cat",
        "dog",
        "airplane",
        "bare_question",
        "punctuation",
        "unknown",
        "long",
    ] {
        assert!(
            labels.contains(required),
            "golden corpus should include {required}",
        );
    }

    assert_eq!(labels.len(), cases.len(), "golden labels should be unique");
}

#[test]
fn golden_corpus_inputs_render_nonempty_buffers() {
    for case in golden_cases() {
        let buffer =
            render_text_canonical_buffer(case.text).expect("golden corpus case should render");

        assert!(
            !buffer.is_empty(),
            "golden corpus case {} should render audio",
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
