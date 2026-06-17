//! Utterance sequencing for rendered droid syllables.

use core::f64::consts::LN_2;

use crate::{
    ArchetypeSelection, CLAUSE_SYLLABLE_SAMPLES, ComplexityAnalysis, KnobSet,
    LEADING_SILENCE_SAMPLES, LONG_PUNCTUATION_PAUSE_SAMPLES, MEDIUM_PUNCTUATION_PAUSE_SAMPLES,
    PerformanceCurves, PhraseBoundaryStrength, PhraseRole, PhraseSyllablePlan,
    SENTENCE_SYLLABLE_SAMPLES, TRAILING_SILENCE_SAMPLES, UtteranceMood, exp, pitch_center_hz,
    plan_phrase_prosody,
    synth::{
        BASE_SYLLABLE_SAMPLES, SyllableConnection, SyllableFinalGlide, SyllablePerformance,
        SyllableRenderState, render_syllable_with_final_glide,
        render_syllable_with_performance_into, render_transition_bridge,
        warble_phase_offset_for_syllable,
    },
};

/// Gives the fixed empty-chirp start pitch-center knob.
pub const EMPTY_CHIRP_START_PITCH_CENTER: f64 = -0.35;

/// Gives the fixed empty-chirp target pitch-center knob.
pub const EMPTY_CHIRP_PITCH_CENTER: f64 = 0.45;

/// Gives the fixed empty-chirp vowel-position knob.
pub const EMPTY_CHIRP_VOWEL_POSITION: f64 = 0.15;

/// Gives the fixed empty-chirp contour knob.
pub const EMPTY_CHIRP_CONTOUR: f64 = 1.0;

/// Gives the fixed empty-chirp warble-depth knob.
pub const EMPTY_CHIRP_WARBLE_DEPTH: f64 = 0.85;

/// Gives the minimum `VOICE_V7` role-gated turn pause in samples (~600 ms).
pub const ROLE_LONG_PAUSE_MIN_SAMPLES: u32 = 26_460;

/// Gives the maximum `VOICE_V7` role-gated turn pause in samples (~1200 ms).
pub const ROLE_LONG_PAUSE_MAX_SAMPLES: u32 = 52_920;

/// Gives the minimum `VOICE_V7` staged-reply internal rest in samples (~30 ms).
pub const STAGED_REPLY_REST_MIN_SAMPLES: u32 = 1_323;

/// Gives the maximum `VOICE_V7` staged-reply internal rest in samples (~80 ms).
pub const STAGED_REPLY_REST_MAX_SAMPLES: u32 = 3_528;

/// Gives a per-syllable `VOICE_V7` timing directive.
///
/// The default reproduces `VOICE_V6` exactly: no pause override and a tonal
/// word-boundary bridge. The discourse planner sets a `pause_override` and/or
/// `bridge_suppressed` to stage long turn gaps and short reply rests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SyllableTiming {
    pause_override: Option<u32>,
    bridge_suppressed: bool,
    hesitation: bool,
    tail_shape: TailShape,
}

/// Gives the amplitude shape of a syllable's trailing edge (`VOICE_V9`).
///
/// Hesitation markers shape the preceding syllable's tail so the *kind* of
/// break is audible before the silent rest that follows: a dash cuts off
/// abruptly, an ellipsis trails off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TailShape {
    /// The standard release; the syllable sustains into its normal envelope.
    #[default]
    Sustained,
    /// An abrupt cutoff: the tail is clipped to silence quickly (dash).
    Clipped,
    /// A gradual trailing-off: the tail decays gently to near silence
    /// (ellipsis).
    Decayed,
}

impl SyllableTiming {
    /// Overrides the post-syllable gap length, clamped to the long-pause
    /// ceiling.
    #[must_use]
    pub fn with_pause_override(mut self, samples: u32) -> Self {
        self.pause_override = Some(samples.min(ROLE_LONG_PAUSE_MAX_SAMPLES));

        self
    }

    /// Renders the post-syllable word gap as a silent rest instead of a bridge.
    #[must_use]
    pub fn suppress_bridge(mut self) -> Self {
        self.bridge_suppressed = true;

        self
    }

