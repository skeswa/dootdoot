//! `VOICE_V7` planner deployment integration tests.

use dootdoot_core::{
    GestureArchetype, ROLE_LONG_PAUSE_MIN_SAMPLES, SequenceEvent, render_text_canonical_buffer,
    sequence_events_for_text,
};

const REFERENCE: &str = "Hello - good morning Sandile. What are you doing today?!";

fn max_zero_run(samples: &[i16]) -> usize {
    let mut best = 0;
    let mut run = 0;

    for &sample in samples {
        if sample == 0 {
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
    }

    best
}

fn archetypes(text: &str) -> Vec<GestureArchetype> {
    sequence_events_for_text(text)
        .expect("text should sequence")
        .iter()
        .filter_map(|event| match event {
            SequenceEvent::Archetype(selection) => Some(selection.archetype()),
            _ => None,
        })
        .collect()
}

#[test]
fn staged_utterance_opens_a_real_turn_gap() {
    let buffer = render_text_canonical_buffer(REFERENCE).expect("reference should render");

    assert!(
        max_zero_run(&buffer) >= ROLE_LONG_PAUSE_MIN_SAMPLES as usize,
        "a staged discourse utterance should open a real turn gap",
    );
}

#[test]
fn plain_statement_keeps_the_smooth_connected_path() {
    let buffer = render_text_canonical_buffer("I am so excited I am so excited")
        .expect("statement should render");

    assert!(
        max_zero_run(&buffer) < ROLE_LONG_PAUSE_MIN_SAMPLES as usize,
        "a plain statement must not be torn apart by long staged rests",
    );
}

#[test]
fn reference_archetypes_are_localized_not_one_yelp() {
    let archetypes = archetypes(REFERENCE);

    assert!(
        archetypes.len() >= 3,
        "reference should have many syllables"
    );

    let first = archetypes[0];
    let varies = archetypes.iter().any(|archetype| *archetype != first);

    assert!(
        varies,
        "archetype should vary by role, not collapse to a single global Yelp: {archetypes:?}",
    );
}

#[test]
fn deployment_is_deterministic() {
    assert_eq!(
        render_text_canonical_buffer(REFERENCE).expect("first render"),
        render_text_canonical_buffer(REFERENCE).expect("second render"),
    );
}
