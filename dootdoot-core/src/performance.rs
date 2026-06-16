//! Deterministic `VOICE_V7` discourse-performance planning.
//!
//! The planner runs after tokenization and before synthesis. It is a pure
//! function of the sequencer event stream: it segments voiced syllables into
//! discourse phrases by punctuation and hesitation timing, assigns each phrase
//! a local role, and emits bounded continuous performance curves per syllable.
//! The synthesis stage (wired in a later task) reads these to deploy the
//! `VOICE_V7` primitives by role rather than applying one global affect to
//! every syllable.

use crate::{KnobSet, ProsodicPunctuation, SequenceEvent, SyllableEvent};

/// Gives the `VOICE_V8` per-syllable movement normalizer (axis-space distance).
///
/// Word-to-word distance in the four-axis knob space is divided by this to a
/// `[0, 1]` movement amount; a typical token-to-token hop saturates it.
const MOVEMENT_NORMALIZER: f64 = 2.0;

/// Gives the `VOICE_V8` semantic-engagement gains applied on top of role
/// curves.
///
/// These widen each performance channel by the syllable's salience/movement
/// "drive", so a neutral, punctuation-less phrase still moves; the additional
/// `ACCENT_*` terms promote one syllable per chatty/probe segment into a
/// bright, whistle/roughness-engaging accent. Every channel stays clamped by
/// [`PerformanceCurves::new`].
const ENGAGE_PITCH_BIAS: f64 = 0.20;
const ENGAGE_PITCH_VELOCITY: f64 = 0.30;
const ENGAGE_FORMANT_VELOCITY: f64 = 0.20;
const ENGAGE_BRIGHTNESS: f64 = 0.30;
const ENGAGE_MOUTH: f64 = 0.12;
const ENGAGE_TENSION: f64 = 0.30;
const ACCENT_PITCH_VELOCITY: f64 = 0.25;
const ACCENT_BRIGHTNESS: f64 = 0.30;
const ACCENT_TENSION: f64 = 0.40;

/// Gives how many trailing syllables of a terminal-flourish segment actually
/// flourish.
///
/// `VOICE_V8`: the whistle/yelp is a **terminal accent**, not a treatment for
/// the whole final phrase. In a long closing segment ("the weather is 78!")
/// only the last few syllables flourish; the lead-in stays a chatty body so the
/// phrase does not whistle on every syllable. Short flourishes (≤ this many
/// syllables, e.g. "hello there?" or "today?!") are unchanged.
const FLOURISH_TAIL_SYLLABLES: usize = 1;

/// Gives one local discourse role for a phrase of voiced syllables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhraseRole {
    /// A leading or question gesture: longer, rising, less dense.
    Probe,
    /// A conversational answer after a reset: shorter, denser, varied.
    ///
    /// Also the neutral baseline used by hand-built events.
    #[default]
    ChattyReply,
    /// A dash/ellipsis hesitation: quiet, rounded, held.
    Hesitation,
    /// A final accented gesture: one whistle/yelp climb, not every syllable.
    TerminalFlourish,
    /// A comma/colon aside: lower, darker, shorter pitch span.
    Aside,
}

/// Gives bounded continuous performance curves for one syllable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceCurves {
    pitch_center_bias: f64,
    pitch_velocity: f64,
    formant_target: f64,
    formant_velocity: f64,
    brightness_pressure: f64,
    mouth_openness: f64,
    archetype_tension: f64,
}

/// Gives one planned syllable: its role and continuous curves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceSyllable {
    syllable_index: usize,
    role: PhraseRole,
    curves: PerformanceCurves,
}

/// Gives the deterministic discourse-performance plan for an utterance.
#[derive(Debug, Clone, PartialEq)]
pub struct PerformancePlan {
    syllables: Vec<PerformanceSyllable>,
}

#[derive(Debug, Clone)]
struct Segment {
    syllable_indices: Vec<usize>,
    terminal: Option<ProsodicPunctuation>,
    hesitation: bool,
}