    /// Marks this syllable as carrying a dash/ellipsis hesitation rest.
    ///
    /// This is distinct from generic `suppress_bridge`: it lets the discourse
    /// planner tell a hesitation marker apart from a staged reply rest even
    /// after deployment has set other suppressed bridges.
    #[must_use]
    pub fn mark_hesitation(mut self) -> Self {
        self.hesitation = true;

        self
    }

    /// Returns the explicit post-syllable gap override, if any.
    pub fn pause_override(self) -> Option<u32> {
        self.pause_override
    }

    /// Returns true when the word-boundary bridge is suppressed.
    pub fn bridge_suppressed(self) -> bool {
        self.bridge_suppressed
    }

    /// Returns true when this syllable carries a hesitation rest.
    pub fn is_hesitation(self) -> bool {
        self.hesitation
    }

    /// Returns a copy carrying a trailing-edge amplitude shape (`VOICE_V9`).
    #[must_use]
    pub(crate) fn with_tail_shape(mut self, tail_shape: TailShape) -> Self {
        self.tail_shape = tail_shape;

        self
    }

    /// Returns the trailing-edge amplitude shape for this syllable.
    pub(crate) fn tail_shape(self) -> TailShape {
        self.tail_shape
    }
}

/// Gives the deterministic dash hesitation pause in samples (~340 ms).
pub const DASH_HESITATION_PAUSE_SAMPLES: u32 = 14_994;

/// Gives the deterministic ellipsis hesitation pause in samples (~500 ms).
pub const ELLIPSIS_HESITATION_PAUSE_SAMPLES: u32 = 22_050;

/// Gives a control-only `VOICE_V7` hesitation marker.
///
/// Standalone dashes and ellipses are not voiced semantic tokens; they shape
/// the preceding syllable's timing with a quiet, deterministic hesitation rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HesitationMarker {
    /// `-`, `--`, en dash, or em dash.
    Dash,
    /// A single-character ellipsis.
    Ellipsis,
}

impl HesitationMarker {
    /// Parses a tokenizer text cell as a frozen hesitation marker.
    pub fn from_text(text: &str) -> Option<Self> {
        match text {
            "-" | "--" | "—" | "–" => Some(Self::Dash),
            "…" => Some(Self::Ellipsis),
            _ => None,
        }
    }

    /// Returns the deterministic hesitation pause this marker contributes.
    pub fn pause_samples(self) -> u32 {
        match self {
            Self::Dash => DASH_HESITATION_PAUSE_SAMPLES,
            Self::Ellipsis => ELLIPSIS_HESITATION_PAUSE_SAMPLES,
        }
    }

    /// Returns the trailing-edge shape this marker imposes on the prior
    /// syllable (`VOICE_V9`): a dash clips abruptly, an ellipsis trails off.
    pub fn tail_shape(self) -> TailShape {
        match self {
            Self::Dash => TailShape::Clipped,
            Self::Ellipsis => TailShape::Decayed,
        }
    }

    /// Returns the timing directive a hesitation marker imposes on the prior
    /// syllable: a quiet, bridge-suppressed rest of the marker's pause length.
    pub fn timing(self) -> SyllableTiming {
        SyllableTiming::default()
            .with_pause_override(self.pause_samples())
            .suppress_bridge()
            .mark_hesitation()
            .with_tail_shape(self.tail_shape())
    }
}

