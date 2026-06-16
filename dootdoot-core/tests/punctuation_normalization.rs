//! `VOICE_V9` punctuation-run normalization tests (R2).
//!
//! ASCII ellipsis runs (`...`) should collapse into one trailing-off ellipsis
//! hesitation marker instead of a stutter of falling-glide periods, and stacked
//! terminal punctuation (`?!`, `!!!`) should resolve to a single terminal
//! contour rather than several.

use dootdoot_core::{
    ELLIPSIS_HESITATION_PAUSE_SAMPLES, ExplainRow, HesitationMarker, ProsodicPunctuation,
    explain_rows_for_text, render_text_canonical_buffer,
};

fn hesitation_markers(rows: &[ExplainRow]) -> Vec<HesitationMarker> {
    rows.iter()
        .filter_map(|row| match row {
            ExplainRow::Hesitation(hesitation) => Some(hesitation.marker()),
            _ => None,
        })
        .collect()
}

fn punctuation_marks(rows: &[ExplainRow]) -> Vec<ProsodicPunctuation> {
    rows.iter()
        .filter_map(|row| match row {
            ExplainRow::Punctuation(punctuation) => Some(punctuation.punctuation()),
            _ => None,
        })
        .collect()
}

fn max_zero_run(samples: &[i16]) -> usize {
    let mut best = 0;
    let mut run = 0;

    for &sample in samples {
        if sample == 0 {
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
    }

    best
}

#[test]
fn ascii_ellipsis_collapses_into_one_ellipsis_hesitation() {
    let rows = explain_rows_for_text("wait... no").expect("explain rows should build");

    assert_eq!(
        hesitation_markers(&rows),
        vec![HesitationMarker::Ellipsis],
        "three ASCII dots should become exactly one ellipsis hesitation",
    );
    assert!(
        punctuation_marks(&rows).is_empty(),
        "the dots should not survive as falling-glide period controls",
    );
}

#[test]
fn four_dot_run_is_still_one_ellipsis() {
    let rows = explain_rows_for_text("wait.... no").expect("explain rows should build");

    assert_eq!(hesitation_markers(&rows), vec![HesitationMarker::Ellipsis]);
}

#[test]
fn ascii_ellipsis_opens_a_quiet_trailing_rest() {
    let rendered = render_text_canonical_buffer("wait... no").expect("render should succeed");

    assert!(
        max_zero_run(&rendered) >= ELLIPSIS_HESITATION_PAUSE_SAMPLES as usize,
        "an ASCII ellipsis should open a real trailing-off rest",
    );
}

#[test]
fn single_period_stays_a_period() {
    let rows = explain_rows_for_text("go. now").expect("explain rows should build");

    assert_eq!(
        punctuation_marks(&rows),
        vec![ProsodicPunctuation::Period],
        "a lone period must remain a sentence period, not an ellipsis",
    );
    assert!(hesitation_markers(&rows).is_empty());
}

#[test]
fn interrobang_resolves_to_one_terminal_contour() {
    let question_first = explain_rows_for_text("really?!").expect("explain rows should build");
    let bang_first = explain_rows_for_text("really!?").expect("explain rows should build");

    assert_eq!(
        punctuation_marks(&question_first),
        vec![ProsodicPunctuation::Question],
        "?! should keep the first-typed terminal contour and not stack a second",
    );
    assert_eq!(
        punctuation_marks(&bang_first),
        vec![ProsodicPunctuation::Exclamation],
        "!? should keep the first-typed terminal contour and not stack a second",
    );
}

#[test]
fn repeated_terminal_marks_collapse_to_one() {
    let rows = explain_rows_for_text("stop!!!").expect("explain rows should build");

    assert_eq!(
        punctuation_marks(&rows),
        vec![ProsodicPunctuation::Exclamation],
        "a run of exclamation marks should not stack three falling glides",
    );
}
