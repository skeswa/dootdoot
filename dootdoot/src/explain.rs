//! Explanation table formatting for the command-line shell.

use std::fmt::Write as _;

use dootdoot_core::{
    EngineError, ExplainRow, HesitationMarker, PhraseRole, ProsodicPunctuation,
    explain_rows_for_text,
};

/// Formats an empty-input explain table.
pub fn explain_table_for_empty_chirp() -> String {
    format_explain_rows(&[])
}

/// Formats the per-token explain table for input text.
///
/// # Errors
///
/// Returns an error if text cannot be tokenized or mapped with the active
/// voice.
pub fn explain_table_for_text(text: &str) -> Result<String, EngineError> {
    Ok(format_explain_rows(&explain_rows_for_text(text)?))
}

fn format_explain_rows(rows: &[ExplainRow]) -> String {
    let token_width = explain_token_width(rows);
    let mut table = String::new();

    writeln!(
        table,
        "{:<token_width$} │ {:>6} │ {:>6} │ {:>7} │ {:>6} │ role",
        "token", "pitch", "vowel", "contour", "warble",
    )
    .expect("writing to a String cannot fail");

    for row in rows {
        match row {
            ExplainRow::Mood(mood) => {
                let mood = mood.mood();

                writeln!(
                    table,
                    "{:<token_width$} │ valence:{:+.3}  arousal:{:+.3}",
                    "mood",
                    mood.valence(),
                    mood.arousal(),
                )
            }
            ExplainRow::Token(token) => {
                let knobs = token.knobs();

                writeln!(
                    table,
                    "{:<token_width$} │ {:>+6.3} │ {:>+6.3} │ {:>+7.3} │ {:>+6.3} │ {}",
                    token.token(),
                    knobs.pitch_center(),
                    knobs.vowel_position(),
                    knobs.contour(),
                    knobs.warble_depth(),
                    role_name(token.role()),
                )
            }
            ExplainRow::Punctuation(punctuation) => {
                writeln!(
                    table,
                    "{:<token_width$} │ control:{}",
                    punctuation.token(),
                    punctuation_name(punctuation.punctuation()),
                )
            }
            ExplainRow::Hesitation(hesitation) => {
                writeln!(
                    table,
                    "{:<token_width$} │ control:{}",
                    hesitation.token(),
                    hesitation_name(hesitation.marker()),
                )
            }
        }
        .expect("writing to a String cannot fail");
    }

    table
}

fn explain_token_width(rows: &[ExplainRow]) -> usize {
    let mut width = "token".len().max("mood".len());

    for row in rows {
        let cell = match row {
            ExplainRow::Mood(_) => "mood".chars().count(),
            ExplainRow::Token(token) => token.token().chars().count(),
            ExplainRow::Punctuation(punctuation) => punctuation.token().chars().count(),
            ExplainRow::Hesitation(hesitation) => hesitation.token().chars().count(),
        };

        width = width.max(cell);
    }

    width
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