/// Maps a role amount to a bounded long turn pause in samples.
pub fn role_long_pause_samples(amount: f64) -> u32 {
    let amount = if amount.is_finite() {
        amount.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let span = f64::from(ROLE_LONG_PAUSE_MAX_SAMPLES - ROLE_LONG_PAUSE_MIN_SAMPLES);
    let target = f64::from(ROLE_LONG_PAUSE_MIN_SAMPLES) + (span * amount);

    round_f64_to_u32(target).clamp(ROLE_LONG_PAUSE_MIN_SAMPLES, ROLE_LONG_PAUSE_MAX_SAMPLES)
}

/// Maps a role amount to a bounded staged-reply internal rest in samples.
pub fn staged_reply_rest_samples(amount: f64) -> u32 {
    let amount = if amount.is_finite() {
        amount.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let span = f64::from(STAGED_REPLY_REST_MAX_SAMPLES - STAGED_REPLY_REST_MIN_SAMPLES);
    let target = f64::from(STAGED_REPLY_REST_MIN_SAMPLES) + (span * amount);

    round_f64_to_u32(target).clamp(STAGED_REPLY_REST_MIN_SAMPLES, STAGED_REPLY_REST_MAX_SAMPLES)
}

/// Gives one input event consumed by the utterance sequencer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SequenceEvent {
    /// A whole-utterance mood control event.
    Mood(UtteranceMood),
    /// A whole-utterance complexity control event.
    Complexity(ComplexityAnalysis),
    /// A selected gesture archetype for the following voiced syllable.
    Archetype(ArchetypeSelection),
    /// A voiced syllable with semantic knobs.
    Syllable(SyllableEvent),
    /// A control-only prosodic punctuation marker.
    Punctuation(ProsodicPunctuation),
}

/// Gives one voiced syllable to lay out in time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyllableEvent {
    knobs: KnobSet,
    continuation: bool,
    timing: SyllableTiming,
    role: PhraseRole,
    curves: PerformanceCurves,
}

/// Gives one prosodic punctuation marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProsodicPunctuation {
    /// `?` rising final glide with a long pause.
    Question,
    /// `.` falling final glide with a long pause.
    Period,
    /// `!` falling final glide with a long pause.
    Exclamation,
    /// `,` medium pause without a contour change.
    Comma,
    /// `;` medium pause without a contour change.
    Semicolon,
    /// `:` medium pause without a contour change.
    Colon,
}

/// Gives the sequenced utterance route.
#[derive(Debug, Clone, PartialEq)]
pub enum SequencedUtterance {
    /// Rendered audio samples.
    Samples(Vec<f64>),
    /// Rendered fixed chirp gesture for empty input.
    EmptyChirp(Vec<f64>),
}

#[derive(Debug, Clone, Copy)]
struct SyllablePlan {
    event: SyllableEvent,
    archetype: ArchetypeSelection,
    final_glide: SyllableFinalGlide,
    punctuation_seen: bool,
}

#[derive(Debug, Clone, Copy)]
struct SequencerMood {
    mood: UtteranceMood,
    explicit: bool,
}

#[derive(Debug, Clone, Copy)]
struct SequencerComplexity {
    analysis: ComplexityAnalysis,
    explicit: bool,
}

impl SequenceEvent {
    /// Builds a whole-utterance mood event.
    pub fn mood(mood: UtteranceMood) -> Self {
        Self::Mood(mood)
    }

    /// Builds a whole-utterance complexity event.
    pub fn complexity(complexity: ComplexityAnalysis) -> Self {
        Self::Complexity(complexity)
    }

    /// Builds a selected gesture archetype event.
    pub fn archetype(archetype: ArchetypeSelection) -> Self {
        Self::Archetype(archetype)
    }

    /// Builds a voiced syllable event.
    pub fn syllable(knobs: KnobSet, continuation: bool) -> Self {
        Self::Syllable(SyllableEvent::new(knobs, continuation))
    }

    /// Builds a voiced syllable event with an explicit timing directive.
    pub fn syllable_with_timing(
        knobs: KnobSet,
        continuation: bool,
        timing: SyllableTiming,
    ) -> Self {
        Self::Syllable(SyllableEvent::new(knobs, continuation).with_timing(timing))
    }

    /// Builds a prosodic punctuation event.
    pub fn punctuation(punctuation: ProsodicPunctuation) -> Self {
        Self::Punctuation(punctuation)
    }
}

impl SyllableEvent {
    /// Builds one voiced syllable event.
    pub fn new(knobs: KnobSet, continuation: bool) -> Self {
        Self {
            knobs,
            continuation,
            timing: SyllableTiming::default(),
            role: PhraseRole::default(),
            curves: PerformanceCurves::neutral(),
        }
    }

