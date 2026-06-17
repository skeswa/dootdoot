//! Utterance sample-count estimate tests.

use dootdoot_core::{
    BASE_SYLLABLE_SAMPLES, KnobSet, LEADING_SILENCE_SAMPLES, LONG_PUNCTUATION_PAUSE_SAMPLES,
    SENTENCE_SYLLABLE_SAMPLES, SequenceEvent, SquashedVector, TRAILING_SILENCE_SAMPLES,
    WORD_PAUSE_SAMPLES, assemble_knobs, estimate_utterance_sample_count, render_canonical_buffer,
    text_syllable_duration_scale,
};

#[test]
fn neutral_text_syllables_are_shorter_than_the_base() {
    // VOICE_V10: calm (low-arousal) text should pace shorter than the un-scaled
    // base so neutral gestures read as quick blips rather than long held tones.
    // The hand-built / empty-chirp path keeps a scale of exactly 1.0 and is
    // unaffected (it never carries an explicit mood).
    let calm = text_syllable_duration_scale(0.10);

    assert!(
        calm < 1.0,
        "calm text should pace shorter than the base, got scale {calm}",
    );
    assert!(
        text_syllable_duration_scale(1.0) < calm,
        "higher arousal should pace faster than calm text",
    );
    for arousal in [0.0, 0.25, 0.5, 0.75, 1.0, f64::NAN, 2.0, -1.0] {
        let scale = text_syllable_duration_scale(arousal);

        assert!(
            (0.7..=1.0).contains(&scale),
            "duration scale must stay bounded, got {scale} for {arousal}",
        );
    }
}

#[test]
fn utterance_estimate_matches_rendered_canonical_buffer_length() {
    let knobs = neutral_knobs();
    let events = [
        SequenceEvent::syllable(knobs, false),
        SequenceEvent::punctuation(dootdoot_core::ProsodicPunctuation::Question),
        SequenceEvent::syllable(knobs, false),
    ];
    let expected = u64::from(LEADING_SILENCE_SAMPLES)
        + u64::from(SENTENCE_SYLLABLE_SAMPLES)
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
