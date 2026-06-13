//! Embedded tokenizer wrapper tests.

use dootdoot_core::embedded_tokenizer;

#[test]
fn embedded_tokenizer_disables_injected_special_tokens() {
    let wrapper = embedded_tokenizer().expect("embedded tokenizer should load");
    let output = wrapper.tokenize("hello").expect("hello should tokenize");

    assert!(!output.is_empty_chirp());
    assert_eq!(output.tokens().len(), 1);
    assert_eq!(output.tokens()[0].id(), 6_598);
    assert_eq!(output.tokens()[0].text(), "hello");
    assert!(!output.tokens()[0].is_continuation());
    assert!(
        !output
            .tokens()
            .iter()
            .any(|token| matches!(token.id(), 2 | 3))
    );
}

#[test]
fn tokenizer_drops_literal_control_tokens_but_keeps_unknown() {
    let wrapper = embedded_tokenizer().expect("embedded tokenizer should load");

    assert_eq!(wrapper.control_token_ids(), [0, 2, 3, 4]);
    assert_eq!(wrapper.unknown_token_id(), 1);

    let control_only = wrapper
        .tokenize("[CLS] [MASK]")
        .expect("control tokens should tokenize");
    assert!(control_only.is_empty_chirp());
    assert!(control_only.tokens().is_empty());

    let unknown = wrapper
        .tokenize("[UNK]")
        .expect("unknown token should tokenize");
    assert!(!unknown.is_empty_chirp());
    assert_eq!(unknown.tokens().len(), 1);
    assert_eq!(unknown.tokens()[0].id(), 1);
    assert_eq!(unknown.tokens()[0].text(), "[UNK]");
}

#[test]
fn tokenizer_marks_wordpiece_continuations() {
    let wrapper = embedded_tokenizer().expect("embedded tokenizer should load");
    let output = wrapper
        .tokenize("tokenizer")
        .expect("tokenizer should split as wordpiece");

    assert_eq!(output.tokens().len(), 2);
    assert_eq!(output.tokens()[0].id(), 18_210);
    assert_eq!(output.tokens()[0].text(), "token");
    assert!(!output.tokens()[0].is_continuation());
    assert_eq!(output.tokens()[1].id(), 16_635);
    assert_eq!(output.tokens()[1].text(), "##izer");
    assert!(output.tokens()[1].is_continuation());
}