    /// Returns a copy of this syllable carrying a timing directive.
    #[must_use]
    pub fn with_timing(mut self, timing: SyllableTiming) -> Self {
        self.timing = timing;

        self
    }

    /// Returns a copy of this syllable carrying a discourse role and curves.
    #[must_use]
    pub fn with_performance(mut self, role: PhraseRole, curves: PerformanceCurves) -> Self {
        self.role = role;
        self.curves = curves;

        self
    }

    /// Returns the discourse role assigned to this syllable.
    pub fn role(&self) -> PhraseRole {
        self.role
    }

    /// Returns the performance curves assigned to this syllable.
    pub fn curves(&self) -> PerformanceCurves {
        self.curves
    }

    /// Returns the semantic knobs for this syllable.
    pub fn knobs(&self) -> KnobSet {
        self.knobs
    }

    /// Returns true when this syllable continues the previous wordpiece.
    pub fn is_continuation(&self) -> bool {
        self.continuation
    }

    /// Returns the timing directive for this syllable.
    pub fn timing(&self) -> SyllableTiming {
        self.timing
    }
}

impl ProsodicPunctuation {
    /// Parses a tokenizer text cell as a frozen prosodic punctuation marker.
    pub fn from_text(text: &str) -> Option<Self> {
        match text {
            "?" => Some(Self::Question),
            "." => Some(Self::Period),
            "!" => Some(Self::Exclamation),
            "," => Some(Self::Comma),
            ";" => Some(Self::Semicolon),
            ":" => Some(Self::Colon),
            _ => None,
        }
    }

    /// Returns the fixed pause length contributed by this control marker.
    pub fn pause_samples(self) -> u32 {
        match self {
            Self::Question | Self::Period | Self::Exclamation => LONG_PUNCTUATION_PAUSE_SAMPLES,
            Self::Comma | Self::Semicolon | Self::Colon => MEDIUM_PUNCTUATION_PAUSE_SAMPLES,
        }
    }

    pub(crate) fn final_glide(self) -> SyllableFinalGlide {
        match self {
            Self::Question => SyllableFinalGlide::Rising,
            Self::Period | Self::Exclamation => SyllableFinalGlide::Falling,
            Self::Comma | Self::Semicolon | Self::Colon => SyllableFinalGlide::Continuation,
        }
    }
}

impl SequencedUtterance {
    /// Returns true when this utterance should route to the empty chirp.
    pub fn is_empty_chirp(&self) -> bool {
        matches!(self, Self::EmptyChirp(_))
    }

    /// Returns rendered samples for this utterance.
    pub fn samples(&self) -> &[f64] {
        match self {
            Self::Samples(samples) | Self::EmptyChirp(samples) => samples,
        }
    }
}

/// Renders the fixed empty-input inquisitive chirp gesture.
pub fn render_empty_chirp() -> Vec<f64> {
    let knobs = KnobSet::from_axes([
        EMPTY_CHIRP_PITCH_CENTER,
        EMPTY_CHIRP_VOWEL_POSITION,
        EMPTY_CHIRP_CONTOUR,
        EMPTY_CHIRP_WARBLE_DEPTH,
    ]);
    let mut samples = Vec::new();

    append_silence(&mut samples, LEADING_SILENCE_SAMPLES);
    samples.extend(render_syllable_with_final_glide(
        knobs,
        pitch_center_hz(EMPTY_CHIRP_START_PITCH_CENTER),
        SyllableFinalGlide::Rising,
        0.0,
    ));
    append_silence(&mut samples, TRAILING_SILENCE_SAMPLES);

    samples
}

