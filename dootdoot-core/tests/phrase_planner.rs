//! Phrase-prosody planner tests.

use dootdoot_core::{
    LONG_PUNCTUATION_PAUSE_SAMPLES, PhraseBoundaryStrength, WORD_PAUSE_SAMPLES,
    plan_phrase_prosody, sequence_events_for_text,
};

#[test]
fn phrase_planner_sets_boundary_metadata_from_real_text() {
    let events = sequence_events_for_text("hello there?").expect("text should analyze");
    let plan = plan_phrase_prosody(&events);
    let syllables = plan.syllables();

    assert_eq!(syllables.len(), 2);
    assert_eq!(
        syllables[0].boundary_strength(),
        PhraseBoundaryStrength::Word,
    );
    assert_eq!(syllables[0].pause_samples(), WORD_PAUSE_SAMPLES);
    assert_float_bits(syllables[0].pre_boundary_lengthening(), 1.0);
    assert!(syllables[0].is_emphasized());

    assert_eq!(
        syllables[1].boundary_strength(),
        PhraseBoundaryStrength::Sentence,
    );
    assert_eq!(syllables[1].pause_samples(), LONG_PUNCTUATION_PAUSE_SAMPLES);
    assert_float_bits(syllables[1].pitch_reset_semitones(), 1.2);
    assert_float_bits(syllables[1].pre_boundary_lengthening(), 1.25);
}

#[test]
fn phrase_planner_resets_declination_after_sentence_boundary() {
    let events = sequence_events_for_text("hello. there friend").expect("text should analyze");
    let plan = plan_phrase_prosody(&events);
    let syllables = plan.syllables();

    assert_eq!(syllables.len(), 3);
    assert_float_bits(syllables[0].declination_offset_semitones(), 0.0);
    assert_float_bits(syllables[1].declination_offset_semitones(), 0.0);
    assert_float_bits(syllables[2].declination_offset_semitones(), -0.28);
}

fn assert_float_bits(actual: f64, expected: f64) {
    assert_eq!(actual.to_bits(), expected.to_bits());
}

#[test]
fn period_settles_deeper_than_an_exclamation_punch() {
    // VOICE_V9 (R1): a period falls all the way to a quiet settle, while an
    // exclamation falls only shallowly from its raised, emphasized peak so it
    // stays energetic. A question keeps its suppressed (rising) close.
    let period = sentence_final_lowering("all done.");
    let exclamation = sentence_final_lowering("all done!");
    let question = sentence_final_lowering("all done?");

    assert!(
        period < exclamation,
        "a period should settle deeper than an exclamation: {period} vs {exclamation}",
    );
    assert!(
        exclamation < 0.0,
        "an exclamation should still fall from its raised peak: {exclamation}",
    );
    assert_float_bits(question, 0.0);
}

fn sentence_final_lowering(text: &str) -> f64 {
    let events = sequence_events_for_text(text).expect("text should analyze");
    let plan = plan_phrase_prosody(&events);

    plan.syllables()
        .last()
        .expect("the phrase should have a final syllable")
        .final_lowering_semitones()
}

#[test]
fn clause_boundary_drops_its_lowering_for_a_continuation_rise() {
    // VOICE_V9 (R4): a clause mark carries an open continuation rise, so it must
    // not also impose a final lowering that would erase that rise at the tail.
    let events = sequence_events_for_text("alpha beta, gamma").expect("text should analyze");
    let plan = plan_phrase_prosody(&events);
    let syllables = plan.syllables();

    assert_eq!(
        syllables[1].boundary_strength(),
        PhraseBoundaryStrength::Clause,
    );
    assert_float_bits(syllables[1].final_lowering_semitones(), 0.0);
}

#[test]
fn phrase_planner_snapshot_is_stable_for_mixed_boundaries() {
    let events = sequence_events_for_text("alpha beta, gamma delta!").expect("text should analyze");
    let plan = plan_phrase_prosody(&events);

    insta::assert_debug_snapshot!(plan, @r###"
    PhrasePlan {
        syllables: [
            PhraseSyllablePlan {
                syllable_index: 0,
                boundary_strength: Word,
                declination_offset_semitones: 0.0,
                pitch_reset_semitones: 0.0,
                final_lowering_semitones: 0.0,
                pre_boundary_lengthening: 1.0,
                pause_samples: 4851,
                emphasized: true,
            },
            PhraseSyllablePlan {
                syllable_index: 1,
                boundary_strength: Clause,
                declination_offset_semitones: -0.28,
                pitch_reset_semitones: 0.45,
                final_lowering_semitones: 0.0,
                pre_boundary_lengthening: 1.12,
                pause_samples: 6615,
                emphasized: false,
            },
            PhraseSyllablePlan {
                syllable_index: 2,
                boundary_strength: Word,
                declination_offset_semitones: -0.56,
                pitch_reset_semitones: 0.0,
                final_lowering_semitones: 0.0,
                pre_boundary_lengthening: 1.0,
                pause_samples: 4851,
                emphasized: false,
            },
            PhraseSyllablePlan {
                syllable_index: 3,
                boundary_strength: Sentence,
                declination_offset_semitones: -0.84,
                pitch_reset_semitones: 1.2,
                final_lowering_semitones: -0.6,
                pre_boundary_lengthening: 1.25,
                pause_samples: 10584,
                emphasized: true,
            },
        ],
    }
    "###);
}
