//! End-to-end text rendering pipeline.

use thiserror::Error;

use crate::{
    COMPOUND_SYLLABLE_DURATION_SCALE, ComplexityAnalysis, GestureArchetype, HesitationMarker,
    KnobSet, MappingError, PerformanceCurves, PerformanceSyllable, PhraseRole, PosClass,
    ProsodicPunctuation, SequenceEvent, SyllableEvent, SyllableTiming, TokenVector, TokenizerError,
    UtteranceMood, analyze_affect_for_text, analyze_complexity_for_text, archetype_for_role,
    assemble_knobs, class_resolution_knobs, embedded_mapping, embedded_tokenizer,
    plan_discourse_performance, plan_gesture_archetypes, pool_sequence, render_canonical_buffer,
    role_long_pause_samples, staged_reply_rest_samples,
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

/// Gives the staged-reply word-rest amount for structured utterances (~55 ms).
const STAGED_REPLY_REST_AMOUNT: f64 = 0.5;

/// Gives the `VOICE_V8` neutral word-rest amount for plain statements (~40 ms),
/// so unpunctuated multi-word input gets short inter-word rests instead of a
/// fully bridged single island.
const NEUTRAL_WORD_REST_AMOUNT: f64 = 0.2;

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
    pos_class: PosClass,
}

/// Gives one render-ready syllable after `VOICE_V12` compound expansion.
///
/// A single-token content word contributes two of these (stem + derived
/// resolution); everything else contributes exactly one. The semantic baseline
/// stays pooled over the original tokens — resolutions never join the pool.
#[derive(Debug, Clone, Copy)]
struct EngineSyllable {
    knobs: KnobSet,
    continuation: bool,
    timing: SyllableTiming,
    pos_class: PosClass,
    duration_scale: f64,
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
    /// A whole-utterance complexity row.
    Complexity(ExplainComplexityRow),
    /// A voiced token row with its full sound profile.
    Token(ExplainTokenRow),
    /// A control-only prosodic punctuation row.
    Punctuation(ExplainPunctuationRow),
    /// A control-only hesitation marker row.
    Hesitation(ExplainHesitationRow),
}

/// Gives one voiced token row in the `--explain` table.
///
/// Carries the full per-token sound profile: the four semantic knobs plus every
/// `VOICE_V7` performance channel that affects the rendered samples (discourse
/// role, gesture archetype, planner performance curves, and deployed timing).
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainTokenRow {
    token: String,
    knobs: KnobSet,
    continuation: bool,
    role: PhraseRole,
    archetype: GestureArchetype,
    curves: PerformanceCurves,
    timing: SyllableTiming,
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

/// Gives one whole-utterance complexity row in the `--explain` table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplainComplexityRow {
    complexity: ComplexityAnalysis,
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
    let mut index = 0;

    while index < tokens.len() {
        let text = tokens[index].text();

        if ProsodicPunctuation::from_text(text).is_some() {
            // VOICE_V9 (R2): collapse a maximal run of consecutive prosodic
            // punctuation. A run of two or more ASCII periods becomes one
            // trailing-off ellipsis (`...` is how people actually type it);
            // any other run keeps only its first marker. The engine already
            // first-wins on consecutive punctuation (`segment_events`,
            // `pending_syllables`), so collapsing a stacked terminal run
            // (`?!`, `!!!`) only makes that pre-existing behavior honest.
            let start = index;
            while index < tokens.len()
                && ProsodicPunctuation::from_text(tokens[index].text()).is_some()
            {
                index += 1;
            }

            let run = &tokens[start..index];
            let all_periods = run.iter().all(|token| token.text() == ".");

            if run.len() >= 2 && all_periods {
                add_hesitation(
                    &mut parsed,
                    run_text(run),
                    HesitationMarker::Ellipsis,
                    last_voiced_index,
                );
            } else {
                let punctuation = ProsodicPunctuation::from_text(run[0].text())
                    .expect("the run head is prosodic punctuation");

                parsed
                    .templates
                    .push(EventTemplate::Punctuation(parsed.punctuation_tokens.len()));
                parsed.punctuation_tokens.push(PunctuationToken {
                    text: run[0].text().to_owned(),
                    punctuation,
                });
            }
        } else if HesitationMarker::from_text(text).is_some() {
            // Collapse a run of consecutive hesitation markers (`--`, `---`) to
            // the longest-pause marker so a dash run is one rest, not several.
            let start = index;
            while index < tokens.len()
                && HesitationMarker::from_text(tokens[index].text()).is_some()
            {
                index += 1;
            }

            let run = &tokens[start..index];
            let marker = run
                .iter()
                .filter_map(|token| HesitationMarker::from_text(token.text()))
                .max_by_key(|marker| marker.pause_samples())
                .expect("the run head is a hesitation marker");

            add_hesitation(&mut parsed, run_text(run), marker, last_voiced_index);
        } else {
            let token = &tokens[index];
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
                pos_class: token.pos_class(),
            });
            index += 1;
        }
    }

    Ok(parsed)
}

