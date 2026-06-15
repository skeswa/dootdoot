//! Utterance sequencer tests.

use dootdoot_core::{
    BASE_SYLLABLE_SAMPLES, CLAUSE_SYLLABLE_SAMPLES, KnobSet, LEADING_SILENCE_SAMPLES,
    LONG_PUNCTUATION_PAUSE_SAMPLES, ProsodicPunctuation, SENTENCE_SYLLABLE_SAMPLES, SequenceEvent,
    SequencedUtterance, SquashedVector, TRAILING_SILENCE_SAMPLES, WORD_PAUSE_SAMPLES,
    assemble_knobs, render_empty_chirp, sequence_utterance,
};

#[test]
fn sequencer_routes_zero_voiced_events_to_empty_chirp() {
    let output = sequence_utterance(&[
        SequenceEvent::punctuation(ProsodicPunctuation::Question),
        SequenceEvent::punctuation(ProsodicPunctuation::Exclamation),
    ]);
    let samples = rendered_samples(&output);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable = usize::try_from(BASE_SYLLABLE_SAMPLES).expect("syllable fits usize");
    let trailing = usize::try_from(TRAILING_SILENCE_SAMPLES).expect("trailing silence fits usize");

    assert!(output.is_empty_chirp());
    assert_eq!(samples.len(), leading + syllable + trailing);
    assert!(
        samples[leading..leading + syllable]
            .iter()
            .any(|sample| sample.abs() > 0.000_001)
    );
}

#[test]
fn empty_chirp_renderer_is_bit_exact_and_used_by_empty_sequence() {
    let direct = render_empty_chirp();
    let repeated = render_empty_chirp();
    let sequenced = sequence_utterance(&[]);
    let direct_bits = direct
        .iter()
        .map(|sample| sample.to_bits())
        .collect::<Vec<_>>();
    let repeated_bits = repeated
        .iter()
        .map(|sample| sample.to_bits())
        .collect::<Vec<_>>();
    let sequenced_bits = rendered_samples(&sequenced)
        .iter()
        .map(|sample| sample.to_bits())
        .collect::<Vec<_>>();

    assert_eq!(direct_bits, repeated_bits);
    assert_eq!(direct_bits, sequenced_bits);
}

#[test]
fn sequencer_wraps_one_syllable_with_fixed_padding() {
    let output = sequence_utterance(&[SequenceEvent::syllable(neutral_knobs(), false)]);
    let samples = rendered_samples(&output);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable = usize::try_from(BASE_SYLLABLE_SAMPLES).expect("syllable fits usize");
    let trailing = usize::try_from(TRAILING_SILENCE_SAMPLES).expect("trailing silence fits usize");

    assert_eq!(samples.len(), leading + syllable + trailing);
    assert!(samples[..leading].iter().all(|sample| *sample == 0.0));
    assert!(
        samples[leading..leading + syllable]
            .iter()
            .any(|sample| sample.abs() > 0.000_001)
    );
    assert!(
        samples[leading + syllable..]
            .iter()
            .all(|sample| *sample == 0.0)
    );
}

#[test]
fn sequencer_inserts_word_pause_between_non_continuation_syllables() {
    let output = sequence_utterance(&[
        SequenceEvent::syllable(neutral_knobs(), false),
        SequenceEvent::syllable(shifted_knobs(), false),
    ]);
    let samples = rendered_samples(&output);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable = usize::try_from(BASE_SYLLABLE_SAMPLES).expect("syllable fits usize");
    let word_pause = usize::try_from(WORD_PAUSE_SAMPLES).expect("word pause fits usize");
    let trailing = usize::try_from(TRAILING_SILENCE_SAMPLES).expect("trailing silence fits usize");
    let pause_start = leading + syllable;
    let pause_end = pause_start + word_pause;

    assert_eq!(
        samples.len(),
        leading + syllable + word_pause + syllable + trailing
    );
    assert!(
        samples[pause_start..pause_end]
            .iter()
            .all(|sample| *sample == 0.0)
    );
}

#[test]
fn sequencer_offsets_warble_phase_for_repeated_identical_syllables() {
    let output = sequence_utterance(&[
        SequenceEvent::syllable(neutral_knobs(), false),
        SequenceEvent::syllable(neutral_knobs(), false),
    ]);
    let samples = rendered_samples(&output);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable = usize::try_from(BASE_SYLLABLE_SAMPLES).expect("syllable fits usize");
    let word_pause = usize::try_from(WORD_PAUSE_SAMPLES).expect("word pause fits usize");
    let probe_offset = 512;
    let first_probe = leading + probe_offset;
    let second_probe = leading + syllable + word_pause + probe_offset;

    assert_ne!(
        samples[first_probe].to_bits(),
        samples[second_probe].to_bits(),
    );
}

