//! `FORMAT_V2` archetype rendering integration tests.

use dootdoot_core::{
    GestureArchetype, SequenceEvent, render_canonical_buffer, sequence_events_for_text,
};

#[test]
fn text_events_carry_archetypes_for_synthesis() {
    let events = sequence_events_for_text("wonderful!").expect("fixture should analyze");
    let archetype = events
        .iter()
        .find_map(|event| match event {
            SequenceEvent::Archetype(selection) => Some(selection.archetype()),
            SequenceEvent::Mood(_)
            | SequenceEvent::Complexity(_)
            | SequenceEvent::Syllable(_)
            | SequenceEvent::Punctuation(_) => None,
        })
        .expect("text analysis should emit archetype");

    assert_eq!(archetype, GestureArchetype::Yelp);
}

#[test]
fn archetype_events_change_texture_without_changing_semantic_events() {
    let events = sequence_events_for_text("wonderful!").expect("fixture should analyze");
    let without_archetypes = events
        .iter()
        .copied()
        .filter(|event| !matches!(event, SequenceEvent::Archetype(_)))
        .collect::<Vec<_>>();
    let with_archetype = render_canonical_buffer(&events);
    let without_archetype = render_canonical_buffer(&without_archetypes);

    assert_eq!(with_archetype.len(), without_archetype.len());
    assert_ne!(with_archetype, without_archetype);
}
