//! Black-box tests for the `VOICE_V12` spike word-class plumbing (T-115).
//!
//! The POS class is a word-level property: word-initial tokens establish it and
//! continuation tokens inherit it. With the default-off `spike-noun-verb` gate
//! disabled, every word classifies `PosClass::Other`, so no downstream path can
//! change and all rendered audio stays byte-identical.

use dootdoot_core::{
    KnobSet, PerformanceSyllable, PosClass, SequenceEvent, SquashedVector, SyllableEvent,
    assemble_knobs, embedded_tokenizer, plan_discourse_performance,
};

fn neutral_knobs() -> KnobSet {
    let neutral = SquashedVector::new([0.0, 0.0, 0.0, 0.0]);

    assemble_knobs(neutral, neutral)
}

#[test]
fn syllable_event_defaults_to_other_class() {
    let event = SyllableEvent::new(neutral_knobs(), false);

    assert_eq!(event.pos_class(), PosClass::Other);
}

#[test]
fn syllable_event_carries_an_assigned_class() {
    let event = SyllableEvent::new(neutral_knobs(), false).with_pos_class(PosClass::Noun);

    assert_eq!(event.pos_class(), PosClass::Noun);
}

#[test]
fn planner_rows_carry_the_syllable_class() {
    let events = vec![
        SequenceEvent::Syllable(SyllableEvent::new(neutral_knobs(), false)),
        SequenceEvent::Syllable(
            SyllableEvent::new(neutral_knobs(), false).with_pos_class(PosClass::Verb),
        ),
        SequenceEvent::Syllable(
            SyllableEvent::new(neutral_knobs(), true).with_pos_class(PosClass::Noun),
        ),
    ];

    let plan = plan_discourse_performance(&events);
    let classes = plan
        .syllables()
        .iter()
        .map(PerformanceSyllable::pos_class)
        .collect::<Vec<_>>();

    assert_eq!(classes, [PosClass::Other, PosClass::Verb, PosClass::Noun]);
}

#[cfg(not(feature = "spike-noun-verb"))]
#[test]
fn gate_off_classifies_every_word_other() {
    let tokenizer = embedded_tokenizer().expect("the embedded tokenizer loads");
    let encoded = tokenizer
        .tokenize("fix the bug in deployment")
        .expect("plain text tokenizes");

    assert!(!encoded.tokens().is_empty());
    assert!(
        encoded
            .tokens()
            .iter()
            .all(|token| token.pos_class() == PosClass::Other)
    );
}

#[cfg(feature = "spike-noun-verb")]
mod gate_on {
    use super::*;

    #[test]
    fn word_initial_tokens_establish_their_lexicon_class() {
        let tokenizer = embedded_tokenizer().expect("the embedded tokenizer loads");
        let encoded = tokenizer.tokenize("add the bug").expect("text tokenizes");
        let classes = encoded
            .tokens()
            .iter()
            .map(|token| (token.text().to_owned(), token.pos_class()))
            .collect::<Vec<_>>();

        assert_eq!(
            classes,
            [
                ("add".to_owned(), PosClass::Verb),
                ("the".to_owned(), PosClass::Other),
                ("bug".to_owned(), PosClass::Noun),
            ]
        );
    }

    #[test]
    fn ambiguous_lemmas_stay_unmarked_under_the_locked_conservative_policy() {
        // T-118 round-1 by-ear decision: the conservative policy won the A/B,
        // so noun/verb-ambiguous coding lemmas classify `Other`.
        let tokenizer = embedded_tokenizer().expect("the embedded tokenizer loads");
        let encoded = tokenizer
            .tokenize("fix run update sync build")
            .expect("text tokenizes");

        assert!(
            encoded
                .tokens()
                .iter()
                .all(|token| token.pos_class() == PosClass::Other)
        );
    }

    #[test]
    fn continuation_tokens_inherit_the_word_class() {
        let tokenizer = embedded_tokenizer().expect("the embedded tokenizer loads");
        let encoded = tokenizer.tokenize("the workflow").expect("text tokenizes");
        let tokens = encoded.tokens();

        // Every token of one word carries the same class as its word-initial
        // token, whatever the WordPiece split happens to be.
        let mut word_class = PosClass::Other;

        for token in tokens {
            if token.is_continuation() {
                assert_eq!(token.pos_class(), word_class);
            } else {
                word_class = token.pos_class();
            }
        }

        let full_word_classes = tokens
            .iter()
            .filter(|token| !token.is_continuation())
            .map(dootdoot_core::TokenizedToken::pos_class)
            .collect::<Vec<_>>();

        assert_eq!(full_word_classes, [PosClass::Other, PosClass::Noun]);
    }
}
