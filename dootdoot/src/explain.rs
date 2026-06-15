//! Explanation table formatting for the command-line shell.

use std::fmt::Write as _;

use dootdoot_core::{
    EngineError, ExplainRow, HesitationMarker, PhraseRole, ProsodicPunctuation,
    explain_rows_for_text,
};

const EXPLAIN_HEADER: &str = "token │ pitch │ vowel │ contour │ warble\n";

/// Formats an empty-input explain table.
pub fn explain_table_for_empty_chirp() -> String {
    format_explain_rows(&[])
}

/// Formats the per-token explain table for input text.
///
/// # Errors
///
/// Returns an error if text cannot be tokenized or mapped with `VOICE_V1`.
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
                    "{} │ {:+.3} │ {:+.3} │ {:+.3} │ {:+.3} │ role:{}",
                    token.token(),
                    knobs.pitch_center(),
                    knobs.vowel_position(),
                    knobs.contour(),
                    knobs.warble_depth(),
                    role_name(token.role()),
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
            ExplainRow::Hesitation(hesitation) => {
                writeln!(
                    table,
                    "{} │ control:{} │ - │ - │ -",
                    hesitation.token(),
                    hesitation_name(hesitation.marker()),
                )
                .expect("writing to a String cannot fail");
            }
        }
    }

    table
}

fn role_name(role: PhraseRole) -> &'static str {
    match role {
        PhraseRole::Probe => "probe",
        PhraseRole::ChattyReply => "chatty-reply",
        PhraseRole::Hesitation => "hesitation",
        PhraseRole::TerminalFlourish => "terminal-flourish",
        PhraseRole::Aside => "aside",
    }
}

fn hesitation_name(marker: HesitationMarker) -> &'static str {
    match marker {
        HesitationMarker::Dash => "hesitation-dash",
        HesitationMarker::Ellipsis => "hesitation-ellipsis",
    }
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
