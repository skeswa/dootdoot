//! Semantic ordering sanity tests.

use dootdoot_core::{PooledVector, embedded_mapping, embedded_tokenizer, pool_sequence};

#[test]
fn semantically_related_tokens_are_closer_than_unrelated_tokens() {
    let mapping = embedded_mapping().expect("mapping should load");
    let cat = mapping.lookup(3_943).expect("cat should map");
    let dog = mapping.lookup(2_905).expect("dog should map");
    let airplane = mapping.lookup(12_303).expect("airplane should map");

    assert_ordering(
        squared_distance(cat.axes(), dog.axes()),
        squared_distance(cat.axes(), airplane.axes()),
    );
}

#[test]
fn semantically_related_sequences_are_closer_than_unrelated_sequences() {
    let cat = pooled_sequence("a small cat");
    let dog = pooled_sequence("a small dog");
    let airplane = pooled_sequence("a small airplane");

    assert_ordering(
        squared_distance(cat.axes(), dog.axes()),
        squared_distance(cat.axes(), airplane.axes()),
    );
}

fn pooled_sequence(text: &str) -> PooledVector {
    let wrapper = embedded_tokenizer().expect("tokenizer should load");
    let mapping = embedded_mapping().expect("mapping should load");
    let output = wrapper.tokenize(text).expect("text should tokenize");
    let vectors = output
        .tokens()
        .iter()
        .map(|token| mapping.lookup(token.id()).expect("token should map"))
        .collect::<Vec<_>>();

    pool_sequence(&vectors).expect("sequence should pool")
}

fn squared_distance(left: [f64; 4], right: [f64; 4]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left_axis, right_axis)| {
            let delta = left_axis - right_axis;

            delta * delta
        })
        .sum()
}

fn assert_ordering(related_distance: f64, unrelated_distance: f64) {
    assert!(
        related_distance < unrelated_distance,
        "expected related distance {related_distance} to be below unrelated distance {unrelated_distance}",
    );
}
