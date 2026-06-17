//! Golden WAV file contract tests.
//!
//! Each golden corpus case renders to a committed `.wav` fixture under
//! `tests/fixtures/golden/`. The test re-renders every case and compares the
//! bytes to its committed fixture byte-for-byte, so the fixtures are the
//! human-auditable, playable form of the determinism contract — a regression
//! can be listened to, not just diffed as an opaque hash.
//!
//! Regenerate the fixtures after an intentional, version-bumped voice change:
//!
//! ```bash
//! DOOTDOOT_REGEN_GOLDEN=1 cargo test -p dootdoot-core --test golden_wav
//! ```

use std::path::PathBuf;

use dootdoot_core::{render_text_canonical_buffer, wav_bytes};

const GOLDEN_CORPUS: &str = include_str!("fixtures/golden_corpus.tsv");

#[test]
fn golden_corpus_wav_files_match_committed_fixtures() {
    if std::env::var_os("DOOTDOOT_REGEN_GOLDEN").is_some() {
        regenerate_golden_wav_fixtures();
        return;
    }

    for case in golden_cases() {
        let actual = render_case_wav(&case);
        let path = fixture_path(case.label);
        let expected = std::fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "missing golden WAV fixture {} ({error}); regenerate with \
                 DOOTDOOT_REGEN_GOLDEN=1",
                path.display(),
            )
        });

        match first_difference(&actual, &expected) {
            None => {}
            Some(offset) => panic!(
                "golden WAV changed for {} at byte {offset} (rendered {} bytes, committed {}); \
                 regenerate with DOOTDOOT_REGEN_GOLDEN=1 if the voice version changed",
                case.label,
                actual.len(),
                expected.len(),
            ),
        }
    }
}

fn regenerate_golden_wav_fixtures() {
    let dir = golden_dir();
    std::fs::create_dir_all(&dir).expect("golden fixture directory should be creatable");

    for case in golden_cases() {
        let bytes = render_case_wav(&case);

        std::fs::write(dir.join(format!("{}.wav", case.label)), bytes)
            .expect("golden WAV fixture should be writable");
    }
}

fn render_case_wav(case: &GoldenCase<'_>) -> Vec<u8> {
    let buffer = render_text_canonical_buffer(case.text).expect("golden corpus case should render");

    wav_bytes(&buffer).expect("golden corpus WAV should serialize")
}

/// Returns the index of the first differing byte, or `None` when equal.
fn first_difference(actual: &[u8], expected: &[u8]) -> Option<usize> {
    actual
        .iter()
        .zip(expected)
        .position(|(left, right)| left != right)
        .or_else(|| (actual.len() != expected.len()).then_some(actual.len().min(expected.len())))
}

fn golden_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/golden"
    ))
}

fn fixture_path(label: &str) -> PathBuf {
    golden_dir().join(format!("{label}.wav"))
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
