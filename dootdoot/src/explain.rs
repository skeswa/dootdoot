//! Explanation table formatting for the command-line shell.

use std::fmt::Write as _;

use dootdoot_core::{EngineError, ExplainRow, ProsodicPunctuation, explain_rows_for_text};

const EXPLAIN_HEADER: &str = "token │ pitch │ vowel │ contour │ warble\n";

/// Formats an empty-input explain table.
pub fn explain_table_for_empty_chirp() -> String {
    format_explain_rows(&[])
}

/// Formats the per-token explain table for input text.
///
/// # Errors
///
/// Returns an error if text cannot be tokenized or mapped with `FORMAT_V1`.
pub fn explain_table_for_text(text: &str) -> Result<String, EngineError> {
    Ok(format_explain_rows(&explain_rows_for_text(text)?))
}

fn format_explain_rows(rows: &[ExplainRow]) -> String {
    let mut table = String::from(EXPLAIN_HEADER);

    for row in rows {
        match row {
            ExplainRow::Mood(mood) => {
                let mood = mood.mood();

                writeln!(
                    table,
                    "mood │ valence:{:+.3} │ arousal:{:+.3} │ - │ -",
                    mood.valence(),
                    mood.arousal(),
                )
                .expect("writing to a String cannot fail");
            }
            ExplainRow::Token(token) => {
                let knobs = token.knobs();

                writeln!(
                    table,
                    "{} │ {:+.3} │ {:+.3} │ {:+.3} │ {:+.3}",
                    token.token(),
                    knobs.pitch_center(),
                    knobs.vowel_position(),
                    knobs.contour(),
                    knobs.warble_depth(),
                )
                .expect("writing to a String cannot fail");
            }
            ExplainRow::Punctuation(punctuation) => {
                writeln!(
                    table,
                    "{} │ control:{} │ - │ - │ -",
                    punctuation.token(),
                    punctuation_name(punctuation.punctuation()),
                )
                .expect("writing to a String cannot fail");
            }
        }
    }

    table
}

fn punctuation_name(punctuation: ProsodicPunctuation) -> &'static str {
    match punctuation {
        ProsodicPunctuation::Question => "question",
        ProsodicPunctuation::Period => "period",
        ProsodicPunctuation::Exclamation => "exclamation",
        ProsodicPunctuation::Comma => "comma",
        ProsodicPunctuation::Semicolon => "semicolon",
        ProsodicPunctuation::Colon => "colon",
    }
}
