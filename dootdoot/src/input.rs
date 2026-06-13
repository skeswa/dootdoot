//! Input resolution for CLI text and stdin.

use std::io::{self, IsTerminal, Read};

use crate::Cli;

/// Gives stdin state to the pure input resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdinInput<'a> {
    /// Stdin is interactive and should not be read as text input.
    Terminal,
    /// Stdin was piped and contains text.
    Piped(&'a str),
}

/// Gives resolved text input or the empty-chirp route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedInput {
    /// Non-empty text to synthesize.
    Text(String),
    /// Empty, whitespace-only, or absent input should synthesize the fixed
    /// chirp.
    EmptyChirp,
}

impl ResolvedInput {
    /// Returns true when input should route to the fixed empty chirp.
    pub fn is_empty_chirp(&self) -> bool {
        matches!(self, Self::EmptyChirp)
    }

    /// Returns non-empty resolved text when present.
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::EmptyChirp => None,
        }
    }
}

/// Resolves positional text versus stdin fallback.
pub fn resolve_input(cli: &Cli, stdin: StdinInput<'_>) -> ResolvedInput {
    if let Some(text) = cli.text.as_deref() {
        return normalize_text(text);
    }

    match stdin {
        StdinInput::Terminal => ResolvedInput::EmptyChirp,
        StdinInput::Piped(text) => normalize_text(text),
    }
}

/// Reads stdin when needed and resolves the effective input.
///
/// # Errors
///
/// Returns an error if piped stdin cannot be read.
pub fn read_resolved_input(cli: &Cli) -> io::Result<ResolvedInput> {
    if cli.text.is_some() {
        return Ok(resolve_input(cli, StdinInput::Terminal));
    }

    let mut stdin = io::stdin();

    if stdin.is_terminal() {
        return Ok(resolve_input(cli, StdinInput::Terminal));
    }

    let mut text = String::new();
    stdin.read_to_string(&mut text)?;

    Ok(resolve_input(cli, StdinInput::Piped(&text)))
}

fn normalize_text(text: &str) -> ResolvedInput {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        ResolvedInput::EmptyChirp
    } else {
        ResolvedInput::Text(trimmed.to_owned())
    }
}
