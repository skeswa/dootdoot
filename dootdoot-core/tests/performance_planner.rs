//! `VOICE_V7` discourse-performance planner tests.

use dootdoot_core::{
    PerformanceCurves, PerformanceSyllable, PhraseRole, plan_discourse_performance,
    sequence_events_for_text,
};

fn roles(text: &str) -> Vec<PhraseRole> {
    let events = sequence_events_for_text(text).expect("text should sequence");

    plan_discourse_performance(&events)
        .syllables()
        .iter()
        .map(PerformanceSyllable::role)
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
fn neutral_phrase_engages_a_semantic_accent() {
    // VOICE_V8: a punctuation-less phrase still earns one expressive accent,
    // driven by semantics rather than punctuation. Roles stay ChattyReply, but
    // one syllable's curves climb into whistle/roughness-engaging territory.
    let events = sequence_events_for_text("cat dog airplane bird").expect("sequence");
    let plan = plan_discourse_performance(&events);
    let tensions = plan
        .syllables()
        .iter()
        .map(|syllable| syllable.curves().archetype_tension())
        .collect::<Vec<_>>();

    assert!(
        tensions.iter().copied().fold(0.0_f64, f64::max) > 0.70,
        "a neutral phrase should engage at least one high-tension accent, got {tensions:?}",
    );
}

#[test]
fn long_final_phrase_reserves_the_flourish_for_its_tail() {
    // Regression: a trailing "!" on a long closing phrase made every syllable
    // whistle (shrill). The flourish is a terminal accent, so only the tail of a
    // long final segment flourishes; the lead-in stays a chatty body, and the
    // flourish curves are never engagement-amplified.
    let plan = {
        let events = sequence_events_for_text("the weather is great!").expect("sequence");
        plan_discourse_performance(&events)
    };
    let roles = plan
        .syllables()
        .iter()
        .map(PerformanceSyllable::role)
        .collect::<Vec<_>>();
    let flourish_count = roles
        .iter()
        .filter(|role| **role == PhraseRole::TerminalFlourish)
        .count();

    assert!(
        roles.len() > flourish_count,
        "a long final phrase should keep a chatty lead-in, got all flourish: {roles:?}",
    );
    assert!(
        flourish_count <= 2,
        "only the tail should flourish, got {flourish_count} flourish syllables",
    );
    assert_eq!(
        roles.last(),
        Some(&PhraseRole::TerminalFlourish),
        "the final syllable should still be the terminal accent",
    );

    // The closing flourish syllable keeps its pure role tension (0.95 at the end
    // of the ramp), not an engagement-inflated value.
    let last = plan
        .syllables()
        .last()
        .expect("plan has syllables")
        .curves();

    assert!(
        (last.archetype_tension() - 0.95).abs() < 1e-9,
        "final flourish tension should be the role value 0.95 (engagement-free), got {}",
        last.archetype_tension(),
    );
}

#[test]
fn semantic_engagement_varies_curves_across_a_neutral_phrase() {
    // VOICE_V8: brightness is no longer a flat constant across a neutral phrase;
    // it varies syllable to syllable so upper-mid energy can burst on accents.
    let events = sequence_events_for_text("cat dog airplane bird").expect("sequence");
    let plan = plan_discourse_performance(&events);
    let brights = plan
        .syllables()
        .iter()
        .map(|syllable| syllable.curves().brightness_pressure())
        .collect::<Vec<_>>();

    assert!(
        brights
            .windows(2)
            .any(|window| (window[0] - window[1]).abs() > 1e-9),
        "brightness should vary across a neutral phrase, got {brights:?}",
    );
}

#[test]
fn exclamation_flourish_descends_and_question_flourish_rises() {
    // VOICE_V10: the terminal flourish whistle is bidirectional — an exclamation
    // lands with a descending whistle (negative pitch velocity) while a question
    // rises (positive). Direction is carried by the pitch_velocity sign.
    let flourish_velocity = |text: &str| {
        let events = sequence_events_for_text(text).expect("sequence");
        let plan = plan_discourse_performance(&events);
        plan.syllables()
            .iter()
            .rev()
            .find(|syllable| syllable.role() == PhraseRole::TerminalFlourish)
            .expect("text should end in a terminal flourish")
            .curves()
            .pitch_velocity()
    };

    assert!(
        flourish_velocity("that is amazing!") < 0.0,
        "an exclamation flourish should descend (negative pitch velocity)",
    );
    assert!(
        flourish_velocity("what are you doing?") > 0.0,
        "a question flourish should rise (positive pitch velocity)",
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
