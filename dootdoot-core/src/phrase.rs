//! Pure phrase-prosody planning for `VOICE_V2` performance metadata.

use crate::{
    LONG_PUNCTUATION_PAUSE_SAMPLES, MEDIUM_PUNCTUATION_PAUSE_SAMPLES, ProsodicPunctuation,
    SequenceEvent, SyllableEvent, WORD_PAUSE_SAMPLES,
};

const DECLINATION_STEP_SEMITONES: f64 = -0.28;
const CLAUSE_PITCH_RESET_SEMITONES: f64 = 0.45;
const SENTENCE_PITCH_RESET_SEMITONES: f64 = 1.20;
const SENTENCE_FINAL_LOWERING_SEMITONES: f64 = -0.90;
/// `VOICE_V9` (R1): a period falls all the way to a quiet settle.
const PERIOD_FINAL_LOWERING_SEMITONES: f64 = -1.40;
/// `VOICE_V9` (R1): an exclamation falls only shallowly from its raised,
/// emphasized peak, so it punches and stays energetic rather than closing.
const EXCLAMATION_FINAL_LOWERING_SEMITONES: f64 = -0.60;
const CLAUSE_LENGTHENING: f64 = 1.12;
const SENTENCE_LENGTHENING: f64 = 1.25;
const EMPHASIS_PERIOD: usize = 5;

/// Gives a pure phrase-prosody plan for voiced syllables.
#[derive(Debug, Clone, PartialEq)]
pub struct PhrasePlan {
    syllables: Vec<PhraseSyllablePlan>,
}

/// Gives deterministic phrase metadata for one voiced syllable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhraseSyllablePlan {
    syllable_index: usize,
    boundary_strength: PhraseBoundaryStrength,
    declination_offset_semitones: f64,
    pitch_reset_semitones: f64,
    final_lowering_semitones: f64,
    pre_boundary_lengthening: f64,
    pause_samples: u32,
    emphasized: bool,
}

/// Gives the phrase-boundary strength after a syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseBoundaryStrength {
    /// The next voiced syllable is a `WordPiece` continuation or does not
    /// exist.
    None,
    /// The next voiced syllable begins a separate word.
    Word,
    /// Punctuation creates a clause boundary.
    Clause,
    /// Punctuation creates a sentence boundary.
    Sentence,
}

#[derive(Debug, Clone, Copy)]
struct PendingSyllable {
    event: SyllableEvent,
    punctuation: Option<ProsodicPunctuation>,
    punctuation_pause_samples: u32,
}

impl PhrasePlan {
    /// Returns true when the input has no voiced syllables.
    pub fn is_empty(&self) -> bool {
        self.syllables.is_empty()
    }

    /// Returns per-syllable phrase metadata.
    pub fn syllables(&self) -> &[PhraseSyllablePlan] {
        &self.syllables
    }
}

impl PhraseSyllablePlan {
    /// Returns the voiced syllable index this metadata belongs to.
    pub fn syllable_index(&self) -> usize {
        self.syllable_index
    }

    /// Returns the boundary strength after this syllable.
    pub fn boundary_strength(&self) -> PhraseBoundaryStrength {
        self.boundary_strength
    }

    /// Returns the phrase-level pitch declination offset in semitones.
    pub fn declination_offset_semitones(&self) -> f64 {
        self.declination_offset_semitones
    }

    /// Returns the pitch reset offered to the next phrase segment.
    pub fn pitch_reset_semitones(&self) -> f64 {
        self.pitch_reset_semitones
    }

    /// Returns phrase-final lowering in semitones.
    pub fn final_lowering_semitones(&self) -> f64 {
        self.final_lowering_semitones
    }

    /// Returns the deterministic pre-boundary duration scale.
    pub fn pre_boundary_lengthening(&self) -> f64 {
        self.pre_boundary_lengthening
    }

    /// Returns pause samples planned after this syllable.
    pub fn pause_samples(&self) -> u32 {
        self.pause_samples
    }

    /// Returns true when this syllable receives sparse phrase emphasis.
    pub fn is_emphasized(&self) -> bool {
        self.emphasized
    }
}

/// Plans deterministic phrase-prosody metadata from sequencer events.
pub fn plan_phrase_prosody(events: &[SequenceEvent]) -> PhrasePlan {
    let pending = pending_syllables(events);
    let mut syllables = Vec::with_capacity(pending.len());
    let mut position_in_phrase = 0_usize;
    let mut syllables_since_sentence = 0_usize;

    for (index, syllable) in pending.iter().copied().enumerate() {
        let boundary_strength = boundary_strength(&pending, index);
        let punctuation = syllable.punctuation;
        let emphasized = syllables_since_sentence == 0
            || matches!(punctuation, Some(ProsodicPunctuation::Exclamation))
            || (syllables_since_sentence > 0
                && syllables_since_sentence.is_multiple_of(EMPHASIS_PERIOD));

        syllables.push(PhraseSyllablePlan {
            syllable_index: index,
            boundary_strength,
            declination_offset_semitones: declination_offset_semitones(position_in_phrase),
            pitch_reset_semitones: pitch_reset_semitones(boundary_strength),
            final_lowering_semitones: final_lowering_semitones(punctuation, boundary_strength),
            pre_boundary_lengthening: pre_boundary_lengthening(boundary_strength),
            pause_samples: pause_samples(syllable, boundary_strength),
            emphasized,
        });

        if boundary_strength == PhraseBoundaryStrength::Sentence {
            position_in_phrase = 0;
            syllables_since_sentence = 0;
        } else {
            position_in_phrase = position_in_phrase.saturating_add(1);
            syllables_since_sentence += 1;
        }
    }

    PhrasePlan { syllables }
}