impl PerformanceCurves {
    /// Builds curves, clamping every channel into its fixed bounded range.
    pub fn new(
        pitch_center_bias: f64,
        pitch_velocity: f64,
        formant_target: f64,
        formant_velocity: f64,
        brightness_pressure: f64,
        mouth_openness: f64,
        archetype_tension: f64,
    ) -> Self {
        Self {
            pitch_center_bias: clamp_signed(pitch_center_bias),
            pitch_velocity: clamp_signed(pitch_velocity),
            formant_target: clamp_signed(formant_target),
            formant_velocity: clamp_signed(formant_velocity),
            brightness_pressure: clamp_unit(brightness_pressure),
            mouth_openness: clamp_unit(mouth_openness),
            archetype_tension: clamp_unit(archetype_tension),
        }
    }

    /// Returns the phrase-level pitch center bias in `[-1, 1]`.
    pub fn pitch_center_bias(&self) -> f64 {
        self.pitch_center_bias
    }

    /// Returns the pitch velocity (rise/fall pressure) in `[-1, 1]`.
    pub fn pitch_velocity(&self) -> f64 {
        self.pitch_velocity
    }

    /// Returns the formant target bias in `[-1, 1]`.
    pub fn formant_target(&self) -> f64 {
        self.formant_target
    }

    /// Returns the formant velocity in `[-1, 1]`.
    pub fn formant_velocity(&self) -> f64 {
        self.formant_velocity
    }

    /// Returns the brightness pressure in `[0, 1]`.
    pub fn brightness_pressure(&self) -> f64 {
        self.brightness_pressure
    }

    /// Returns the mouth openness in `[0, 1]`.
    pub fn mouth_openness(&self) -> f64 {
        self.mouth_openness
    }

    /// Returns the archetype tension/release in `[0, 1]`.
    pub fn archetype_tension(&self) -> f64 {
        self.archetype_tension
    }

    /// Builds the neutral curve set, where every performance channel is off.
    ///
    /// A syllable rendered with neutral curves is byte-identical to `VOICE_V6`,
    /// so hand-built events and the empty chirp are unaffected; only the
    /// engine's planner-driven text path attaches expressive curves.
    pub fn neutral() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }
}

impl Default for PerformanceCurves {
    fn default() -> Self {
        Self::neutral()
    }
}

impl PerformanceSyllable {
    /// Returns the voiced syllable index this plan row belongs to.
    pub fn syllable_index(&self) -> usize {
        self.syllable_index
    }

    /// Returns the local discourse role for this syllable.
    pub fn role(&self) -> PhraseRole {
        self.role
    }

    /// Returns the continuous performance curves for this syllable.
    pub fn curves(&self) -> PerformanceCurves {
        self.curves
    }
}

impl PerformancePlan {
    /// Returns the per-syllable performance plan rows.
    pub fn syllables(&self) -> &[PerformanceSyllable] {
        &self.syllables
    }

    /// Returns true when the utterance has no voiced syllables.
    pub fn is_empty(&self) -> bool {
        self.syllables.is_empty()
    }
}

/// Plans deterministic discourse performance from sequencer events.
pub fn plan_discourse_performance(events: &[SequenceEvent]) -> PerformancePlan {
    let voiced_knobs = collect_voiced_knobs(events);
    let segments = segment_events(events);
    let segment_count = segments.len();
    let mut syllables = Vec::new();

    for (segment_index, segment) in segments.iter().enumerate() {
        let role = segment_role(
            segment,
            segment_index == 0,
            segment_index + 1 == segment_count,
        );
        let length = segment.syllable_indices.len();
        let accent_position = accent_position(segment, role, &voiced_knobs);

        for (position, syllable_index) in segment.syllable_indices.iter().copied().enumerate() {
            let fraction = if length <= 1 {
                0.0
            } else {
                position_to_f64(position) / position_to_f64(length - 1)
            };
            let knobs = voiced_knobs.get(syllable_index).copied();
            let previous_knobs = position
                .checked_sub(1)
                .and_then(|previous| segment.syllable_indices.get(previous).copied())
                .and_then(|index| voiced_knobs.get(index).copied());
            let salience = knobs.map_or(0.0, syllable_salience);
            let movement = match (knobs, previous_knobs) {
                (Some(current), Some(previous)) => syllable_movement(current, previous),
                _ => 0.0,
            };
            // Reserve the flourish for the tail of a long final segment; the
            // lead-in stays a chatty body so the closing phrase does not whistle
            // on every syllable.
            let effective_role = flourish_tail_role(role, position, length);
            let is_accent = accent_position == Some(position);
            let base = role_curves(effective_role, fraction);
            // Semantic engagement targets only the body roles (chatty-reply /
            // probe). The terminal flourish, hesitation, and aside already carry
            // intentional, self-contained curve shapes; amplifying them with the
            // drive/accent stacked tension and roughness onto an already-intense
            // gesture (a trailing "!" turned the whole closing phrase erratic).
            let curves = if matches!(effective_role, PhraseRole::ChattyReply | PhraseRole::Probe) {
                engage_curves(base, salience, movement, is_accent)
            } else {
                base
            };

            syllables.push(PerformanceSyllable {
                syllable_index,
                role: effective_role,
                curves,
            });
        }
    }

    PerformancePlan { syllables }
}

