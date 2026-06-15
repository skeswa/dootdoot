//! `VOICE_V2` complexity scalar tests.

use dootdoot_core::analyze_complexity_for_text;

#[test]
fn complexity_scalar_keeps_common_short_words_simple() {
    let cat = analyze_complexity_for_text("cat").expect("fixture should analyze");
    let dog = analyze_complexity_for_text("dog").expect("fixture should analyze");

    assert_eq!(cat.wordpiece_subtoken_count(), 0);
    assert_eq!(dog.wordpiece_subtoken_count(), 0);
    assert!(cat.scalar() < 0.10);
    assert!(dog.scalar() < 0.10);
}

#[test]
fn complexity_scalar_rises_for_longer_wordpiece_shapes() {
    let simple = analyze_complexity_for_text("cat").expect("simple fixture should analyze");
    let complex = analyze_complexity_for_text("antidisestablishmentarianism")
        .expect("complex fixture should analyze");

    assert!(complex.character_count() > simple.character_count());
    assert!(complex.wordpiece_subtoken_count() > simple.wordpiece_subtoken_count());
    assert!(complex.scalar() > simple.scalar());
}

#[test]
fn complexity_analysis_is_deterministic() {
    let first =
        analyze_complexity_for_text("hyperdrive recalibration").expect("fixture should analyze");
    let second =
        analyze_complexity_for_text("hyperdrive recalibration").expect("fixture should analyze");

    assert_eq!(first, second);
}