fn pending_syllables(events: &[SequenceEvent]) -> Vec<PendingSyllable> {
    let mut syllables = Vec::new();

    for event in events {
        match event {
            SequenceEvent::Mood(_) | SequenceEvent::Complexity(_) | SequenceEvent::Archetype(_) => {
            }
            SequenceEvent::Syllable(syllable) => syllables.push(PendingSyllable {
                event: *syllable,
                punctuation: None,
                punctuation_pause_samples: 0,
            }),
            SequenceEvent::Punctuation(punctuation) => {
                if let Some(syllable) = syllables.last_mut() {
                    if syllable.punctuation.is_none() {
                        syllable.punctuation = Some(*punctuation);
                    }

                    syllable.punctuation_pause_samples = syllable
                        .punctuation_pause_samples
                        .max(punctuation.pause_samples());
                }
            }
        }
    }

    syllables
}

fn boundary_strength(syllables: &[PendingSyllable], index: usize) -> PhraseBoundaryStrength {
    if let Some(punctuation) = syllables[index].punctuation {
        return match punctuation {
            ProsodicPunctuation::Question
            | ProsodicPunctuation::Period
            | ProsodicPunctuation::Exclamation => PhraseBoundaryStrength::Sentence,
            ProsodicPunctuation::Comma
            | ProsodicPunctuation::Semicolon
            | ProsodicPunctuation::Colon => PhraseBoundaryStrength::Clause,
        };
    }

    if let Some(next_syllable) = syllables.get(index + 1) {
        if next_syllable.event.is_continuation() {
            PhraseBoundaryStrength::None
        } else {
            PhraseBoundaryStrength::Word
        }
    } else {
        PhraseBoundaryStrength::None
    }
}

fn pause_samples(syllable: PendingSyllable, boundary_strength: PhraseBoundaryStrength) -> u32 {
    if syllable.punctuation_pause_samples > 0 {
        return syllable.punctuation_pause_samples;
    }

    match boundary_strength {
        PhraseBoundaryStrength::None => 0,
        PhraseBoundaryStrength::Word => WORD_PAUSE_SAMPLES,
        PhraseBoundaryStrength::Clause => MEDIUM_PUNCTUATION_PAUSE_SAMPLES,
        PhraseBoundaryStrength::Sentence => LONG_PUNCTUATION_PAUSE_SAMPLES,
    }
}

fn declination_offset_semitones(position_in_phrase: usize) -> f64 {
    match position_in_phrase {
        0 => 0.0,
        1 => DECLINATION_STEP_SEMITONES,
        2 => -0.56,
        3 => -0.84,
        4 => -1.12,
        5 => -1.40,
        _ => {
            let Ok(position) = u32::try_from(position_in_phrase) else {
                return -1.40;
            };

            DECLINATION_STEP_SEMITONES * f64::from(position)
        }
    }
}

fn pitch_reset_semitones(boundary_strength: PhraseBoundaryStrength) -> f64 {
    match boundary_strength {
        PhraseBoundaryStrength::None | PhraseBoundaryStrength::Word => 0.0,
        PhraseBoundaryStrength::Clause => CLAUSE_PITCH_RESET_SEMITONES,
        PhraseBoundaryStrength::Sentence => SENTENCE_PITCH_RESET_SEMITONES,
    }
}

fn final_lowering_semitones(
    punctuation: Option<ProsodicPunctuation>,
    boundary_strength: PhraseBoundaryStrength,
) -> f64 {
    match (punctuation, boundary_strength) {
        (Some(ProsodicPunctuation::Question), PhraseBoundaryStrength::Sentence) => 0.0,
        (Some(ProsodicPunctuation::Period), PhraseBoundaryStrength::Sentence) => {
            PERIOD_FINAL_LOWERING_SEMITONES
        }
        (Some(ProsodicPunctuation::Exclamation), PhraseBoundaryStrength::Sentence) => {
            EXCLAMATION_FINAL_LOWERING_SEMITONES
        }
        (_, PhraseBoundaryStrength::Sentence) => SENTENCE_FINAL_LOWERING_SEMITONES,
        // VOICE_V9 (R4): a clause boundary (and word/none) carries no lowering.
        // A clause's "more coming" tone is its continuation rise (the syllable's
        // final glide); a lowering here would override that rise at the tail.
        _ => 0.0,
    }
}

fn pre_boundary_lengthening(boundary_strength: PhraseBoundaryStrength) -> f64 {
    match boundary_strength {
        PhraseBoundaryStrength::None | PhraseBoundaryStrength::Word => 1.0,
        PhraseBoundaryStrength::Clause => CLAUSE_LENGTHENING,
        PhraseBoundaryStrength::Sentence => SENTENCE_LENGTHENING,
    }
}
