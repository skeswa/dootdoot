//! Deterministic affect analysis for `FORMAT_V2` mood planning.

use crate::{
    TokenizedToken, TokenizerError, complexity::analyze_complexity_for_tokens, embedded_tokenizer,
};

const VADER_VALENCE: &str = include_str!("../../assets/affect/vader_valence.tsv");
const AROUSAL_SIGNALS: &str = include_str!("../../assets/affect/arousal_signals.toml");
const VADER_MAX_ABS_VALENCE: f64 = 4.0;
const BASE_AROUSAL: f64 = 0.10;
const MARKER_WEIGHT: f64 = 0.10;
const QUESTION_WEIGHT: f64 = 0.12;
const EXCLAMATION_WEIGHT: f64 = 0.18;
const COMMA_CLAUSE_WEIGHT: f64 = 0.05;
const PUNCTUATION_MAX: f64 = 0.45;
const REPEATED_MARKER_WEIGHT: f64 = 0.08;
const REPEATED_MARKER_MAX: f64 = 0.24;
const ALL_CAPS_WORD_WEIGHT: f64 = 0.12;
const ALL_CAPS_MIN_WORD_LEN: usize = 2;
const ALL_CAPS_MAX: f64 = 0.36;
const BOOSTER_WEIGHT: f64 = 0.16;
const DAMPENER_WEIGHT: f64 = -0.10;
const TOKEN_COUNT_WEIGHT: f64 = 0.012;
const TOKEN_COUNT_MAX: f64 = 0.20;
const VALENCE_AROUSAL_WEIGHT: f64 = 0.20;
const BOOSTERS: &[&str] = &[
    "absolutely",
    "awfully",
    "completely",
    "deeply",
    "especially",
    "extremely",
    "incredibly",
    "really",
    "so",
    "totally",
    "truly",
    "very",
];
const DAMPENERS: &[&str] = &[
    "barely", "hardly", "kinda", "little", "maybe", "mildly", "partly", "slightly", "somewhat",
];
const DAMPENER_PHRASES: &[&str] = &["kind of"];

/// Gives deterministic affect scores for an utterance.
#[derive(Debug, Clone, PartialEq)]
pub struct AffectAnalysis {
    token_scores: Vec<AffectTokenScore>,
    mood: UtteranceMood,
}

/// Gives one token's affect contribution.
#[derive(Debug, Clone, PartialEq)]
pub struct AffectTokenScore {
    token: String,
    valence: f64,
    arousal: f64,
}

/// Gives pooled utterance-level valence and arousal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UtteranceMood {
    valence: f64,
    arousal: f64,
}

impl AffectAnalysis {
    /// Returns per-token affect scores.
    pub fn token_scores(&self) -> &[AffectTokenScore] {
        &self.token_scores
    }

    /// Returns the pooled utterance mood.
    pub fn mood(&self) -> UtteranceMood {
        self.mood
    }
}

impl AffectTokenScore {
    /// Returns the tokenizer text for this affect row.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns normalized valence in `[-1, 1]`.
    pub fn valence(&self) -> f64 {
        self.valence
    }

    /// Returns this token's local arousal proxy in `[0, 1]`.
    pub fn arousal(&self) -> f64 {
        self.arousal
    }
}

impl UtteranceMood {
    /// Builds a mood from already normalized values.
    pub fn new(valence: f64, arousal: f64) -> Self {
        Self {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
        }
    }

    /// Builds the neutral zero-valence, low-arousal mood.
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            arousal: BASE_AROUSAL,
        }
    }

    /// Returns pooled normalized valence in `[-1, 1]`.
    pub fn valence(&self) -> f64 {
        self.valence
    }

    /// Returns pooled arousal in `[0, 1]`.
    pub fn arousal(&self) -> f64 {
        self.arousal
    }
}

/// Analyzes deterministic affect signals for text.
///
/// # Errors
///
/// Returns an error if the embedded tokenizer cannot process the input.
pub fn analyze_affect_for_text(text: &str) -> Result<AffectAnalysis, TokenizerError> {
    let tokenizer = embedded_tokenizer()?;
    let encoded_input = tokenizer.tokenize(text)?;
    let token_scores = encoded_input
        .tokens()
        .iter()
        .map(score_token)
        .collect::<Vec<_>>();
    let valence = pooled_valence(&token_scores);
    let arousal = pooled_arousal(text, encoded_input.tokens(), valence);

    debug_assert!(AROUSAL_SIGNALS.contains("[punctuation]"));

    Ok(AffectAnalysis {
        token_scores,
        mood: UtteranceMood::new(valence, arousal),
    })
}

