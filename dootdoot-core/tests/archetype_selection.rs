//! `FORMAT_V2` gesture-archetype selection tests.

use dootdoot_core::{GestureArchetype, plan_gesture_archetypes, sequence_events_for_text};

#[test]
fn archetype_selection_is_deterministic_for_same_events() {
    let events = sequence_events_for_text("VERY happy!!!").expect("fixture should analyze");
    let first = plan_gesture_archetypes(&events);
    let second = plan_gesture_archetypes(&events);

    assert_eq!(first, second);
}

#[test]
fn archetype_palette_responds_to_affect_complexity_and_punctuation() {
    let yelp = first_archetype("wonderful!");
    let moan = first_archetype("sad");
    let tremble = first_archetype("terrible danger!!!");
    let stutter_burst = first_archetype("antidisestablishmentarianism");

    assert_eq!(yelp, GestureArchetype::Yelp);
    assert_eq!(moan, GestureArchetype::Moan);
    assert_eq!(tremble, GestureArchetype::Tremble);
    assert_eq!(stutter_burst, GestureArchetype::StutterBurst);
}

#[test]
fn sparse_seasoning_depends_on_phrase_position_without_randomness() {
    let events = sequence_events_for_text(
        "antidisestablishmentarianism antidisestablishmentarianism antidisestablishmentarianism",
    )
    .expect("fixture should analyze");
    let selections = plan_gesture_archetypes(&events);

    assert!(selections[0].servo_seasoning());
    assert!(!selections[1].servo_seasoning());
    assert!(selections[2].servo_seasoning());
}

fn first_archetype(text: &str) -> GestureArchetype {
    let events = sequence_events_for_text(text).expect("fixture should analyze");
    let selections = plan_gesture_archetypes(&events);

    selections
        .first()
        .expect("fixture should have a voiced selection")
        .archetype()
}
