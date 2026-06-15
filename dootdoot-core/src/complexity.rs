//! Deterministic `FORMAT_V2` lexical complexity analysis.

use crate::{TokenizedToken, TokenizerError, embedded_tokenizer};

const WORDPIECE_COMPLEXITY_WEIGHT: f64 = 0.04;
const CHARACTER_COMPLEXITY_WEIGHT: f64 = 0.006;
const COMPLEXITY_AROUSAL_MAX: f64 = 0.25;

/// Gives deterministic lexical complexity signals for an utterance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexityAnalysis {
    wordpiece_subtoken_count: usize,
    character_count: usize,
    scalar: f64,
}

impl ComplexityAnalysis {
    pub(crate) fn zero() -> Self {
        Self {
            wordpiece_subtoken_count: 0,
            character_count: 0,
            scalar: 0.0,
        }
    }

    /// Returns continuation `WordPiece` subtokens beyond each word's first
    /// piece.
    pub fn wordpiece_subtoken_count(&self) -> usize {
        self.wordpiece_subtoken_count
    }

    /// Returns the non-whitespace input character count.
    pub fn character_count(&self) -> usize {
        self.character_count
    }

    /// Returns a bounded complexity scalar in `[0, 1]`.
    pub fn scalar(&self) -> f64 {
        self.scalar
    }

    pub(crate) fn arousal_contribution(&self) -> f64 {
        self.scalar * COMPLEXITY_AROUSAL_MAX
    }
}

/// Analyzes deterministic lexical complexity signals for text.
///
/// # Errors
///
/// Returns an error if the embedded tokenizer cannot process the input.
pub fn analyze_complexity_for_text(text: &str) -> Result<ComplexityAnalysis, TokenizerError> {
    let tokenizer = embedded_tokenizer()?;
    let encoded_input = tokenizer.tokenize(text)?;

    Ok(analyze_complexity_for_tokens(text, encoded_input.tokens()))
}

pub(crate) fn analyze_complexity_for_tokens(
    text: &str,
    tokens: &[TokenizedToken],
) -> ComplexityAnalysis {
    let wordpiece_subtoken_count = tokens
        .iter()
        .filter(|token| token.is_continuation())
        .count();
    let character_count = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    let contribution = (usize_to_f64(wordpiece_subtoken_count) * WORDPIECE_COMPLEXITY_WEIGHT)
        + (usize_to_f64(character_count) * CHARACTER_COMPLEXITY_WEIGHT);
    let scalar = (contribution / COMPLEXITY_AROUSAL_MAX).clamp(0.0, 1.0);

    ComplexityAnalysis {
        wordpiece_subtoken_count,
        character_count,
        scalar,
    }
}

fn usize_to_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
