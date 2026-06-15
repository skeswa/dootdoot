//! `VOICE_V7` dash/ellipsis hesitation marker tests.

use dootdoot_core::{
    DASH_HESITATION_PAUSE_SAMPLES, ExplainRow, HesitationMarker, explain_rows_for_text,
    render_text_canonical_buffer,
};

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

fn has_token_row(rows: &[ExplainRow], text: &str) -> bool {
    rows.iter().any(|row| match row {
        ExplainRow::Token(token) => token.token() == text,
        _ => false,
    })
}

fn has_hesitation_row(rows: &[ExplainRow]) -> bool {
    rows.iter()
        .any(|row| matches!(row, ExplainRow::Hesitation(_)))
}

#[test]
fn hesitation_marker_parses_dash_and_ellipsis_forms() {
    assert_eq!(
        HesitationMarker::from_text("-"),
        Some(HesitationMarker::Dash)
    );
    assert_eq!(
        HesitationMarker::from_text("—"),
        Some(HesitationMarker::Dash),
    );
    assert_eq!(
        HesitationMarker::from_text("–"),
        Some(HesitationMarker::Dash),
    );
    assert_eq!(
        HesitationMarker::from_text("…"),
        Some(HesitationMarker::Ellipsis),
    );
    assert_eq!(HesitationMarker::from_text("a"), None);
    assert_eq!(HesitationMarker::from_text("."), None);
}

#[test]
fn standalone_dash_is_not_a_four_axis_token() {
    let rows = explain_rows_for_text("a - b").expect("explain rows should build");

    assert!(
        !has_token_row(&rows, "-"),
        "the standalone dash must not appear with four-axis values",
    );
    assert!(has_token_row(&rows, "a"));
    assert!(has_token_row(&rows, "b"));
    assert!(
        has_hesitation_row(&rows),
        "the dash should surface as a hesitation control row",
    );
}

#[test]
fn em_dash_is_not_a_four_axis_token() {
    let rows = explain_rows_for_text("a — b").expect("explain rows should build");

    assert!(!has_token_row(&rows, "—"));
    assert!(has_hesitation_row(&rows));
}

#[test]
fn dash_routes_to_a_real_rest_instead_of_a_bridge() {
    let bridged = render_text_canonical_buffer("a b").expect("render should succeed");
    let hesitated = render_text_canonical_buffer("a - b").expect("render should succeed");

    assert!(
        max_zero_run(&bridged) < DASH_HESITATION_PAUSE_SAMPLES as usize,
        "a plain word boundary should bridge with tone",
    );
    assert!(
        max_zero_run(&hesitated) >= DASH_HESITATION_PAUSE_SAMPLES as usize,
        "a dash should open a real hesitation rest",
    );
}

#[test]
fn bare_dash_routes_to_the_empty_chirp() {
    let chirp = render_text_canonical_buffer("").expect("empty render should succeed");
    let bare_dash = render_text_canonical_buffer("-").expect("bare dash render should succeed");

    assert_eq!(
        bare_dash, chirp,
        "a bare dash has no syllable, so it chirps"
    );
}
