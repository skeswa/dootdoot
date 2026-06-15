//! End-to-end text rendering pipeline.

use thiserror::Error;

use crate::{
    KnobSet, MappingError, ProsodicPunctuation, SequenceEvent, TokenVector, TokenizerError,
    UtteranceMood, analyze_affect_for_text, analyze_complexity_for_text, assemble_knobs,
    embedded_mapping, embedded_tokenizer, plan_gesture_archetypes, pool_sequence,
    render_canonical_buffer,
};

/// Reports why text could not be rendered.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Tokenization failed.
    #[error("{0}")]
    Tokenizer(#[from] TokenizerError),
    /// Semantic mapping failed.
    #[error("{0}")]
    Mapping(#[from] MappingError),
}

#[derive(Debug, Clone, Copy)]
enum EventTemplate {
    Voiced(usize),
    Punctuation(usize),
}

#[derive(Debug, Clone)]
struct VoicedToken {
    text: String,
    vector: TokenVector,
    continuation: bool,
}

#[derive(Debug, Clone)]
struct PunctuationToken {
    text: String,
    punctuation: ProsodicPunctuation,
}

#[derive(Debug, Clone)]
struct TextAnalysis {
    events: Vec<SequenceEvent>,
    explain_rows: Vec<ExplainRow>,
}

/// Gives one row in the `--explain` table.
#[derive(Debug, Clone, PartialEq)]
pub enum ExplainRow {
    /// A whole-utterance mood row.
    Mood(ExplainMoodRow),
    /// A voiced token row with semantic knobs.
    Token(ExplainTokenRow),
    /// A control-only prosodic punctuation row.
    Punctuation(ExplainPunctuationRow),
}

/// Gives one voiced token row in the `--explain` table.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainTokenRow {
    token: String,
    knobs: KnobSet,
    continuation: bool,
}

/// Gives one prosodic punctuation row in the `--explain` table.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainPunctuationRow {
    token: String,
    punctuation: ProsodicPunctuation,
}

/// Gives one whole-utterance mood row in the `--explain` table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplainMoodRow {
    mood: UtteranceMood,
}

/// Converts text into sequencer events.
///
/// # Errors
///
/// Returns an error if the embedded tokenizer or mapping cannot process the
/// input.
pub fn sequence_events_for_text(text: &str) -> Result<Vec<SequenceEvent>, EngineError> {
    Ok(analyze_text(text)?.events)
}

/// Converts text into `--explain` rows.
///
/// # Errors
///
/// Returns an error if the embedded tokenizer or mapping cannot process the
/// input.
pub fn explain_rows_for_text(text: &str) -> Result<Vec<ExplainRow>, EngineError> {
    Ok(analyze_text(text)?.explain_rows)
}

