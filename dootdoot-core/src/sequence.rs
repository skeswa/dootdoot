//! Utterance sequencing for rendered droid syllables.

use crate::{
    KnobSet, LEADING_SILENCE_SAMPLES, LONG_PUNCTUATION_PAUSE_SAMPLES,
    MEDIUM_PUNCTUATION_PAUSE_SAMPLES, TRAILING_SILENCE_SAMPLES, WORD_PAUSE_SAMPLES,
    pitch_center_hz,
    synth::{SyllableFinalGlide, render_syllable_with_final_glide},
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

/// Gives one input event consumed by the utterance sequencer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SequenceEvent {
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
    final_glide: SyllableFinalGlide,
    punctuation_pause_samples: Option<u32>,
    punctuation_seen: bool,
}

impl SequenceEvent {
    /// Builds a voiced syllable event.
    pub fn syllable(knobs: KnobSet, continuation: bool) -> Self {
        Self::Syllable(SyllableEvent::new(knobs, continuation))
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
        }
    }

    /// Returns the semantic knobs for this syllable.
    pub fn knobs(&self) -> KnobSet {
        self.knobs
    }

    /// Returns true when this syllable continues the previous wordpiece.
    pub fn is_continuation(&self) -> bool {
        self.continuation
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
            Self::Comma | Self::Semicolon | Self::Colon => SyllableFinalGlide::Neutral,
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
    ));
    append_silence(&mut samples, TRAILING_SILENCE_SAMPLES);

    samples
}

/// Lays out voiced syllables and control punctuation into an utterance.
pub fn sequence_utterance(events: &[SequenceEvent]) -> SequencedUtterance {
    let mut plans = Vec::new();

    for event in events {
        match event {
            SequenceEvent::Syllable(syllable) => plans.push(SyllablePlan::new(*syllable)),
            SequenceEvent::Punctuation(punctuation) => {
                if let Some(plan) = plans.last_mut() {
                    plan.attach_punctuation(*punctuation);
                }
            }
        }
    }

    if plans.is_empty() {
        return SequencedUtterance::EmptyChirp(render_empty_chirp());
    }

    let mut samples = Vec::new();
    let mut previous_pitch_hz = None;

    append_silence(&mut samples, LEADING_SILENCE_SAMPLES);

    for (index, plan) in plans.iter().copied().enumerate() {
        let syllable = plan.event;
        let target_pitch_hz = pitch_center_hz(syllable.knobs().pitch_center());
        let start_pitch_hz = match previous_pitch_hz {
            Some(previous_pitch_hz) => previous_pitch_hz,
            None => target_pitch_hz,
        };

        samples.extend(render_syllable_with_final_glide(
            syllable.knobs(),
            start_pitch_hz,
            plan.final_glide,
        ));
        previous_pitch_hz = Some(target_pitch_hz);

        if let Some(pause_samples) = plan.punctuation_pause_samples {
            append_silence(&mut samples, pause_samples);
        } else if let Some(next_plan) = plans.get(index + 1)
            && !next_plan.event.is_continuation()
        {
            append_silence(&mut samples, WORD_PAUSE_SAMPLES);
        }
    }

    append_silence(&mut samples, TRAILING_SILENCE_SAMPLES);

    SequencedUtterance::Samples(samples)
}

fn append_silence(samples: &mut Vec<f64>, count: u32) {
    for _ in 0..count {
        samples.push(0.0);
    }
}

impl SyllablePlan {
    fn new(event: SyllableEvent) -> Self {
        Self {
            event,
            final_glide: SyllableFinalGlide::Neutral,
            punctuation_pause_samples: None,
            punctuation_seen: false,
        }
    }

    fn attach_punctuation(&mut self, punctuation: ProsodicPunctuation) {
        if !self.punctuation_seen {
            self.final_glide = punctuation.final_glide();
            self.punctuation_seen = true;
        }

        let pause_samples = punctuation.pause_samples();
        self.punctuation_pause_samples = Some(match self.punctuation_pause_samples {
            Some(existing_pause_samples) => existing_pause_samples.max(pause_samples),
            None => pause_samples,
        });
    }
}