fn collect_voiced_knobs(events: &[SequenceEvent]) -> Vec<KnobSet> {
    events
        .iter()
        .filter_map(|event| match event {
            SequenceEvent::Syllable(syllable) => Some(syllable.knobs()),
            SequenceEvent::Mood(_)
            | SequenceEvent::Complexity(_)
            | SequenceEvent::Archetype(_)
            | SequenceEvent::Punctuation(_) => None,
        })
        .collect()
}

/// Picks the in-segment position of the highest-salience syllable to accent.
///
/// Only the body roles (`ChattyReply`, `Probe`) earn a semantic accent; the
/// terminal flourish, hesitation, and aside already carry their own treatment.
/// Ties resolve to the earliest syllable.
fn accent_position(segment: &Segment, role: PhraseRole, voiced_knobs: &[KnobSet]) -> Option<usize> {
    if !matches!(role, PhraseRole::ChattyReply | PhraseRole::Probe) {
        return None;
    }

    let mut best: Option<(usize, f64)> = None;

    for (position, index) in segment.syllable_indices.iter().copied().enumerate() {
        let salience = voiced_knobs
            .get(index)
            .copied()
            .map_or(0.0, syllable_salience);

        if best.is_none_or(|(_, best_salience)| salience > best_salience) {
            best = Some((position, salience));
        }
    }

    best.map(|(position, _)| position)
}

/// Reserves a terminal flourish for the trailing syllables of its segment.
///
/// Earlier syllables of a long final segment fall back to `ChattyReply` so they
/// read as a chatty lead-in rather than a whole-phrase whistle; short segments
/// (length ≤ [`FLOURISH_TAIL_SYLLABLES`]) keep the flourish on every syllable.
fn flourish_tail_role(role: PhraseRole, position: usize, length: usize) -> PhraseRole {
    if role == PhraseRole::TerminalFlourish && position + FLOURISH_TAIL_SYLLABLES < length {
        PhraseRole::ChattyReply
    } else {
        role
    }
}

/// Scores a syllable's expressive salience in `[0, 1]` from its semantic knobs.
fn syllable_salience(knobs: KnobSet) -> f64 {
    let contour = knobs.contour().abs();
    let warble = (knobs.warble_depth().clamp(-1.0, 1.0) + 1.0) * 0.5;
    let pitch = knobs.pitch_center().abs();

    ((0.5 * contour) + (0.3 * warble) + (0.2 * pitch)).clamp(0.0, 1.0)
}

/// Measures word-to-word movement in `[0, 1]` as normalized axis-space
/// distance.
fn syllable_movement(current: KnobSet, previous: KnobSet) -> f64 {
    let distance = current
        .axes()
        .iter()
        .zip(previous.axes())
        .map(|(current, previous)| {
            let delta = current - previous;

            delta * delta
        })
        .sum::<f64>()
        .sqrt();

    (distance / MOVEMENT_NORMALIZER).clamp(0.0, 1.0)
}

