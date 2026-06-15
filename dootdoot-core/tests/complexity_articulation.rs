//! `FORMAT_V2` complexity articulation integration tests.

use dootdoot_core::{
    SequenceEvent, analyze_complexity_for_text, estimate_utterance_sample_count,
    render_canonical_buffer, sequence_events_for_text,
};

#[test]
fn text_events_carry_complexity_for_synthesis() {
    let expected =
        analyze_complexity_for_text("antidisestablishmentarianism").expect("fixture analyzes");
    let events =
        sequence_events_for_text("antidisestablishmentarianism").expect("fixture analyzes");
    let complexity = events
        .iter()
        .find_map(|event| match event {
            SequenceEvent::Complexity(complexity) => Some(*complexity),
            SequenceEvent::Mood(_) | SequenceEvent::Syllable(_) | SequenceEvent::Punctuation(_) => {
                None
            }
        })
        .expect("text analysis should emit complexity");

    assert_eq!(complexity, expected);
    assert!(complexity.scalar() > 0.70);
}

#[test]
fn complexity_changes_articulation_without_changing_semantic_events() {
    let events =
        sequence_events_for_text("antidisestablishmentarianism").expect("fixture analyzes");
    let without_complexity = events
        .iter()
        .copied()
        .filter(|event| !matches!(event, SequenceEvent::Complexity(_)))
        .collect::<Vec<_>>();
    let with_complexity = render_canonical_buffer(&events);
    let without_complexity_audio = render_canonical_buffer(&without_complexity);

    assert!(with_complexity.len() > without_complexity_audio.len());
    assert_ne!(
        &with_complexity[..without_complexity_audio.len()],
        without_complexity_audio.as_slice(),
    );
    assert_eq!(
        estimate_utterance_sample_count(&events),
        u64::try_from(with_complexity.len()).expect("rendered length fits u64"),
    );
}
