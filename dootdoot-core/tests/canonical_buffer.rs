//! Canonical audio buffer tests.

use dootdoot_core::{
    KnobSet, SequenceEvent, SquashedVector, assemble_knobs, quantize_sample,
    render_canonical_buffer, sequence_utterance,
};

#[test]
fn canonical_buffer_quantizes_the_sequenced_audio_samples() {
    let events = [
        SequenceEvent::syllable(neutral_knobs(), false),
        SequenceEvent::syllable(shifted_knobs(), true),
    ];
    let expected = sequence_utterance(&events)
        .samples()
        .iter()
        .copied()
        .map(quantize_sample)
        .collect::<Vec<_>>();

    assert_eq!(render_canonical_buffer(&events), expected);
}

#[test]
fn canonical_buffer_is_bit_exact_for_empty_chirp() {
    let first = render_canonical_buffer(&[]);
    let second = render_canonical_buffer(&[]);

    assert_eq!(first, second);
    assert!(first.iter().any(|sample| *sample != 0));
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
