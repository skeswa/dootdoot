//! Black-box tests for the `VOICE_V12` word-class plumbing (T-115/FR-115).
//!
//! The POS class is a word-level property: word-initial tokens establish it
//! from the baked class table and continuation tokens inherit it; hand-built
//! events default to `PosClass::Other` and render exactly as `VOICE_V11`.

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

mod classified {
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
    fn explain_rows_carry_class_marker_and_silhouette() {
        let rows =
            dootdoot_core::explain_rows_for_text("verify the changelog").expect("text explains");
        let token_rows = rows
            .iter()
            .filter_map(|row| match row {
                dootdoot_core::ExplainRow::Token(token) => Some(token),
                _ => None,
            })
            .collect::<Vec<_>>();
        let summary = token_rows
            .iter()
            .map(|token| {
                (
                    token.token(),
                    token.pos_class(),
                    token.onset_class(),
                    token.resolution_class(),
                    token.syllable_count(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            summary,
            [
                // `verify`: single-token verb — marked onset, expands to a
                // stem→push pair.
                ("verify", PosClass::Verb, PosClass::Verb, PosClass::Verb, 2),
                ("the", PosClass::Other, PosClass::Other, PosClass::Other, 1),
                // `changelog` splits: the word-initial subword fires the
                // marker, the final subword is the settle resolution.
                ("change", PosClass::Noun, PosClass::Noun, PosClass::Other, 1),
                ("##log", PosClass::Noun, PosClass::Other, PosClass::Noun, 1),
            ]
        );
    }

    #[test]
    fn ambiguous_lemmas_stay_unmarked_under_the_locked_conservative_policy() {
        // T-118 round-1 by-ear decision: the conservative policy won the A/B,
        // so noun/verb-ambiguous coding lemmas classify `Other`. (`fix`,
        // `run`, `update`, and `build` all exceed the 25% minority-use
        // threshold in the pinned corpus snapshot.)
        let tokenizer = embedded_tokenizer().expect("the embedded tokenizer loads");
        let encoded = tokenizer
            .tokenize("fix run update build")
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
        let encoded = tokenizer
            .tokenize("the deployment")
            .expect("text tokenizes");
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