/// Widens role curves by semantic drive and promotes the accent syllable.
fn engage_curves(
    base: PerformanceCurves,
    salience: f64,
    movement: f64,
    is_accent: bool,
) -> PerformanceCurves {
    let drive = salience.max(0.6 * movement).clamp(0.0, 1.0);
    let accent = f64::from(u8::from(is_accent));

    PerformanceCurves::new(
        base.pitch_center_bias() + (ENGAGE_PITCH_BIAS * movement),
        base.pitch_velocity() + (ENGAGE_PITCH_VELOCITY * drive) + (ACCENT_PITCH_VELOCITY * accent),
        base.formant_target(),
        base.formant_velocity() + (ENGAGE_FORMANT_VELOCITY * drive),
        base.brightness_pressure() + (ENGAGE_BRIGHTNESS * drive) + (ACCENT_BRIGHTNESS * accent),
        base.mouth_openness() + (ENGAGE_MOUTH * drive),
        base.archetype_tension() + (ENGAGE_TENSION * drive) + (ACCENT_TENSION * accent),
    )
}

fn segment_events(events: &[SequenceEvent]) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current = Segment::empty();
    let mut voiced_index = 0_usize;

    for event in events {
        match event {
            SequenceEvent::Syllable(syllable) => {
                current.syllable_indices.push(voiced_index);
                voiced_index += 1;

                if syllable_is_hesitation(syllable) {
                    current.hesitation = true;
                    segments.push(std::mem::replace(&mut current, Segment::empty()));
                }
            }
            SequenceEvent::Punctuation(punctuation) => {
                if !current.syllable_indices.is_empty() {
                    current.terminal = Some(*punctuation);
                    segments.push(std::mem::replace(&mut current, Segment::empty()));
                }
            }
            SequenceEvent::Mood(_) | SequenceEvent::Complexity(_) | SequenceEvent::Archetype(_) => {
            }
        }
    }

    if !current.syllable_indices.is_empty() {
        segments.push(current);
    }

    segments
}

fn syllable_is_hesitation(syllable: &SyllableEvent) -> bool {
    syllable.timing().is_hesitation()
}

fn segment_role(segment: &Segment, is_first: bool, is_last: bool) -> PhraseRole {
    if segment.hesitation {
        return PhraseRole::Hesitation;
    }

    match segment.terminal {
        Some(
            ProsodicPunctuation::Comma
            | ProsodicPunctuation::Semicolon
            | ProsodicPunctuation::Colon,
        ) => PhraseRole::Aside,
        Some(ProsodicPunctuation::Question | ProsodicPunctuation::Exclamation) if is_last => {
            PhraseRole::TerminalFlourish
        }
        Some(ProsodicPunctuation::Question) => PhraseRole::Probe,
        _ if is_first && !is_last => PhraseRole::Probe,
        _ => PhraseRole::ChattyReply,
    }
}

fn role_curves(role: PhraseRole, fraction: f64) -> PerformanceCurves {
    let fraction = fraction.clamp(0.0, 1.0);

    match role {
        PhraseRole::Probe => PerformanceCurves::new(
            0.15 + (0.25 * fraction),
            0.45,
            0.20,
            0.25,
            0.40 + (0.20 * fraction),
            0.65,
            0.45,
        ),
        PhraseRole::ChattyReply => PerformanceCurves::new(0.0, 0.10, 0.0, 0.20, 0.20, 0.40, 0.40),
        PhraseRole::Hesitation => {
            PerformanceCurves::new(-0.10, -0.15, -0.25, -0.05, 0.20, 0.80, 0.20)
        }
        PhraseRole::TerminalFlourish => PerformanceCurves::new(
            0.35 + (0.45 * fraction),
            0.55 + (0.40 * fraction),
            0.30,
            0.35,
            0.55 + (0.35 * fraction),
            0.55,
            0.60 + (0.35 * fraction),
        ),
        PhraseRole::Aside => PerformanceCurves::new(-0.30, -0.10, -0.20, -0.10, 0.25, 0.50, 0.30),
    }
}

impl Segment {
    fn empty() -> Self {
        Self {
            syllable_indices: Vec::new(),
            terminal: None,
            hesitation: false,
        }
    }
}

fn clamp_signed(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

fn clamp_unit(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn position_to_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
