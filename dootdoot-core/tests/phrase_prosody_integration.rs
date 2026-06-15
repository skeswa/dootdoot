//! Phrase-prosody synthesis integration tests.

use dootdoot_core::{
    CLAUSE_SYLLABLE_SAMPLES, LEADING_SILENCE_SAMPLES, LONG_PUNCTUATION_PAUSE_SAMPLES,
    MEDIUM_PUNCTUATION_PAUSE_SAMPLES, SENTENCE_SYLLABLE_SAMPLES, TRAILING_SILENCE_SAMPLES,
    estimate_utterance_sample_count, render_canonical_buffer, sequence_events_for_text,
};

#[test]
fn phrase_prosody_lengthens_clause_and_sentence_boundaries() {
    let events = sequence_events_for_text("hello, there.").expect("text should analyze");
    let expected = u64::from(LEADING_SILENCE_SAMPLES)
        + u64::from(CLAUSE_SYLLABLE_SAMPLES)
        + u64::from(MEDIUM_PUNCTUATION_PAUSE_SAMPLES)
        + u64::from(SENTENCE_SYLLABLE_SAMPLES)
        + u64::from(LONG_PUNCTUATION_PAUSE_SAMPLES)
        + u64::from(TRAILING_SILENCE_SAMPLES);

    assert_eq!(estimate_utterance_sample_count(&events), expected);
    assert_eq!(
        render_canonical_buffer(&events).len(),
        usize::try_from(expected).expect("expected sample count fits usize"),
    );
}

#[test]
fn phrase_prosody_changes_samples_beyond_v1_pause_timing() {
    let phrase_events = sequence_events_for_text("hello there.").expect("text should analyze");
    let plain_events = sequence_events_for_text("hello there").expect("text should analyze");
    let phrase = render_canonical_buffer(&phrase_events);
    let plain = render_canonical_buffer(&plain_events);

    assert!(phrase.len() > plain.len());
    assert_ne!(&phrase[..plain.len()], plain.as_slice());
}
