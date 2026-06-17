//! Explanation table formatting for the command-line shell.
//!
//! `--explain` is meant to be a *complete* account of why an utterance sounds
//! the way it does. It surfaces every channel that affects the rendered
//! samples: the utterance-level mood and complexity, and, per voiced token, the
//! four semantic knobs, the discourse role, the gesture archetype, the
//! planner's continuous performance curves, and any deployed timing (turn gaps
//! / staged rests). Control markers (punctuation, dash/ellipsis) show the glide
//! and pause they impose.

use std::fmt::Write as _;

use dootdoot_core::{
    EngineError, ExplainRow, GestureArchetype, HesitationMarker, PhraseRole, ProsodicPunctuation,
    SYNTH_SAMPLE_RATE_HZ, explain_rows_for_text,
};

/// Formats an empty-input explain table.
pub fn explain_table_for_empty_chirp() -> String {
    format_explain_rows(&[])
}

/// Formats the full per-token sound profile for input text.
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

    write_summary(&mut table, rows);
    write_main_table(&mut table, rows, token_width);
    write_curves_table(&mut table, rows, token_width);

    table
}

fn write_summary(table: &mut String, rows: &[ExplainRow]) {
    let mut wrote = false;

    for row in rows {
        match row {
            ExplainRow::Mood(mood) => {
                let mood = mood.mood();

                writeln!(
                    table,
                    "mood        valence:{:+.3}  arousal:{:+.3}",
                    mood.valence(),
                    mood.arousal(),
                )
                .expect("writing to a String cannot fail");
                wrote = true;
            }
            ExplainRow::Complexity(complexity) => {
                let complexity = complexity.complexity();

                writeln!(
                    table,
                    "complexity  scalar:{:+.3}  subtokens:{}  chars:{}",
                    complexity.scalar(),
                    complexity.wordpiece_subtoken_count(),
                    complexity.character_count(),
                )
                .expect("writing to a String cannot fail");
                wrote = true;
            }
            _ => {}
        }
    }

    if wrote {
        table.push('\n');
    }
}

fn write_main_table(table: &mut String, rows: &[ExplainRow], token_width: usize) {
    writeln!(
        table,
        "{:<token_width$} │ {:>6} │ {:>6} │ {:>7} │ {:>6} │ {:<17} │ archetype",
        "token", "pitch", "vowel", "contour", "warble", "role",
    )
    .expect("writing to a String cannot fail");

    for row in rows {
        match row {
            ExplainRow::Token(token) => {
                let knobs = token.knobs();

                writeln!(
                    table,
                    "{:<token_width$} │ {:>+6.3} │ {:>+6.3} │ {:>+7.3} │ {:>+6.3} │ {:<17} │ {}",
                    token.token(),
                    knobs.pitch_center(),
                    knobs.vowel_position(),
                    knobs.contour(),
                    knobs.warble_depth(),
                    role_name(token.role()),
                    archetype_name(token.archetype()),
                )
                .expect("writing to a String cannot fail");
            }
            ExplainRow::Punctuation(punctuation) => {
                writeln!(
                    table,
                    "{:<token_width$} │ {}",
                    punctuation.token(),
                    punctuation_effect(punctuation.punctuation()),
                )
                .expect("writing to a String cannot fail");
            }
            ExplainRow::Hesitation(hesitation) => {
                writeln!(
                    table,
                    "{:<token_width$} │ {}",
                    hesitation.token(),
                    hesitation_effect(hesitation.marker()),
                )
                .expect("writing to a String cannot fail");
            }
            ExplainRow::Mood(_) | ExplainRow::Complexity(_) => {}
        }
    }
}

fn write_curves_table(table: &mut String, rows: &[ExplainRow], token_width: usize) {
    let has_tokens = rows.iter().any(|row| matches!(row, ExplainRow::Token(_)));

    if !has_tokens {
        return;
    }

    table.push('\n');
    writeln!(
        table,
        "{:<token_width$} │ {:>6} │ {:>6} │ {:>6} │ {:>6} │ {:>6} │ {:>6} │ {:>6} │ gap",
        "curves", "p.bias", "p.vel", "f.tgt", "f.vel", "bright", "mouth", "tens",
    )
    .expect("writing to a String cannot fail");

    for row in rows {
        if let ExplainRow::Token(token) = row {
            let curves = token.curves();

            writeln!(
                table,
                "{:<token_width$} │ {:>+6.3} │ {:>+6.3} │ {:>+6.3} │ {:>+6.3} │ {:>6.3} │ {:>6.3} │ {:>6.3} │ {}",
                token.token(),
                curves.pitch_center_bias(),
                curves.pitch_velocity(),
                curves.formant_target(),
                curves.formant_velocity(),
                curves.brightness_pressure(),
                curves.mouth_openness(),
                curves.archetype_tension(),
                gap_label(token.timing().pause_override()),
            )
            .expect("writing to a String cannot fail");
        }
    }
}

fn explain_token_width(rows: &[ExplainRow]) -> usize {
    let mut width = "token".len().max("curves".len());

    for row in rows {
        let cell = match row {
            ExplainRow::Token(token) => token.token().chars().count(),
            ExplainRow::Punctuation(punctuation) => punctuation.token().chars().count(),
            ExplainRow::Hesitation(hesitation) => hesitation.token().chars().count(),
            ExplainRow::Mood(_) | ExplainRow::Complexity(_) => 0,
        };

        width = width.max(cell);
    }

    width
}

fn gap_label(pause_override: Option<u32>) -> String {
    pause_override.map_or_else(
        || "-".to_owned(),
        |samples| format!("{} ms", pause_ms(samples)),
    )
}

fn punctuation_effect(punctuation: ProsodicPunctuation) -> String {
    let pause = pause_ms(punctuation.pause_samples());

    match punctuation_glide(punctuation) {
        Some(glide) => format!(
            "control:{} · {glide} · pause {pause} ms",
            punctuation_name(punctuation),
        ),
        None => format!(
            "control:{} · pause {pause} ms",
            punctuation_name(punctuation)
        ),
    }
}

fn hesitation_effect(marker: HesitationMarker) -> String {
    format!(
        "{} · quiet rest {} ms",
        hesitation_name(marker),
        pause_ms(marker.pause_samples()),
    )
}

fn punctuation_glide(punctuation: ProsodicPunctuation) -> Option<&'static str> {
    match punctuation {
        ProsodicPunctuation::Question => Some("rising glide"),
        ProsodicPunctuation::Period | ProsodicPunctuation::Exclamation => Some("falling glide"),
        ProsodicPunctuation::Comma
        | ProsodicPunctuation::Semicolon
        | ProsodicPunctuation::Colon => Some("continuation rise"),
    }
}

fn pause_ms(samples: u32) -> u64 {
    (u64::from(samples) * 1_000) / u64::from(SYNTH_SAMPLE_RATE_HZ)
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

fn archetype_name(archetype: GestureArchetype) -> &'static str {
    match archetype {
        GestureArchetype::Chatter => "chatter",
        GestureArchetype::Yelp => "yelp",
        GestureArchetype::Moan => "moan",
        GestureArchetype::StutterBurst => "stutter-burst",
        GestureArchetype::Tremble => "tremble",
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
