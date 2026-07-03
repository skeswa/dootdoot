//! `--explain` class-table snapshots (T-126, FR-120).

use dootdoot::explain_table_for_text;

#[test]
fn unclassified_utterances_omit_the_class_table() {
    // Every content word here is either closed-class or ambiguous-unmarked,
    // so no class table is printed and the pre-`VOICE_V12` layout holds.
    let table = explain_table_for_text("fix the build").expect("text explains");

    assert!(!table.contains("silhouette"));
}

#[test]
fn mixed_sentence_class_table() {
    // A verb, a function word, and a noun in one utterance.
    insta::assert_snapshot!(explain_table_for_text("verify the bug").expect("text explains"));
}

#[test]
fn multi_subword_content_word_class_table() {
    // `changelog` splits into `change` + `##log`; the word-initial subword
    // fires the marker and the final subword settles.
    insta::assert_snapshot!(explain_table_for_text("changelog").expect("text explains"));
}
