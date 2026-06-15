//! Deterministic `VOICE_V7` discourse-performance planning.
//!
//! The planner runs after tokenization and before synthesis. It is a pure
//! function of the sequencer event stream: it segments voiced syllables into
//! discourse phrases by punctuation and hesitation timing, assigns each phrase
//! a local role, and emits bounded continuous performance curves per syllable.
//! The synthesis stage (wired in a later task) reads these to deploy the
//! `VOICE_V7` primitives by role rather than applying one global affect to
//! every syllable.

use crate::{ProsodicPunctuation, SequenceEvent, SyllableEvent};

/// Gives one local discourse role for a phrase of voiced syllables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseRole {
    /// A leading or question gesture: longer, rising, less dense.
    Probe,
    /// A conversational answer after a reset: shorter, denser, varied.
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

        for (position, syllable_index) in segment.syllable_indices.iter().copied().enumerate() {
            let fraction = if length <= 1 {
                0.0
            } else {
                position_to_f64(position) / position_to_f64(length - 1)
            };

            syllables.push(PerformanceSyllable {
                syllable_index,
                role,
                curves: role_curves(role, fraction),
            });
        }
    }

    PerformancePlan { syllables }
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
    let timing = syllable.timing();

    timing.bridge_suppressed() && timing.pause_override().is_some()
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
        PhraseRole::ChattyReply => PerformanceCurves::new(0.0, 0.10, 0.0, 0.20, 0.45, 0.40, 0.40),
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
