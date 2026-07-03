//! Black-box tests for the `VOICE_V12` layered co-onset class markers (T-116).
//!
//! Noun = broadband click/pop splash, verb = up-swept dual-sine chirp; both
//! start together with the tonal body (co-onset, zero added duration) and fire
//! only on word-initial content syllables. The `Other`/neutral path stays
//! byte-identical.

use dootdoot_core::{
    ATTACK_TRANSIENT_MIX, KnobSet, NOUN_MARKER_MIX, NOUN_MARKER_SECONDS, PosClass, SequenceEvent,
    SquashedVector, SyllableEvent, VERB_MARKER_MIX, VERB_MARKER_SECONDS, assemble_knobs,
    class_onset_marker_sample, sequence_utterance,
};

fn neutral_knobs() -> KnobSet {
    let neutral = SquashedVector::new([0.0, 0.0, 0.0, 0.0]);

    assemble_knobs(neutral, neutral)
}

fn rendered(events: &[SequenceEvent]) -> Vec<f64> {
    sequence_utterance(events).samples().to_vec()
}

#[test]
fn other_class_marker_is_silent() {
    for step in 0..200 {
        let elapsed_seconds = f64::from(step) * 0.000_5;

        assert_eq!(
            class_onset_marker_sample(elapsed_seconds, PosClass::Other).to_bits(),
            0.0_f64.to_bits(),
        );
    }
}

#[test]
fn noun_marker_is_bounded_and_windowed() {
    let mut heard_energy = false;

    for step in 0..400 {
        let elapsed_seconds = f64::from(step) * 0.000_1;
        let sample = class_onset_marker_sample(elapsed_seconds, PosClass::Noun);

        assert!(sample.is_finite());
        assert!(sample.abs() <= NOUN_MARKER_MIX);

        if elapsed_seconds >= NOUN_MARKER_SECONDS {
            assert_eq!(sample.to_bits(), 0.0_f64.to_bits());
        } else if sample.abs() > 0.01 {
            heard_energy = true;
        }
    }

    assert!(heard_energy);
}

#[test]
fn verb_marker_is_bounded_and_windowed() {
    let mut heard_energy = false;

    for step in 0..800 {
        let elapsed_seconds = f64::from(step) * 0.000_1;
        let sample = class_onset_marker_sample(elapsed_seconds, PosClass::Verb);

        assert!(sample.is_finite());
        assert!(sample.abs() <= VERB_MARKER_MIX);

        if elapsed_seconds >= VERB_MARKER_SECONDS {
            assert_eq!(sample.to_bits(), 0.0_f64.to_bits());
        } else if sample.abs() > 0.01 {
            heard_energy = true;
        }
    }

    assert!(heard_energy);
}

#[test]
fn verb_marker_sweeps_upward() {
    // An up-swept chirp crosses zero more often late in its window than early.
    let half_window = VERB_MARKER_SECONDS / 2.0;
    let crossings = |start: f64| {
        let mut count = 0_u32;
        let mut last_sign = 0_i8;

        for step in 0..2_000 {
            let elapsed_seconds = start + (f64::from(step) / 2_000.0) * half_window;
            let sample = class_onset_marker_sample(elapsed_seconds, PosClass::Verb);
            let sign = if sample > 0.0 {
                1
            } else if sample < 0.0 {
                -1
            } else {
                0
            };

            if sign != 0 && last_sign != 0 && sign != last_sign {
                count += 1;
            }

            if sign != 0 {
                last_sign = sign;
            }
        }

        count
    };

    assert!(crossings(half_window) > crossings(0.0));
}

#[test]
fn markers_are_louder_than_the_softened_word_transient() {
    const {
        assert!(NOUN_MARKER_MIX > ATTACK_TRANSIENT_MIX);
        assert!(VERB_MARKER_MIX > ATTACK_TRANSIENT_MIX);
    }
}

#[test]
fn marker_classes_cross_temporal_categories() {
    // Noun = short impact, verb = longer glide (P6: cross-category contrast).
    const {
        assert!(NOUN_MARKER_SECONDS < VERB_MARKER_SECONDS);
    }
}

#[test]
fn word_initial_content_syllable_renders_the_marker() {
    let unmarked = rendered(&[SequenceEvent::Syllable(SyllableEvent::new(
        neutral_knobs(),
        false,
    ))]);
    let marked = rendered(&[SequenceEvent::Syllable(
        SyllableEvent::new(neutral_knobs(), false).with_pos_class(PosClass::Noun),
    )]);

    assert_eq!(unmarked.len(), marked.len());
    assert_ne!(unmarked, marked);
}

#[test]
fn noun_and_verb_markers_render_differently() {
    let noun = rendered(&[SequenceEvent::Syllable(
        SyllableEvent::new(neutral_knobs(), false).with_pos_class(PosClass::Noun),
    )]);
    let verb = rendered(&[SequenceEvent::Syllable(
        SyllableEvent::new(neutral_knobs(), false).with_pos_class(PosClass::Verb),
    )]);

    assert_ne!(noun, verb);
}

#[test]
fn continuation_syllables_stay_unmarked() {
    let base = vec![
        SequenceEvent::Syllable(SyllableEvent::new(neutral_knobs(), false)),
        SequenceEvent::Syllable(SyllableEvent::new(neutral_knobs(), true)),
    ];
    let classed = vec![
        base[0],
        SequenceEvent::Syllable(
            SyllableEvent::new(neutral_knobs(), true).with_pos_class(PosClass::Verb),
        ),
    ];

    assert_eq!(rendered(&base), rendered(&classed));
}

#[test]
fn marked_rendering_is_deterministic_and_finite() {
    let events = vec![
        SequenceEvent::Syllable(
            SyllableEvent::new(neutral_knobs(), false).with_pos_class(PosClass::Noun),
        ),
        SequenceEvent::Syllable(
            SyllableEvent::new(neutral_knobs(), false).with_pos_class(PosClass::Verb),
        ),
    ];
    let first = rendered(&events);
    let second = rendered(&events);

    assert_eq!(first, second);
    assert!(first.iter().all(|sample| sample.is_finite()));
}
