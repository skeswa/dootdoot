//! End-to-end text rendering pipeline.

use thiserror::Error;

use crate::{
    ComplexityAnalysis, HesitationMarker, KnobSet, MappingError, PerformanceCurves,
    PerformanceSyllable, PhraseRole, ProsodicPunctuation, SequenceEvent, SyllableEvent,
    SyllableTiming, TokenVector, TokenizerError, UtteranceMood, analyze_affect_for_text,
    analyze_complexity_for_text, archetype_for_role, assemble_knobs, embedded_mapping,
    embedded_tokenizer, plan_discourse_performance, plan_gesture_archetypes, pool_sequence,
    render_canonical_buffer, role_long_pause_samples, staged_reply_rest_samples,
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
    Hesitation(usize),
}

#[derive(Debug, Clone)]
struct VoicedToken {
    text: String,
    vector: TokenVector,
    continuation: bool,
    timing: SyllableTiming,
}

#[derive(Debug, Clone)]
struct PunctuationToken {
    text: String,
    punctuation: ProsodicPunctuation,
}

#[derive(Debug, Clone)]
struct HesitationToken {
    text: String,
    marker: HesitationMarker,
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
    /// A control-only hesitation marker row.
    Hesitation(ExplainHesitationRow),
}

/// Gives one voiced token row in the `--explain` table.
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainTokenRow {
    token: String,
    knobs: KnobSet,
    continuation: bool,
    role: PhraseRole,
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

/// Gives one control-only hesitation marker row in the `--explain` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainHesitationRow {
    token: String,
    marker: HesitationMarker,
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

#[derive(Debug, Default)]
struct ParsedTokens {
    templates: Vec<EventTemplate>,
    voiced_tokens: Vec<VoicedToken>,
    punctuation_tokens: Vec<PunctuationToken>,
    hesitation_tokens: Vec<HesitationToken>,
}

fn analyze_text(text: &str) -> Result<TextAnalysis, EngineError> {
    let tokenizer = embedded_tokenizer()?;
    let mapping = embedded_mapping()?;
    let mood = analyze_affect_for_text(text)?.mood();
    let complexity = analyze_complexity_for_text(text)?;
    let encoded_input = tokenizer.tokenize(text)?;
    let parsed = parse_tokens(encoded_input.tokens(), &mapping)?;

    if parsed.voiced_tokens.is_empty() {
        Ok(empty_analysis(mood, complexity, &parsed))
    } else {
        voiced_analysis(mood, complexity, &parsed, &mapping)
    }
}

fn parse_tokens(
    tokens: &[crate::TokenizedToken],
    mapping: &crate::Mapping,
) -> Result<ParsedTokens, EngineError> {
    let mut parsed = ParsedTokens::default();
    let mut last_voiced_index: Option<usize> = None;

    for token in tokens {
        if let Some(punctuation) = ProsodicPunctuation::from_text(token.text()) {
            parsed
                .templates
                .push(EventTemplate::Punctuation(parsed.punctuation_tokens.len()));
            parsed.punctuation_tokens.push(PunctuationToken {
                text: token.text().to_owned(),
                punctuation,
            });
        } else if let Some(marker) = HesitationMarker::from_text(token.text()) {
            parsed
                .templates
                .push(EventTemplate::Hesitation(parsed.hesitation_tokens.len()));
            parsed.hesitation_tokens.push(HesitationToken {
                text: token.text().to_owned(),
                marker,
            });

            if let Some(index) = last_voiced_index {
                parsed.voiced_tokens[index].timing =
                    longer_hesitation(parsed.voiced_tokens[index].timing, marker.timing());
            }
        } else {
            let token_vector = mapping.lookup(token.id())?;

            last_voiced_index = Some(parsed.voiced_tokens.len());
            parsed
                .templates
                .push(EventTemplate::Voiced(parsed.voiced_tokens.len()));
            parsed.voiced_tokens.push(VoicedToken {
                text: token.text().to_owned(),
                vector: token_vector,
                continuation: token.is_continuation(),
                timing: SyllableTiming::default(),
            });
        }
    }

    Ok(parsed)
}

fn empty_analysis(
    mood: UtteranceMood,
    complexity: ComplexityAnalysis,
    parsed: &ParsedTokens,
) -> TextAnalysis {
    let mut events = vec![
        SequenceEvent::mood(mood),
        SequenceEvent::complexity(complexity),
    ];
    let mut explain_rows = vec![ExplainRow::mood(mood)];

    for template in &parsed.templates {
        match template {
            EventTemplate::Punctuation(index) => {
                let punctuation = &parsed.punctuation_tokens[*index];

                events.push(SequenceEvent::punctuation(punctuation.punctuation));
                explain_rows.push(ExplainRow::punctuation(
                    punctuation.text.clone(),
                    punctuation.punctuation,
                ));
            }
            EventTemplate::Hesitation(index) => {
                let hesitation = &parsed.hesitation_tokens[*index];

                explain_rows.push(ExplainRow::hesitation(
                    hesitation.text.clone(),
                    hesitation.marker,
                ));
            }
            EventTemplate::Voiced(_) => {}
        }
    }

    TextAnalysis {
        events: with_archetype_events(events),
        explain_rows,
    }
}

fn voiced_analysis(
    mood: UtteranceMood,
    complexity: ComplexityAnalysis,
    parsed: &ParsedTokens,
    mapping: &crate::Mapping,
) -> Result<TextAnalysis, EngineError> {
    let token_vectors = parsed
        .voiced_tokens
        .iter()
        .map(|token| token.vector)
        .collect::<Vec<_>>();
    let baseline = mapping.squash_pooled(pool_sequence(&token_vectors)?);
    let knobs_per_voiced = token_vectors
        .iter()
        .copied()
        .map(|token_vector| assemble_knobs(baseline, mapping.squash_token(token_vector)))
        .collect::<Vec<_>>();
    let base_events = base_events_for_plan(mood, complexity, parsed, &knobs_per_voiced);
    let plan = plan_discourse_performance(&base_events);
    let plan_rows = plan.syllables();
    let deployed = deploy_performance_timing(&parsed.voiced_tokens, plan_rows);

    let mut events = vec![
        SequenceEvent::mood(mood),
        SequenceEvent::complexity(complexity),
    ];
    let mut explain_rows = vec![ExplainRow::mood(mood)];

    for template in &parsed.templates {
        match template {
            EventTemplate::Punctuation(index) => {
                let punctuation = &parsed.punctuation_tokens[*index];

                events.push(SequenceEvent::punctuation(punctuation.punctuation));
                explain_rows.push(ExplainRow::punctuation(
                    punctuation.text.clone(),
                    punctuation.punctuation,
                ));
            }
            EventTemplate::Hesitation(index) => {
                let hesitation = &parsed.hesitation_tokens[*index];

                explain_rows.push(ExplainRow::hesitation(
                    hesitation.text.clone(),
                    hesitation.marker,
                ));
            }
            EventTemplate::Voiced(index) => {
                let token = &parsed.voiced_tokens[*index];
                let knobs = knobs_per_voiced[*index];
                let role = plan_rows
                    .get(*index)
                    .map_or(PhraseRole::ChattyReply, PerformanceSyllable::role);
                let curves = plan_rows
                    .get(*index)
                    .map_or_else(PerformanceCurves::neutral, PerformanceSyllable::curves);

                events.push(SequenceEvent::archetype(archetype_for_role(role, *index)));
                events.push(SequenceEvent::Syllable(
                    SyllableEvent::new(knobs, token.continuation)
                        .with_timing(deployed[*index])
                        .with_performance(role, curves),
                ));
                explain_rows.push(ExplainRow::token(
                    token.text.clone(),
                    knobs,
                    token.continuation,
                    role,
                ));
            }
        }
    }

    Ok(TextAnalysis {
        events,
        explain_rows,
    })
}

fn base_events_for_plan(
    mood: UtteranceMood,
    complexity: ComplexityAnalysis,
    parsed: &ParsedTokens,
    knobs_per_voiced: &[KnobSet],
) -> Vec<SequenceEvent> {
    let mut base_events = vec![
        SequenceEvent::mood(mood),
        SequenceEvent::complexity(complexity),
    ];

    for template in &parsed.templates {
        match template {
            EventTemplate::Punctuation(index) => {
                base_events.push(SequenceEvent::punctuation(
                    parsed.punctuation_tokens[*index].punctuation,
                ));
            }
            EventTemplate::Voiced(index) => {
                base_events.push(SequenceEvent::syllable_with_timing(
                    knobs_per_voiced[*index],
                    parsed.voiced_tokens[*index].continuation,
                    parsed.voiced_tokens[*index].timing,
                ));
            }
            EventTemplate::Hesitation(_) => {}
        }
    }

    base_events
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

    fn token(token: String, knobs: KnobSet, continuation: bool, role: PhraseRole) -> Self {
        Self::Token(ExplainTokenRow {
            token,
            knobs,
            continuation,
            role,
        })
    }

    fn punctuation(token: String, punctuation: ProsodicPunctuation) -> Self {
        Self::Punctuation(ExplainPunctuationRow { token, punctuation })
    }

    fn hesitation(token: String, marker: HesitationMarker) -> Self {
        Self::Hesitation(ExplainHesitationRow { token, marker })
    }
}

fn longer_hesitation(current: SyllableTiming, marker: SyllableTiming) -> SyllableTiming {
    let current_pause = current.pause_override().unwrap_or(0);
    let marker_pause = marker.pause_override().unwrap_or(0);

    if marker_pause >= current_pause {
        marker
    } else {
        current
    }
}

impl ExplainHesitationRow {
    /// Returns the tokenizer text for this hesitation row.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the hesitation marker for this control row.
    pub fn marker(&self) -> HesitationMarker {
        self.marker
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

fn deploy_performance_timing(
    voiced: &[VoicedToken],
    plan: &[PerformanceSyllable],
) -> Vec<SyllableTiming> {
    // Only stage long turn gaps and reply rests when the utterance has real
    // discourse structure (some non-`ChattyReply` role). A plain statement stays
    // on the smooth `VOICE_V3`/`VOICE_V6` connected path.
    let has_structure = plan
        .iter()
        .any(|syllable| syllable.role() != PhraseRole::ChattyReply);
    let mut timings = Vec::with_capacity(voiced.len());

    for index in 0..voiced.len() {
        let base = voiced[index].timing;
        let role = plan
            .get(index)
            .map_or(PhraseRole::ChattyReply, PerformanceSyllable::role);
        let next_role = plan.get(index + 1).map(PerformanceSyllable::role);
        let next_continuation = voiced.get(index + 1).map(|token| token.continuation);

        timings.push(deploy_one_timing(
            base,
            role,
            next_role,
            next_continuation,
            has_structure,
        ));
    }

    timings
}

fn deploy_one_timing(
    base: SyllableTiming,
    role: PhraseRole,
    next_role: Option<PhraseRole>,
    next_continuation: Option<bool>,
    has_structure: bool,
) -> SyllableTiming {
    match role {
        PhraseRole::Probe | PhraseRole::Hesitation => {
            if next_role.is_some_and(|next| next != role) {
                let turn = role_long_pause_samples(0.55);
                let pause = base.pause_override().unwrap_or(0).max(turn);

                base.with_pause_override(pause).suppress_bridge()
            } else {
                base
            }
        }
        PhraseRole::ChattyReply => {
            if has_structure
                && next_role == Some(PhraseRole::ChattyReply)
                && next_continuation == Some(false)
                && base.pause_override().is_none()
            {
                base.with_pause_override(staged_reply_rest_samples(0.5))
                    .suppress_bridge()
            } else {
                base
            }
        }
        PhraseRole::TerminalFlourish | PhraseRole::Aside => base,
    }
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

    /// Returns the discourse role assigned to this voiced row.
    pub fn role(&self) -> PhraseRole {
        self.role
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