/// Estimates the exact number of samples an utterance will render.
pub fn estimate_utterance_sample_count(events: &[SequenceEvent]) -> u64 {
    let plans = syllable_plans(events);

    if plans.is_empty() {
        return empty_chirp_sample_count();
    }

    let mut sample_count = u64::from(LEADING_SILENCE_SAMPLES) + u64::from(TRAILING_SILENCE_SAMPLES);
    let mood = mood_from_events(events);
    let complexity = complexity_from_events(events);
    let phrase_plan = plan_phrase_prosody(events);

    for (plan, phrase_syllable) in plans.iter().zip(phrase_plan.syllables()) {
        sample_count += u64::from(phrase_syllable_samples(*phrase_syllable, mood, complexity));
        sample_count += u64::from(effective_pause_samples(
            plan.event.timing(),
            *phrase_syllable,
        ));
    }

    sample_count
}

fn effective_pause_samples(timing: SyllableTiming, phrase_syllable: PhraseSyllablePlan) -> u32 {
    timing
        .pause_override()
        .unwrap_or_else(|| phrase_syllable.pause_samples())
}

/// Lays out voiced syllables and control punctuation into an utterance.
pub fn sequence_utterance(events: &[SequenceEvent]) -> SequencedUtterance {
    let plans = syllable_plans(events);
    let phrase_plan = plan_phrase_prosody(events);
    let mood = mood_from_events(events);
    let complexity = complexity_from_events(events);

    if plans.is_empty() {
        return SequencedUtterance::EmptyChirp(render_empty_chirp());
    }

    let mut samples = Vec::new();
    let mut synth_state = SyllableRenderState::new();
    let mut previous_pitch_hz = None;
    let mut pending_reset_semitones = 0.0;

    append_silence(&mut samples, LEADING_SILENCE_SAMPLES);

    for (index, (plan, phrase_syllable)) in plans
        .iter()
        .copied()
        .zip(phrase_plan.syllables().iter().copied())
        .enumerate()
    {
        let syllable = plan.event;
        let pitch_offset_semitones = phrase_syllable.declination_offset_semitones()
            + pending_reset_semitones
            + mood_pitch_offset_semitones(mood);
        let target_pitch_hz = pitch_with_offset(syllable.knobs(), pitch_offset_semitones);
        let start_pitch_hz = match previous_pitch_hz {
            Some(previous_pitch_hz) => previous_pitch_hz,
            None => target_pitch_hz,
        };
        let start_connection = phrase_plan
            .syllables()
            .get(index.saturating_sub(1))
            .copied()
            .filter(|_| index > 0)
            .map_or(SyllableConnection::Detached, start_connection_from_previous);
        let end_connection = if index + 1 < plans.len() {
            end_connection_from_boundary(phrase_syllable)
        } else {
            SyllableConnection::Detached
        };

        render_syllable_with_performance_into(
            syllable.knobs(),
            start_pitch_hz,
            plan.final_glide,
            warble_phase_offset_for_syllable(index),
            SyllablePerformance::new(
                phrase_syllable_samples(phrase_syllable, mood, complexity),
                pitch_offset_semitones,
                phrase_syllable.final_lowering_semitones(),
                phrase_syllable.is_emphasized(),
            )
            .with_connections(start_connection, end_connection)
            .with_expression(
                mood.valence(),
                mood.arousal(),
                complexity.scalar(),
                plan.archetype,
            )
            .with_curves(syllable.role(), syllable.curves())
            .with_tail_shape(syllable.timing().tail_shape()),
            &mut synth_state,
            &mut samples,
        );
        pending_reset_semitones = phrase_syllable.pitch_reset_semitones();
        previous_pitch_hz = if pending_reset_semitones > 0.0 {
            None
        } else {
            Some(target_pitch_hz)
        };

        let timing = plan.event.timing();
        let pause_samples = effective_pause_samples(timing, phrase_syllable);

        if pause_samples > 0 {
            if phrase_syllable.boundary_strength() == PhraseBoundaryStrength::Word
                && !timing.bridge_suppressed()
            {
                let bridge_target_pitch_hz = next_target_pitch_hz(
                    index,
                    &plans,
                    phrase_plan.syllables(),
                    mood,
                    pending_reset_semitones,
                )
                .unwrap_or(target_pitch_hz);

                render_transition_bridge(
                    syllable.knobs(),
                    previous_pitch_hz.unwrap_or(target_pitch_hz),
                    bridge_target_pitch_hz,
                    pause_samples,
                    &mut synth_state,
                    &mut samples,
                );
                previous_pitch_hz = Some(bridge_target_pitch_hz);
            } else {
                append_silence(&mut samples, pause_samples);
                synth_state = SyllableRenderState::new();
            }
        }
    }

    append_silence(&mut samples, TRAILING_SILENCE_SAMPLES);

    SequencedUtterance::Samples(samples)
}

