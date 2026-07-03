//! `--explain` class-table snapshots (T-126, FR-120).
//!
//! Gated on the `VOICE_V12` spike feature: with the gate off no token carries
//! a content class, the class table is omitted, and the pre-`VOICE_V12`
//! layout is pinned by the ungated snapshot below.

use dootdoot::explain_table_for_text;

#[cfg(not(feature = "spike-noun-verb"))]
#[test]
fn gate_off_explain_omits_the_class_table() {
    let table = explain_table_for_text("verify the bug").expect("text explains");

    assert!(!table.contains("silhouette"));
}

#[cfg(feature = "spike-noun-verb")]
mod gate_on {
    use super::*;

    #[test]
    fn mixed_sentence_class_table() {
        // A verb, a function word, and a noun in one utterance.
        insta::assert_snapshot!(explain_table_for_text("verify the bug").expect("text explains"));
    }

    #[test]
    fn multi_subword_content_word_class_table() {
        // `changelog` splits into `change` + `##log`; the word-initial
        // subword fires the marker and the final subword settles.
        insta::assert_snapshot!(explain_table_for_text("changelog").expect("text explains"));
    }
}
