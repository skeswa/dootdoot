use super::{
    AmbiguityPolicy, PosClass, SPIKE_AMBIGUOUS, SPIKE_NOUNS, SPIKE_VERBS, spike_lexicon_class,
};

#[test]
fn lexicon_tables_are_sorted_and_unique_for_binary_search() {
    assert!(SPIKE_NOUNS.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(SPIKE_VERBS.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(SPIKE_AMBIGUOUS.windows(2).all(|pair| pair[0].0 < pair[1].0));
}

#[test]
fn ambiguous_lemmas_never_repeat_the_unambiguous_tables() {
    for (lemma, _class) in SPIKE_AMBIGUOUS {
        assert!(SPIKE_NOUNS.binary_search(&lemma).is_err(), "{lemma}");
        assert!(SPIKE_VERBS.binary_search(&lemma).is_err(), "{lemma}");
    }
}

#[test]
fn unambiguous_lemmas_classify_by_their_table() {
    assert_eq!(
        spike_lexicon_class("bug", AmbiguityPolicy::DominantClass),
        PosClass::Noun
    );
    assert_eq!(
        spike_lexicon_class("deploy", AmbiguityPolicy::DominantClass),
        PosClass::Verb
    );
}

#[test]
fn closed_class_and_unknown_words_stay_other() {
    for word in ["the", "can", "will", "of", "quixotic"] {
        assert_eq!(
            spike_lexicon_class(word, AmbiguityPolicy::DominantClass),
            PosClass::Other,
            "{word}"
        );
    }
}

#[test]
fn lookup_is_case_insensitive() {
    assert_eq!(
        spike_lexicon_class("Fix", AmbiguityPolicy::DominantClass),
        PosClass::Verb
    );
    assert_eq!(
        spike_lexicon_class("ERROR", AmbiguityPolicy::DominantClass),
        PosClass::Noun
    );
}

#[test]
fn dominant_policy_marks_ambiguous_lemmas_with_their_dominant_class() {
    assert_eq!(
        spike_lexicon_class("run", AmbiguityPolicy::DominantClass),
        PosClass::Verb
    );
    assert_eq!(
        spike_lexicon_class("log", AmbiguityPolicy::DominantClass),
        PosClass::Noun
    );
}

#[test]
fn conservative_policy_leaves_ambiguous_lemmas_unmarked() {
    for lemma in ["build", "fix", "run", "update", "sync"] {
        assert_eq!(
            spike_lexicon_class(lemma, AmbiguityPolicy::FallBackToOther),
            PosClass::Other,
            "{lemma}"
        );
    }
}

#[test]
fn content_classes_report_content() {
    assert!(PosClass::Noun.is_content());
    assert!(PosClass::Verb.is_content());
    assert!(!PosClass::Other.is_content());
}
