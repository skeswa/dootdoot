//! Black-box tests for the `VOICE_V12` compound `stem → class-resolution`
//! silhouette (T-117).
//!
//! A single-token content word expands to two syllables: the stem carries the
//! word's semantics and the resolution is a frozen per-class transform of the
//! stem's own knobs — noun *settles* (rounds toward `oo`, steps down,
//! flattens), verb *pushes* (brightens, rises). Compound words shorten their
//! per-syllable base duration so a two-syllable word is not twice a single
//! blip. Function words stay single light blips.

use dootdoot_core::{
    BASE_SYLLABLE_SAMPLES, COMPOUND_SYLLABLE_DURATION_SCALE, KnobSet, PosClass, SequenceEvent,
    SquashedVector, SyllableEvent, assemble_knobs, class_resolution_knobs,
    estimate_syllable_sample_counts, estimate_utterance_sample_count, sequence_utterance,
};

fn neutral_knobs() -> KnobSet {
    let neutral = SquashedVector::new([0.0, 0.0, 0.0, 0.0]);

    assemble_knobs(neutral, neutral)
}

fn shifted_knobs() -> KnobSet {
    assemble_knobs(
        SquashedVector::new([0.0, 0.0, 0.0, 0.0]),
        SquashedVector::new([0.35, -0.25, 0.50, 0.10]),
    )
}

#[test]
fn other_class_resolution_returns_the_stem_unchanged() {
    let stem = shifted_knobs();

    assert_eq!(class_resolution_knobs(stem, PosClass::Other), stem);
}

#[test]
fn noun_resolution_settles() {
    let stem = shifted_knobs();
    let resolution = class_resolution_knobs(stem, PosClass::Noun);

    // Object at rest: pitch steps down, the vowel rounds toward `oo` (+1),
    // and the contour flattens.
    assert!(resolution.pitch_center() < stem.pitch_center());
    assert!(resolution.vowel_position() > stem.vowel_position());
    assert!(resolution.contour().abs() < stem.contour().abs());
}

#[test]
fn verb_resolution_pushes() {
    let stem = shifted_knobs();
    let resolution = class_resolution_knobs(stem, PosClass::Verb);

    // Action carries forward: brighter (toward `ee`, -1) and rising.
    assert!(resolution.vowel_position() < stem.vowel_position());
    assert!(resolution.contour() > 0.3);
    assert!(resolution.pitch_center() >= stem.pitch_center());
}

#[test]
fn resolution_knobs_stay_inside_the_knob_bounds() {
    for axes in [
        [1.0, 1.0, 1.0, 1.0],
        [-1.0, -1.0, -1.0, -1.0],
        [1.0, -1.0, 1.0, -1.0],
    ] {
        let stem = assemble_knobs(
            SquashedVector::new([0.0, 0.0, 0.0, 0.0]),
            SquashedVector::new(axes),
        );

        for pos_class in [PosClass::Noun, PosClass::Verb] {
            let resolution = class_resolution_knobs(stem, pos_class);

            for axis in resolution.axes() {
                assert!((-1.0..=1.0).contains(&axis));
            }
        }
    }
}

#[test]
fn resolution_transform_is_deterministic() {
    let stem = shifted_knobs();

    assert_eq!(
        class_resolution_knobs(stem, PosClass::Noun),
        class_resolution_knobs(stem, PosClass::Noun),
    );
}

#[test]
fn compound_duration_scale_shortens_without_halving() {
    // A 2-syllable compound word should be longer than one blip but clearly
    // shorter than two.
    const {
        assert!(COMPOUND_SYLLABLE_DURATION_SCALE < 1.0);
        assert!(COMPOUND_SYLLABLE_DURATION_SCALE * 2.0 > 1.0);
    }
}

#[test]
fn default_duration_scale_renders_byte_identically() {
    let plain = sequence_utterance(&[SequenceEvent::Syllable(SyllableEvent::new(
        neutral_knobs(),
        false,
    ))]);
    let scaled = sequence_utterance(&[SequenceEvent::Syllable(
        SyllableEvent::new(neutral_knobs(), false).with_duration_scale(1.0),
    )]);

    assert_eq!(plain.samples(), scaled.samples());
}

#[test]
fn duration_scale_shortens_the_rendered_syllable() {
    let events = |scale: f64| {
        vec![SequenceEvent::Syllable(
            SyllableEvent::new(neutral_knobs(), false).with_duration_scale(scale),
        )]
    };
    let full = estimate_syllable_sample_counts(&events(1.0));
    let compound = estimate_syllable_sample_counts(&events(COMPOUND_SYLLABLE_DURATION_SCALE));

    assert_eq!(full, vec![BASE_SYLLABLE_SAMPLES]);
    assert!(compound[0] < full[0]);
    assert!(compound[0] > BASE_SYLLABLE_SAMPLES / 2);
}

#[test]
fn estimation_matches_rendering_for_compound_events() {
    let events = vec![
        SequenceEvent::Syllable(
            SyllableEvent::new(shifted_knobs(), false)
                .with_pos_class(PosClass::Noun)
                .with_duration_scale(COMPOUND_SYLLABLE_DURATION_SCALE),
        ),
        SequenceEvent::Syllable(
            SyllableEvent::new(
                class_resolution_knobs(shifted_knobs(), PosClass::Noun),
                true,
            )
            .with_pos_class(PosClass::Noun)
            .with_duration_scale(COMPOUND_SYLLABLE_DURATION_SCALE),
        ),
    ];
    let rendered = sequence_utterance(&events);
    let estimated = estimate_utterance_sample_count(&events);

    assert_eq!(
        u64::try_from(rendered.samples().len()).expect("length fits u64"),
        estimated
    );
}

#[cfg(feature = "spike-noun-verb")]
mod gate_on {
    use dootdoot_core::{PosClass, SequenceEvent, sequence_events_for_text};

    fn syllables(text: &str) -> Vec<dootdoot_core::SyllableEvent> {
        sequence_events_for_text(text)
            .expect("text renders")
            .iter()
            .filter_map(|event| match event {
                SequenceEvent::Syllable(syllable) => Some(*syllable),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn single_token_content_word_expands_to_stem_and_resolution() {
        let rows = syllables("bug");

        assert_eq!(rows.len(), 2);
        assert!(!rows[0].is_continuation());
        assert!(rows[1].is_continuation());
        assert_eq!(rows[0].pos_class(), PosClass::Noun);
        assert_eq!(rows[1].pos_class(), PosClass::Noun);
    }

    #[test]
    fn compound_syllables_carry_the_shortened_duration() {
        let rows = syllables("bug");

        for row in rows {
            assert!(row.duration_scale() < 1.0);
        }
    }

    #[test]
    fn function_words_stay_single_light_blips() {
        let rows = syllables("the");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pos_class(), PosClass::Other);
        assert!((rows[0].duration_scale() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn estimation_matches_rendering_for_expanded_text() {
        let events = sequence_events_for_text("fix the bug").expect("text renders");
        let rendered = dootdoot_core::sequence_utterance(&events);
        let estimated = dootdoot_core::estimate_utterance_sample_count(&events);

        assert_eq!(
            u64::try_from(rendered.samples().len()).expect("length fits u64"),
            estimated
        );
    }
}
