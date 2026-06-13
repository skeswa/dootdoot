//! End-to-end text rendering tests.

use dootdoot_core::{render_canonical_buffer, render_text_canonical_buffer};

#[test]
fn text_rendering_is_deterministic_and_non_silent() {
    let first = render_text_canonical_buffer("hello").expect("text should render");
    let second = render_text_canonical_buffer("hello").expect("text should render");

    assert_eq!(first, second);
    assert!(first.iter().any(|sample| *sample != 0));
}

#[test]
fn punctuation_only_text_uses_empty_chirp_buffer() {
    let punctuation = render_text_canonical_buffer("?").expect("punctuation should render");
    let chirp = render_canonical_buffer(&[]);

    assert_eq!(punctuation, chirp);
}
