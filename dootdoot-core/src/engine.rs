//! End-to-end text rendering pipeline.

use thiserror::Error;

use crate::{
    MappingError, ProsodicPunctuation, SequenceEvent, TokenizerError, assemble_knobs,
    embedded_mapping, embedded_tokenizer, pool_sequence, render_canonical_buffer,
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
    Punctuation(ProsodicPunctuation),
}

/// Converts text into sequencer events.
///
/// # Errors
///
/// Returns an error if the embedded tokenizer or mapping cannot process the
/// input.
pub fn sequence_events_for_text(text: &str) -> Result<Vec<SequenceEvent>, EngineError> {
    let tokenizer = embedded_tokenizer()?;
    let mapping = embedded_mapping()?;
    let encoded_input = tokenizer.tokenize(text)?;
    let mut templates = Vec::new();
    let mut voiced_tokens = Vec::new();

    for token in encoded_input.tokens() {
        if let Some(punctuation) = ProsodicPunctuation::from_text(token.text()) {
            templates.push(EventTemplate::Punctuation(punctuation));
        } else {
            let token_vector = mapping.lookup(token.id())?;

            templates.push(EventTemplate::Voiced(voiced_tokens.len()));
            voiced_tokens.push((token_vector, token.is_continuation()));
        }
    }

    if voiced_tokens.is_empty() {
        return Ok(templates
            .into_iter()
            .filter_map(|template| match template {
                EventTemplate::Punctuation(punctuation) => {
                    Some(SequenceEvent::punctuation(punctuation))
                }
                EventTemplate::Voiced(_) => None,
            })
            .collect());
    }

    let token_vectors = voiced_tokens
        .iter()
        .map(|(token_vector, _continuation)| *token_vector)
        .collect::<Vec<_>>();
    let baseline = mapping.squash_pooled(pool_sequence(&token_vectors)?);
    let squashed_tokens = token_vectors
        .iter()
        .copied()
        .map(|token_vector| mapping.squash_token(token_vector))
        .collect::<Vec<_>>();

    Ok(templates
        .into_iter()
        .map(|template| match template {
            EventTemplate::Punctuation(punctuation) => SequenceEvent::punctuation(punctuation),
            EventTemplate::Voiced(index) => SequenceEvent::syllable(
                assemble_knobs(baseline, squashed_tokens[index]),
                voiced_tokens[index].1,
            ),
        })
        .collect())
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