/// Joins a run of control tokens into one display string (e.g. `...`, `--`).
fn run_text(run: &[crate::TokenizedToken]) -> String {
    run.iter().map(crate::TokenizedToken::text).collect()
}

/// Records one hesitation marker and attaches its quiet rest to the preceding
/// voiced syllable, keeping the longer rest when one is already present.
fn add_hesitation(
    parsed: &mut ParsedTokens,
    text: String,
    marker: HesitationMarker,
    last_voiced_index: Option<usize>,
) {
    parsed
        .templates
        .push(EventTemplate::Hesitation(parsed.hesitation_tokens.len()));
    parsed
        .hesitation_tokens
        .push(HesitationToken { text, marker });

    if let Some(index) = last_voiced_index {
        parsed.voiced_tokens[index].timing =
            longer_hesitation(parsed.voiced_tokens[index].timing, marker.timing());
    }
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
    let mut explain_rows = vec![ExplainRow::mood(mood), ExplainRow::complexity(complexity)];

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
    // The semantic baseline pools over the ORIGINAL tokenizer tokens, before
    // any VOICE_V12 compound expansion, so derived resolution syllables never
    // distort the sequence mood.
    let baseline = mapping.squash_pooled(pool_sequence(&token_vectors)?);
    let knobs_per_voiced = token_vectors
        .iter()
        .copied()
        .map(|token_vector| assemble_knobs(baseline, mapping.squash_token(token_vector)))
        .collect::<Vec<_>>();
    let (syllables, token_ranges) = expand_engine_syllables(parsed, &knobs_per_voiced);
    let base_events = base_events_for_plan(mood, complexity, parsed, &syllables, &token_ranges);
    let plan = plan_discourse_performance(&base_events);
    let plan_rows = plan.syllables();
    let deployed = deploy_performance_timing(&syllables, plan_rows);

    let mut events = vec![
        SequenceEvent::mood(mood),
        SequenceEvent::complexity(complexity),
    ];
    let mut explain_rows = vec![ExplainRow::mood(mood), ExplainRow::complexity(complexity)];

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
                push_voiced_token(
                    &mut events,
                    &mut explain_rows,
                    &parsed.voiced_tokens[*index],
                    &syllables,
                    token_ranges[*index],
                    plan_rows,
                    &deployed,
                );
            }
        }
    }

    Ok(TextAnalysis {
        events,
        explain_rows,
    })
}

/// Emits the sequencer events and the `--explain` row for one voiced token's
/// expanded syllable range.
///
/// The explain row reports the stem; resolution syllables are rendered but not
/// separately rowed until T-126 extends `--explain` with the silhouette.
fn push_voiced_token(
    events: &mut Vec<SequenceEvent>,
    explain_rows: &mut Vec<ExplainRow>,
    token: &VoicedToken,
    syllables: &[EngineSyllable],
    token_range: (usize, usize),
    plan_rows: &[PerformanceSyllable],
    deployed: &[SyllableTiming],
) {
    let (start, end) = token_range;

    for syllable_index in start..end {
        let syllable = &syllables[syllable_index];
        let role = plan_rows
            .get(syllable_index)
            .map_or(PhraseRole::ChattyReply, PerformanceSyllable::role);
        let curves = plan_rows
            .get(syllable_index)
            .map_or_else(PerformanceCurves::neutral, PerformanceSyllable::curves);
        let archetype = archetype_for_role(role, syllable_index);

        events.push(SequenceEvent::archetype(archetype));
        events.push(SequenceEvent::Syllable(
            SyllableEvent::new(syllable.knobs, syllable.continuation)
                .with_timing(deployed[syllable_index])
                .with_performance(role, curves)
                .with_pos_class(syllable.pos_class)
                .with_duration_scale(syllable.duration_scale),
        ));

        if syllable_index == start {
            explain_rows.push(ExplainRow::token(
                token.text.clone(),
                syllable.knobs,
                syllable.continuation,
                role,
                archetype.archetype(),
                curves,
                deployed[syllable_index],
            ));
        }
    }
}

