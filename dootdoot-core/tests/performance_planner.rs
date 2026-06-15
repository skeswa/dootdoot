//! `VOICE_V7` discourse-performance planner tests.

use dootdoot_core::{
    PerformanceCurves, PhraseRole, plan_discourse_performance, sequence_events_for_text,
};

fn roles(text: &str) -> Vec<PhraseRole> {
    let events = sequence_events_for_text(text).expect("text should sequence");

    plan_discourse_performance(&events)
        .syllables()
        .iter()
        .map(|syllable| syllable.role())
        .collect()
}

fn curves_in_bounds(curves: PerformanceCurves) -> bool {
    (-1.0..=1.0).contains(&curves.pitch_center_bias())
        && (-1.0..=1.0).contains(&curves.pitch_velocity())
        && (-1.0..=1.0).contains(&curves.formant_target())
        && (-1.0..=1.0).contains(&curves.formant_velocity())
        && (0.0..=1.0).contains(&curves.brightness_pressure())
        && (0.0..=1.0).contains(&curves.mouth_openness())
        && (0.0..=1.0).contains(&curves.archetype_tension())
}

#[test]
fn reference_phrase_stages_opener_reply_and_flourish() {
    let roles = roles("Hello - good morning Sandile. What are you doing today?!");

    assert_eq!(
        roles.first(),
        Some(&PhraseRole::Hesitation),
        "the opener before the dash should read as a hesitation",
    );
    assert_eq!(
        roles.last(),
        Some(&PhraseRole::TerminalFlourish),
        "the final ?! should read as a terminal flourish",
    );
    assert!(
        roles.contains(&PhraseRole::ChattyReply),
        "the middle should answer with chatty reply",
    );
    assert!(
        !roles.contains(&PhraseRole::TerminalFlourish)
            || roles
                .iter()
                .filter(|role| **role == PhraseRole::TerminalFlourish)
                .count()
                <= "What are you doing today".split_whitespace().count() + 2,
        "the flourish should not engulf the whole utterance",
    );
}

#[test]
fn plain_statement_is_a_chatty_reply() {
    assert!(
        roles("hello there")
            .iter()
            .all(|role| *role == PhraseRole::ChattyReply)
    );
}

#[test]
fn leading_phrase_before_a_reset_is_a_probe() {
    let roles = roles("hi there. okay then.");

    assert_eq!(roles.first(), Some(&PhraseRole::Probe));
}

#[test]
fn comma_segment_is_an_aside() {
    let roles = roles("well, fine");

    assert_eq!(
        roles.first(),
        Some(&PhraseRole::Aside),
        "a comma-delimited lead segment should read as an aside",
    );
}

#[test]
fn planner_is_deterministic() {
    let text = "What are you doing today?!";
    let first = sequence_events_for_text(text).expect("sequence");
    let second = sequence_events_for_text(text).expect("sequence");

    assert_eq!(
        plan_discourse_performance(&first),
        plan_discourse_performance(&second),
    );
}

#[test]
fn every_syllable_has_bounded_curves() {
    for text in [
        "hello there cat dog",
        "Hello - good morning Sandile. What are you doing today?!",
        "wow!!! that is amazing, truly",
        "?",
    ] {
        let events = sequence_events_for_text(text).expect("sequence");

        for syllable in plan_discourse_performance(&events).syllables() {
            assert!(
                curves_in_bounds(syllable.curves()),
                "curves out of bounds for {text}",
            );
        }
    }
}

#[test]
fn plan_row_count_matches_voiced_syllables() {
    let events = sequence_events_for_text("playing airplane").expect("sequence");
    let voiced = events
        .iter()
        .filter(|event| matches!(event, dootdoot_core::SequenceEvent::Syllable(_)))
        .count();

    assert_eq!(
        plan_discourse_performance(&events).syllables().len(),
        voiced
    );
}