fn phrase_syllable_samples(
    phrase_syllable: PhraseSyllablePlan,
    mood: SequencerMood,
    complexity: SequencerComplexity,
) -> u32 {
    let base_samples = match phrase_syllable.boundary_strength() {
        PhraseBoundaryStrength::None | PhraseBoundaryStrength::Word => BASE_SYLLABLE_SAMPLES,
        PhraseBoundaryStrength::Clause => CLAUSE_SYLLABLE_SAMPLES,
        PhraseBoundaryStrength::Sentence => SENTENCE_SYLLABLE_SAMPLES,
    };
    let duration_scale = if mood.explicit {
        text_syllable_duration_scale(mood.mood.arousal())
    } else {
        1.0
    };

    let complexity_scale = if complexity.explicit {
        1.0 + (0.12 * complexity.scalar())
    } else {
        1.0
    };

    round_f64_to_u32(f64::from(base_samples) * duration_scale * complexity_scale)
}

/// Gives the text-path per-syllable duration scale from utterance arousal.
///
/// Applies only to the explicit (text-derived) mood path; hand-built events
/// keep a scale of exactly `1.0` (`BASE_SYLLABLE_SAMPLES`), so they stay
/// byte-identical. Higher arousal paces faster (shorter syllables).
///
/// `VOICE_V10` lowers the whole curve below `1.0`: the taxonomy found neutral
/// dootdoot gestures run far longer than BB-8's short blips (calm text used to
/// pace *longer* than the base). Calm text now paces a touch shorter than the
/// base and arousal shortens it further, so neutral gestures read as quick
/// pips.
pub fn text_syllable_duration_scale(arousal: f64) -> f64 {
    let arousal = if arousal.is_finite() {
        arousal.clamp(0.0, 1.0)
    } else {
        0.0
    };

    (0.90 - (0.10 * arousal)).clamp(0.80, 0.90)
}

fn pitch_with_offset(knobs: KnobSet, offset_semitones: f64) -> f64 {
    pitch_center_hz(knobs.pitch_center()) * exp((LN_2 * offset_semitones) / 12.0)
}

fn start_connection_from_previous(phrase_syllable: PhraseSyllablePlan) -> SyllableConnection {
    end_connection_from_boundary(phrase_syllable)
}

fn end_connection_from_boundary(phrase_syllable: PhraseSyllablePlan) -> SyllableConnection {
    match phrase_syllable.boundary_strength() {
        PhraseBoundaryStrength::None => SyllableConnection::Subword,
        PhraseBoundaryStrength::Word => SyllableConnection::Word,
        PhraseBoundaryStrength::Clause | PhraseBoundaryStrength::Sentence => {
            SyllableConnection::Detached
        }
    }
}

fn next_target_pitch_hz(
    index: usize,
    plans: &[SyllablePlan],
    phrase_syllables: &[PhraseSyllablePlan],
    mood: SequencerMood,
    pending_reset_semitones: f64,
) -> Option<f64> {
    let next_index = index.checked_add(1)?;
    let next_plan = plans.get(next_index)?;
    let next_phrase_syllable = phrase_syllables.get(next_index)?;
    let pitch_offset_semitones = next_phrase_syllable.declination_offset_semitones()
        + pending_reset_semitones
        + mood_pitch_offset_semitones(mood);

    Some(pitch_with_offset(
        next_plan.event.knobs(),
        pitch_offset_semitones,
    ))
}

