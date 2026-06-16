//! Learnability spread validation tests.

use dootdoot_core::{ExplainRow, explain_rows_for_text, render_text_canonical_buffer};

const LEARNABILITY: &str = include_str!("../../docs/validation/learnability-spread.md");

#[test]
fn final_semantic_and_audio_spread_keep_close_words_closer() {
    let cat_dog_knobs = knob_distance("cat", "dog");
    let cat_airplane_knobs = knob_distance("cat", "airplane");
    let cat_dog_audio = audio_distance("cat", "dog");
    let cat_airplane_audio = audio_distance("cat", "airplane");

    assert!(cat_dog_knobs < cat_airplane_knobs);
    assert!(cat_dog_audio < cat_airplane_audio);
}

#[test]
fn learnability_validation_note_records_final_cluster_spread() {
    for expected in [
        "Finalized for VOICE_V1",
        "semantic clusters",
        "cat",
        "dog",
        "airplane",
        "audibly distinct",
        "audibly similar",
    ] {
        assert!(
            LEARNABILITY.contains(expected),
            "learnability note should mention {expected}",
        );
    }
}

fn knob_distance(left: &str, right: &str) -> f64 {
    let left = first_knobs(left);
    let right = first_knobs(right);

    left.into_iter()
        .zip(right)
        .map(|(left, right)| {
            let delta = left - right;

            delta * delta
        })
        .sum()
}

fn first_knobs(text: &str) -> [f64; 4] {
    explain_rows_for_text(text)
        .expect("fixture text should render explain rows")
        .into_iter()
        .find_map(|row| match row {
            ExplainRow::Token(token) => Some(token.knobs().axes()),
            ExplainRow::Mood(_)
            | ExplainRow::Complexity(_)
            | ExplainRow::Punctuation(_)
            | ExplainRow::Hesitation(_) => None,
        })
        .expect("fixture text should have a voiced token")
}

fn audio_distance(left: &str, right: &str) -> f64 {
    let left = render_text_canonical_buffer(left).expect("left fixture should render");
    let right = render_text_canonical_buffer(right).expect("right fixture should render");
    let shared_len = left.len().min(right.len());
    let shared_delta = left
        .iter()
        .zip(&right)
        .take(shared_len)
        .map(|(left, right)| f64::from((i32::from(*left) - i32::from(*right)).abs()))
        .sum::<f64>();
    let length_delta = left.len().abs_diff(right.len());
    let denominator = u32::try_from(shared_len.max(1)).expect("fixture length should fit u32");

    (shared_delta / f64::from(denominator))
        + f64::from(u32::try_from(length_delta).expect("fixture length delta should fit u32"))
}