fn analyze_text(text: &str) -> Result<TextAnalysis, EngineError> {
    let tokenizer = embedded_tokenizer()?;
    let mapping = embedded_mapping()?;
    let mood = analyze_affect_for_text(text)?.mood();
    let complexity = analyze_complexity_for_text(text)?;
    let encoded_input = tokenizer.tokenize(text)?;
    let mut templates = Vec::new();
    let mut voiced_tokens = Vec::new();
    let mut punctuation_tokens = Vec::new();

    for token in encoded_input.tokens() {
        if let Some(punctuation) = ProsodicPunctuation::from_text(token.text()) {
            templates.push(EventTemplate::Punctuation(punctuation_tokens.len()));
            punctuation_tokens.push(PunctuationToken {
                text: token.text().to_owned(),
                punctuation,
            });
        } else {
            let token_vector = mapping.lookup(token.id())?;

            templates.push(EventTemplate::Voiced(voiced_tokens.len()));
            voiced_tokens.push(VoicedToken {
                text: token.text().to_owned(),
                vector: token_vector,
                continuation: token.is_continuation(),
            });
        }
    }

    if voiced_tokens.is_empty() {
        let mut events = vec![
            SequenceEvent::mood(mood),
            SequenceEvent::complexity(complexity),
        ];
        let mut explain_rows = vec![ExplainRow::mood(mood)];

        for template in templates {
            match template {
                EventTemplate::Punctuation(index) => {
                    let punctuation = &punctuation_tokens[index];

                    events.push(SequenceEvent::punctuation(punctuation.punctuation));
                    explain_rows.push(ExplainRow::punctuation(
                        punctuation.text.clone(),
                        punctuation.punctuation,
                    ));
                }
                EventTemplate::Voiced(_) => {}
            }
        }

        return Ok(TextAnalysis {
            events: with_archetype_events(events),
            explain_rows,
        });
    }

    let token_vectors = voiced_tokens
        .iter()
        .map(|token| token.vector)
        .collect::<Vec<_>>();
    let baseline = mapping.squash_pooled(pool_sequence(&token_vectors)?);
    let squashed_tokens = token_vectors
        .iter()
        .copied()
        .map(|token_vector| mapping.squash_token(token_vector))
        .collect::<Vec<_>>();
    let mut events = vec![
        SequenceEvent::mood(mood),
        SequenceEvent::complexity(complexity),
    ];
    let mut explain_rows = vec![ExplainRow::mood(mood)];

    for template in templates {
        match template {
            EventTemplate::Punctuation(index) => {
                let punctuation = &punctuation_tokens[index];

                events.push(SequenceEvent::punctuation(punctuation.punctuation));
                explain_rows.push(ExplainRow::punctuation(
                    punctuation.text.clone(),
                    punctuation.punctuation,
                ));
            }
            EventTemplate::Voiced(index) => {
                let token = &voiced_tokens[index];
                let knobs = assemble_knobs(baseline, squashed_tokens[index]);

                events.push(SequenceEvent::syllable(knobs, token.continuation));
                explain_rows.push(ExplainRow::token(
                    token.text.clone(),
                    knobs,
                    token.continuation,
                ));
            }
        }
    }

    Ok(TextAnalysis {
        events: with_archetype_events(events),
        explain_rows,
    })
}

/// Renders text into the canonical signed 16-bit mono audio buffer.
///
/// # Errors
///
/// Returns an error if text cannot be converted into sequencer events.
pub fn render_text_canonical_buffer(text: &str) -> Result<Vec<i16>, EngineError> {
    let events = sequence_events_for_text(text)?;

    Ok(render_canonical_buffer(&events))
}

impl ExplainRow {
    fn mood(mood: UtteranceMood) -> Self {
        Self::Mood(ExplainMoodRow { mood })
    }

    fn token(token: String, knobs: KnobSet, continuation: bool) -> Self {
        Self::Token(ExplainTokenRow {
            token,
            knobs,
            continuation,
        })
    }

    fn punctuation(token: String, punctuation: ProsodicPunctuation) -> Self {
        Self::Punctuation(ExplainPunctuationRow { token, punctuation })
    }
}

fn with_archetype_events(events: Vec<SequenceEvent>) -> Vec<SequenceEvent> {
    let archetypes = plan_gesture_archetypes(&events);
    let mut archetype_index = 0_usize;
    let mut output = Vec::with_capacity(events.len() + archetypes.len());

    for event in events {
        if matches!(event, SequenceEvent::Syllable(_)) {
            if let Some(archetype) = archetypes.get(archetype_index).copied() {
                output.push(SequenceEvent::archetype(archetype));
            }

            archetype_index = archetype_index.saturating_add(1);
        }

        output.push(event);
    }

    output
}

impl ExplainMoodRow {
    /// Returns the utterance mood for this control row.
    pub fn mood(&self) -> UtteranceMood {
        self.mood
    }
}

impl ExplainTokenRow {
    /// Returns the tokenizer text for this voiced row.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the semantic knobs for this voiced row.
    pub fn knobs(&self) -> KnobSet {
        self.knobs
    }

    /// Returns true when this token is a `WordPiece` continuation.
    pub fn is_continuation(&self) -> bool {
        self.continuation
    }
}

impl ExplainPunctuationRow {
    /// Returns the tokenizer text for this control row.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the prosodic punctuation marker for this control row.
    pub fn punctuation(&self) -> ProsodicPunctuation {
        self.punctuation
    }
}