/// Expands voiced tokens into render syllables (`VOICE_V12`, T-117/T-125).
///
/// A content word's syllable target is `max(subword_count, 2)`, capped at 3 —
/// derived syllables are only ever added, never removed, so at two or more
/// natural subwords nothing is inserted. Concretely:
///
/// - A **single-token** content word becomes a `stem → class-resolution` pair:
///   the resolution derives from the stem's own knobs via the frozen per-class
///   transform and joins the stem as a subword continuation.
/// - A **multi-token** content word keeps its natural subword syllables and
///   shapes its **last** subword as the class resolution, so the silhouette is
///   consistent at any length.
/// - Every syllable of a content word takes the shortened compound duration;
///   unclassified words map through one-to-one at scale `1.0`.
///
/// The word-final timing (hesitation rests, tail shapes) stays on the word's
/// last syllable so the rest still follows the whole word. Returns the
/// syllables plus each token's `(start, end)` range into them.
fn expand_engine_syllables(
    parsed: &ParsedTokens,
    knobs_per_voiced: &[KnobSet],
) -> (Vec<EngineSyllable>, Vec<(usize, usize)>) {
    let mut syllables = Vec::with_capacity(parsed.voiced_tokens.len());
    let mut token_ranges = Vec::with_capacity(parsed.voiced_tokens.len());

    for (index, token) in parsed.voiced_tokens.iter().enumerate() {
        let start = syllables.len();
        let knobs = knobs_per_voiced[index];
        let next_is_continuation = parsed
            .voiced_tokens
            .get(index + 1)
            .is_some_and(|next| next.continuation);
        let word_final = !next_is_continuation;

        if token.pos_class.is_content() {
            if !token.continuation && word_final {
                // Single-token content word: stem plus derived resolution.
                syllables.push(EngineSyllable {
                    knobs,
                    continuation: false,
                    timing: SyllableTiming::default(),
                    pos_class: token.pos_class,
                    duration_scale: COMPOUND_SYLLABLE_DURATION_SCALE,
                });
                syllables.push(EngineSyllable {
                    knobs: class_resolution_knobs(knobs, token.pos_class),
                    continuation: true,
                    timing: token.timing,
                    pos_class: token.pos_class,
                    duration_scale: COMPOUND_SYLLABLE_DURATION_SCALE,
                });
            } else {
                // Member of a multi-token content word: one-to-one, with the
                // final subword shaped as the class resolution.
                let shaped_knobs = if word_final {
                    class_resolution_knobs(knobs, token.pos_class)
                } else {
                    knobs
                };

                syllables.push(EngineSyllable {
                    knobs: shaped_knobs,
                    continuation: token.continuation,
                    timing: token.timing,
                    pos_class: token.pos_class,
                    duration_scale: COMPOUND_SYLLABLE_DURATION_SCALE,
                });
            }
        } else {
            syllables.push(EngineSyllable {
                knobs,
                continuation: token.continuation,
                timing: token.timing,
                pos_class: token.pos_class,
                duration_scale: 1.0,
            });
        }

        token_ranges.push((start, syllables.len()));
    }

    (syllables, token_ranges)
}

fn base_events_for_plan(
    mood: UtteranceMood,
    complexity: ComplexityAnalysis,
    parsed: &ParsedTokens,
    syllables: &[EngineSyllable],
    token_ranges: &[(usize, usize)],
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
                let (start, end) = token_ranges[*index];

                for syllable in &syllables[start..end] {
                    base_events.push(SequenceEvent::Syllable(
                        SyllableEvent::new(syllable.knobs, syllable.continuation)
                            .with_timing(syllable.timing)
                            .with_pos_class(syllable.pos_class),
                    ));
                }
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

    fn complexity(complexity: ComplexityAnalysis) -> Self {
        Self::Complexity(ExplainComplexityRow { complexity })
    }

    fn token(
        token: String,
        knobs: KnobSet,
        continuation: bool,
        role: PhraseRole,
        archetype: GestureArchetype,
        curves: PerformanceCurves,
        timing: SyllableTiming,
    ) -> Self {
        Self::Token(ExplainTokenRow {
            token,
            knobs,
            continuation,
            role,
            archetype,
            curves,
            timing,
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
    syllables: &[EngineSyllable],
    plan: &[PerformanceSyllable],
) -> Vec<SyllableTiming> {
    // Only stage long turn gaps and reply rests when the utterance has real
    // discourse structure (some non-`ChattyReply` role). A plain statement stays
    // on the smooth `VOICE_V3`/`VOICE_V6` connected path.
    let has_structure = plan
        .iter()
        .any(|syllable| syllable.role() != PhraseRole::ChattyReply);
    let mut timings = Vec::with_capacity(syllables.len());

    for index in 0..syllables.len() {
        let base = syllables[index].timing;
        let role = plan
            .get(index)
            .map_or(PhraseRole::ChattyReply, PerformanceSyllable::role);
        let next_role = plan.get(index + 1).map(PerformanceSyllable::role);
        let next_continuation = syllables.get(index + 1).map(|next| next.continuation);

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
            if next_role == Some(PhraseRole::ChattyReply)
                && next_continuation == Some(false)
                && base.pause_override().is_none()
            {
                // VOICE_V8: insert a short word-boundary rest even on a plain,
                // structure-less statement so neutral multi-word input stops
                // fully bridging into one long island. Staged replies (utterances
                // that already have discourse structure) keep their longer rest.
                let amount = if has_structure {
                    STAGED_REPLY_REST_AMOUNT
                } else {
                    NEUTRAL_WORD_REST_AMOUNT
                };

                base.with_pause_override(staged_reply_rest_samples(amount))
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

impl ExplainComplexityRow {
    /// Returns the utterance complexity analysis for this control row.
    pub fn complexity(&self) -> ComplexityAnalysis {
        self.complexity
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

    /// Returns the gesture archetype selected for this voiced row.
    pub fn archetype(&self) -> GestureArchetype {
        self.archetype
    }

    /// Returns the planner performance curves for this voiced row.
    pub fn curves(&self) -> PerformanceCurves {
        self.curves
    }

    /// Returns the deployed timing directive for this voiced row.
    pub fn timing(&self) -> SyllableTiming {
        self.timing
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