fn mood_from_events(events: &[SequenceEvent]) -> SequencerMood {
    events
        .iter()
        .find_map(|event| match event {
            SequenceEvent::Mood(mood) => Some(*mood),
            SequenceEvent::Complexity(_)
            | SequenceEvent::Archetype(_)
            | SequenceEvent::Syllable(_)
            | SequenceEvent::Punctuation(_) => None,
        })
        .map_or_else(SequencerMood::absent, SequencerMood::explicit)
}

fn complexity_from_events(events: &[SequenceEvent]) -> SequencerComplexity {
    events
        .iter()
        .find_map(|event| match event {
            SequenceEvent::Complexity(complexity) => Some(*complexity),
            SequenceEvent::Mood(_)
            | SequenceEvent::Archetype(_)
            | SequenceEvent::Syllable(_)
            | SequenceEvent::Punctuation(_) => None,
        })
        .map_or_else(SequencerComplexity::absent, SequencerComplexity::explicit)
}

fn mood_pitch_offset_semitones(mood: SequencerMood) -> f64 {
    if mood.explicit {
        (1.20 * mood.mood.arousal()) + (0.40 * mood.mood.valence())
    } else {
        0.0
    }
}

fn round_f64_to_u32(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 1;
    }

    let rounded = value.round();
    let mut low = 1_u32;
    let mut high = u32::MAX;

    while low <= high {
        let midpoint = low + ((high - low) / 2);
        let midpoint_value = f64::from(midpoint);

        if midpoint_value < rounded {
            low = midpoint.saturating_add(1);
        } else if midpoint_value > rounded {
            high = midpoint.saturating_sub(1);
        } else {
            return midpoint;
        }
    }

    u32::MAX
}

fn syllable_plans(events: &[SequenceEvent]) -> Vec<SyllablePlan> {
    let mut plans = Vec::new();
    let mut pending_archetype = None;

    for event in events {
        match event {
            SequenceEvent::Mood(_) | SequenceEvent::Complexity(_) => {}
            SequenceEvent::Archetype(archetype) => pending_archetype = Some(*archetype),
            SequenceEvent::Syllable(syllable) => {
                let archetype = pending_archetype
                    .take()
                    .unwrap_or_else(|| ArchetypeSelection::chatter(plans.len()));

                plans.push(SyllablePlan::new(*syllable, archetype));
            }
            SequenceEvent::Punctuation(punctuation) => {
                if let Some(plan) = plans.last_mut() {
                    plan.attach_punctuation(*punctuation);
                }
            }
        }
    }

    plans
}

fn empty_chirp_sample_count() -> u64 {
    u64::from(LEADING_SILENCE_SAMPLES)
        + u64::from(BASE_SYLLABLE_SAMPLES)
        + u64::from(TRAILING_SILENCE_SAMPLES)
}

fn append_silence(samples: &mut Vec<f64>, count: u32) {
    for _ in 0..count {
        samples.push(0.0);
    }
}

impl SyllablePlan {
    fn new(event: SyllableEvent, archetype: ArchetypeSelection) -> Self {
        Self {
            event,
            archetype,
            final_glide: SyllableFinalGlide::Neutral,
            punctuation_seen: false,
        }
    }

    fn attach_punctuation(&mut self, punctuation: ProsodicPunctuation) {
        if !self.punctuation_seen {
            self.final_glide = punctuation.final_glide();
            self.punctuation_seen = true;
        }
    }
}

impl SequencerMood {
    fn explicit(mood: UtteranceMood) -> Self {
        Self {
            mood,
            explicit: true,
        }
    }

    fn absent() -> Self {
        Self {
            mood: UtteranceMood::new(0.0, 0.0),
            explicit: false,
        }
    }

    fn valence(self) -> f64 {
        self.mood.valence()
    }

    fn arousal(self) -> f64 {
        self.mood.arousal()
    }
}

impl SequencerComplexity {
    fn explicit(analysis: ComplexityAnalysis) -> Self {
        Self {
            analysis,
            explicit: true,
        }
    }

    fn absent() -> Self {
        Self {
            analysis: ComplexityAnalysis::zero(),
            explicit: false,
        }
    }

    fn scalar(self) -> f64 {
        self.analysis.scalar()
    }
}
