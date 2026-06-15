//! `FORMAT_V2` affect analysis tests.

use dootdoot_core::analyze_affect_for_text;

#[test]
fn affect_analysis_pools_positive_and_negative_valence() {
    let positive = analyze_affect_for_text("excellent happy").expect("affect should analyze");
    let negative = analyze_affect_for_text("terrible bad").expect("affect should analyze");

    assert!(positive.mood().valence() > 0.55);
    assert!(negative.mood().valence() < -0.55);
    assert!(
        positive
            .token_scores()
            .iter()
            .any(|score| score.token() == "happy" && score.valence() > 0.0),
    );
}

#[test]
fn affect_analysis_arousal_responds_to_punctuation_case_and_intensifiers() {
    let calm = analyze_affect_for_text("happy day").expect("affect should analyze");
    let excited = analyze_affect_for_text("VERY HAPPY!!!").expect("affect should analyze");
    let dampened = analyze_affect_for_text("barely happy.").expect("affect should analyze");

    assert!(excited.mood().arousal() > calm.mood().arousal() + 0.25);
    assert!(excited.mood().arousal() > dampened.mood().arousal());
}

#[test]
fn affect_analysis_is_deterministic() {
    let first = analyze_affect_for_text("really excellent?!").expect("affect should analyze");
    let second = analyze_affect_for_text("really excellent?!").expect("affect should analyze");

    assert_eq!(first, second);
}