#[test]
fn sequencer_connects_continuation_syllables_without_silence() {
    let output = sequence_utterance(&[
        SequenceEvent::syllable(neutral_knobs(), false),
        SequenceEvent::syllable(shifted_knobs(), true),
    ]);
    let samples = rendered_samples(&output);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable = usize::try_from(BASE_SYLLABLE_SAMPLES).expect("syllable fits usize");
    let trailing = usize::try_from(TRAILING_SILENCE_SAMPLES).expect("trailing silence fits usize");
    let boundary = leading + syllable;

    assert_eq!(samples.len(), leading + syllable + syllable + trailing);
    assert!(
        samples[boundary..boundary + syllable]
            .iter()
            .any(|sample| sample.abs() > 0.000_001)
    );
}

#[test]
fn sequencer_drops_leading_punctuation_and_adds_attached_pause() {
    let output = sequence_utterance(&[
        SequenceEvent::punctuation(ProsodicPunctuation::Question),
        SequenceEvent::syllable(neutral_knobs(), false),
        SequenceEvent::punctuation(ProsodicPunctuation::Exclamation),
    ]);
    let samples = rendered_samples(&output);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable = usize::try_from(SENTENCE_SYLLABLE_SAMPLES).expect("syllable fits usize");
    let long_pause =
        usize::try_from(LONG_PUNCTUATION_PAUSE_SAMPLES).expect("long pause fits usize");
    let trailing = usize::try_from(TRAILING_SILENCE_SAMPLES).expect("trailing silence fits usize");
    let pause_start = leading + syllable;
    let pause_end = pause_start + long_pause;

    assert_eq!(samples.len(), leading + syllable + long_pause + trailing);
    assert!(
        samples[pause_start..pause_end]
            .iter()
            .all(|sample| *sample == 0.0)
    );
}

#[test]
fn sequencer_shapes_attached_punctuation_final_glide() {
    let question = sequence_utterance(&[
        SequenceEvent::syllable(shifted_knobs(), false),
        SequenceEvent::punctuation(ProsodicPunctuation::Question),
    ]);
    let period = sequence_utterance(&[
        SequenceEvent::syllable(shifted_knobs(), false),
        SequenceEvent::punctuation(ProsodicPunctuation::Period),
    ]);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable = usize::try_from(BASE_SYLLABLE_SAMPLES).expect("syllable fits usize");
    let question_bits = rendered_samples(&question)[leading..leading + syllable]
        .iter()
        .map(|sample| sample.to_bits())
        .collect::<Vec<_>>();
    let period_bits = rendered_samples(&period)[leading..leading + syllable]
        .iter()
        .map(|sample| sample.to_bits())
        .collect::<Vec<_>>();

    assert!(question_bits != period_bits);
}

#[test]
fn prosodic_punctuation_parses_only_frozen_control_symbols() {
    assert_eq!(
        ProsodicPunctuation::from_text("?"),
        Some(ProsodicPunctuation::Question),
    );
    assert_eq!(
        ProsodicPunctuation::from_text("."),
        Some(ProsodicPunctuation::Period),
    );
    assert_eq!(
        ProsodicPunctuation::from_text("!"),
        Some(ProsodicPunctuation::Exclamation),
    );
    assert_eq!(
        ProsodicPunctuation::from_text(","),
        Some(ProsodicPunctuation::Comma),
    );
    assert_eq!(
        ProsodicPunctuation::from_text(";"),
        Some(ProsodicPunctuation::Semicolon),
    );
    assert_eq!(
        ProsodicPunctuation::from_text(":"),
        Some(ProsodicPunctuation::Colon),
    );
    assert_eq!(ProsodicPunctuation::from_text("..."), None);
    assert_eq!(ProsodicPunctuation::from_text("[UNK]"), None);
}

#[test]
fn sequencer_uses_first_consecutive_marker_for_glide_and_longest_single_pause() {
    let comma_then_question = sequence_utterance(&[
        SequenceEvent::syllable(shifted_knobs(), false),
        SequenceEvent::punctuation(ProsodicPunctuation::Comma),
        SequenceEvent::punctuation(ProsodicPunctuation::Question),
    ]);
    let comma_only = sequence_utterance(&[
        SequenceEvent::syllable(shifted_knobs(), false),
        SequenceEvent::punctuation(ProsodicPunctuation::Comma),
    ]);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable = usize::try_from(CLAUSE_SYLLABLE_SAMPLES).expect("syllable fits usize");
    let long_pause =
        usize::try_from(LONG_PUNCTUATION_PAUSE_SAMPLES).expect("long pause fits usize");
    let trailing = usize::try_from(TRAILING_SILENCE_SAMPLES).expect("trailing silence fits usize");
    let comma_question_samples = rendered_samples(&comma_then_question);
    let comma_only_samples = rendered_samples(&comma_only);
    let comma_question_bits = comma_question_samples[leading..leading + syllable]
        .iter()
        .map(|sample| sample.to_bits())
        .collect::<Vec<_>>();
    let comma_only_bits = comma_only_samples[leading..leading + syllable]
        .iter()
        .map(|sample| sample.to_bits())
        .collect::<Vec<_>>();

    assert_eq!(
        comma_question_samples.len(),
        leading + syllable + long_pause + trailing,
    );
    assert_eq!(comma_question_bits, comma_only_bits);
}

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

fn rendered_samples(output: &SequencedUtterance) -> &[f64] {
    output.samples()
}
