//! Utterance sample-count estimate tests.

use dootdoot_core::{
    BASE_SYLLABLE_SAMPLES, KnobSet, LEADING_SILENCE_SAMPLES, LONG_PUNCTUATION_PAUSE_SAMPLES,
    SequenceEvent, SquashedVector, TRAILING_SILENCE_SAMPLES, WORD_PAUSE_SAMPLES, assemble_knobs,
    estimate_utterance_sample_count, render_canonical_buffer,
};

#[test]
fn utterance_estimate_matches_rendered_canonical_buffer_length() {
    let knobs = neutral_knobs();
    let events = [
        SequenceEvent::syllable(knobs, false),
        SequenceEvent::punctuation(dootdoot_core::ProsodicPunctuation::Question),
        SequenceEvent::syllable(knobs, false),
    ];
    let expected = u64::from(LEADING_SILENCE_SAMPLES)
        + u64::from(BASE_SYLLABLE_SAMPLES)
        + u64::from(LONG_PUNCTUATION_PAUSE_SAMPLES)
        + u64::from(BASE_SYLLABLE_SAMPLES)
        + u64::from(TRAILING_SILENCE_SAMPLES);

    assert_eq!(estimate_utterance_sample_count(&events), expected);
    assert_eq!(
        usize::try_from(expected).expect("expected sample count fits usize"),
        render_canonical_buffer(&events).len(),
    );
}

#[test]
fn utterance_estimate_includes_word_pause_between_non_continuations() {
    let knobs = neutral_knobs();
    let events = [
        SequenceEvent::syllable(knobs, false),
        SequenceEvent::syllable(knobs, false),
    ];
    let expected = u64::from(LEADING_SILENCE_SAMPLES)
        + u64::from(BASE_SYLLABLE_SAMPLES)
        + u64::from(WORD_PAUSE_SAMPLES)
        + u64::from(BASE_SYLLABLE_SAMPLES)
        + u64::from(TRAILING_SILENCE_SAMPLES);

    assert_eq!(estimate_utterance_sample_count(&events), expected);
}

fn neutral_knobs() -> KnobSet {
    let neutral = SquashedVector::new([0.0, 0.0, 0.0, 0.0]);

    assemble_knobs(neutral, neutral)
}