fn score_token(token: &TokenizedToken) -> AffectTokenScore {
    let token_text = token.text();
    let normalized = token_text
        .strip_prefix("##")
        .unwrap_or(token_text)
        .to_lowercase();
    let valence = lookup_vader_valence(&normalized).unwrap_or(0.0) / VADER_MAX_ABS_VALENCE;

    AffectTokenScore {
        token: token_text.to_owned(),
        valence,
        arousal: valence.abs() * VALENCE_AROUSAL_WEIGHT,
    }
}

fn lookup_vader_valence(term: &str) -> Option<f64> {
    for line in VADER_VALENCE.lines() {
        let (entry, score) = line.split_once('\t')?;

        if entry == term {
            return score.parse::<f64>().ok();
        }
    }

    None
}

fn pooled_valence(token_scores: &[AffectTokenScore]) -> f64 {
    let mut total = 0.0;
    let mut count = 0_u32;

    for score in token_scores {
        if score.valence != 0.0 {
            total += score.valence;
            count = count.saturating_add(1);
        }
    }

    if count == 0 {
        0.0
    } else {
        (total / f64::from(count)).clamp(-1.0, 1.0)
    }
}

fn pooled_arousal(text: &str, tokens: &[TokenizedToken], valence: f64) -> f64 {
    let punctuation = punctuation_arousal(text);
    let repeated_markers = repeated_marker_arousal(text);
    let all_caps = all_caps_arousal(text);
    let intensifiers = intensifier_arousal(text);
    let token_count = (usize_to_f64(tokens.len()) * TOKEN_COUNT_WEIGHT).min(TOKEN_COUNT_MAX);
    let complexity = analyze_complexity_for_tokens(text, tokens).arousal_contribution();
    let valence_energy = valence.abs() * VALENCE_AROUSAL_WEIGHT;

    (BASE_AROUSAL
        + punctuation
        + repeated_markers
        + all_caps
        + intensifiers
        + token_count
        + complexity
        + valence_energy)
        .clamp(0.0, 1.0)
}

fn punctuation_arousal(text: &str) -> f64 {
    let mut score = 0.0;

    for character in text.chars() {
        score += match character {
            '?' => QUESTION_WEIGHT,
            '!' => EXCLAMATION_WEIGHT,
            ',' | ';' | ':' => COMMA_CLAUSE_WEIGHT,
            '.' => MARKER_WEIGHT,
            _ => 0.0,
        };
    }

    score.min(PUNCTUATION_MAX)
}

fn repeated_marker_arousal(text: &str) -> f64 {
    let mut score = 0.0;
    let mut previous = '\0';
    let mut run_len = 0_u32;

    for character in text.chars() {
        if matches!(character, '?' | '!' | '.' | ',' | ';' | ':') && character == previous {
            run_len = run_len.saturating_add(1);
            score += REPEATED_MARKER_WEIGHT;
        } else {
            run_len = 0;
        }

        previous = character;

        if run_len > 0 && score >= REPEATED_MARKER_MAX {
            return REPEATED_MARKER_MAX;
        }
    }

    score.min(REPEATED_MARKER_MAX)
}

fn all_caps_arousal(text: &str) -> f64 {
    let mut score = 0.0;

    for word in text.split_whitespace() {
        let letters = word.chars().filter(|character| character.is_alphabetic());
        let mut letter_count = 0_usize;
        let mut uppercase_count = 0_usize;

        for letter in letters {
            letter_count = letter_count.saturating_add(1);
            if letter.is_uppercase() {
                uppercase_count = uppercase_count.saturating_add(1);
            }
        }

        if letter_count >= ALL_CAPS_MIN_WORD_LEN && letter_count == uppercase_count {
            score += ALL_CAPS_WORD_WEIGHT;
        }
    }

    score.min(ALL_CAPS_MAX)
}

fn intensifier_arousal(text: &str) -> f64 {
    let normalized_words = normalized_words(text);
    let mut score = 0.0;

    for word in &normalized_words {
        if BOOSTERS.contains(&word.as_str()) {
            score += BOOSTER_WEIGHT;
        } else if DAMPENERS.contains(&word.as_str()) {
            score += DAMPENER_WEIGHT;
        }
    }

    let normalized_text = normalized_words.join(" ");
    for phrase in DAMPENER_PHRASES {
        if normalized_text.contains(phrase) {
            score += DAMPENER_WEIGHT;
        }
    }

    score
}

fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|character| character.is_alphanumeric() || *character == '\'')
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

fn usize_to_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
