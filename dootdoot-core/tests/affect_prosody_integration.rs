//! Affect-driven prosody integration tests.

use dootdoot_core::{SequenceEvent, render_text_canonical_buffer, sequence_events_for_text};

#[test]
fn text_events_carry_utterance_mood_for_synthesis() {
    let events = sequence_events_for_text("VERY HAPPY!!!").expect("text should analyze");

    match events.first() {
        Some(SequenceEvent::Mood(mood)) => {
            assert!(mood.valence() > 0.0);
            assert!(mood.arousal() > 0.65);
        }
        other => panic!("first event should be mood, got {other:?}"),
    }
}

#[test]
fn affect_arousal_changes_rendered_rate_for_same_tokens() {
    let calm = render_text_canonical_buffer("happy there!").expect("text should render");
    let shouted = render_text_canonical_buffer("HAPPY THERE!").expect("text should render");

    assert!(shouted.len() < calm.len());
    assert_ne!(shouted, calm);
}
