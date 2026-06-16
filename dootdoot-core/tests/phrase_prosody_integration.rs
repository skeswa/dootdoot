//! Phrase-prosody synthesis integration tests.

use dootdoot_core::{
    ExplainRow, KnobSet, LONG_PUNCTUATION_PAUSE_SAMPLES, MEDIUM_PUNCTUATION_PAUSE_SAMPLES,
    ProsodicPunctuation, SequenceEvent, UtteranceMood, estimate_utterance_sample_count,
    explain_rows_for_text, render_canonical_buffer, sequence_events_for_text,
};

#[test]
fn phrase_prosody_lengthens_clause_and_sentence_boundaries() {
    let plain_events = phrase_test_events(None);
    let clause_events = phrase_test_events(Some(ProsodicPunctuation::Comma));
    let sentence_events = phrase_test_events(Some(ProsodicPunctuation::Period));
    let plain_samples = estimate_utterance_sample_count(&plain_events);
    let clause_samples = estimate_utterance_sample_count(&clause_events);
    let sentence_samples = estimate_utterance_sample_count(&sentence_events);

    assert!(clause_samples > plain_samples + u64::from(MEDIUM_PUNCTUATION_PAUSE_SAMPLES));
    assert!(sentence_samples > plain_samples + u64::from(LONG_PUNCTUATION_PAUSE_SAMPLES));
    assert_eq!(
        render_canonical_buffer(&clause_events).len(),
        usize::try_from(clause_samples).expect("expected sample count fits usize"),
    );
    assert_eq!(
        render_canonical_buffer(&sentence_events).len(),
        usize::try_from(sentence_samples).expect("expected sample count fits usize"),
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

fn phrase_test_events(punctuation: Option<ProsodicPunctuation>) -> Vec<SequenceEvent> {
    let knobs = phrase_test_knobs();
    let mut events = vec![
        SequenceEvent::mood(UtteranceMood::new(0.0, 0.0)),
        SequenceEvent::syllable(knobs, false),
    ];

    if let Some(punctuation) = punctuation {
        events.push(SequenceEvent::punctuation(punctuation));
    }

    events
}

fn phrase_test_knobs() -> KnobSet {
    explain_rows_for_text("hello")
        .expect("fixture text should analyze")
        .into_iter()
        .find_map(|row| match row {
            ExplainRow::Mood(_)
            | ExplainRow::Complexity(_)
            | ExplainRow::Punctuation(_)
            | ExplainRow::Hesitation(_) => None,
            ExplainRow::Token(token) => Some(token.knobs()),
        })
        .expect("fixture text should have a token row")
}
