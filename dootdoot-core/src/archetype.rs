//! Deterministic `FORMAT_V2` gesture-archetype selection.

use crate::{ProsodicPunctuation, SequenceEvent, UtteranceMood};

const STUTTER_COMPLEXITY_THRESHOLD: f64 = 0.58;
const SEASONING_COMPLEXITY_THRESHOLD: f64 = 0.65;
const NEGATIVE_VALENCE_THRESHOLD: f64 = -0.20;
const POSITIVE_VALENCE_THRESHOLD: f64 = 0.30;
const HIGH_AROUSAL_THRESHOLD: f64 = 0.55;
const YELP_AROUSAL_THRESHOLD: f64 = 0.45;
const TREMBLE_AROUSAL_THRESHOLD: f64 = 0.62;

/// Gives the bounded `FORMAT_V2` gesture-archetype palette.
pub const GESTURE_ARCHETYPE_PALETTE: [GestureArchetype; 5] = [
    GestureArchetype::Chatter,
    GestureArchetype::Yelp,
    GestureArchetype::Moan,
    GestureArchetype::StutterBurst,
    GestureArchetype::Tremble,
];

/// Gives one bounded gesture family selected for a syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureArchetype {
    /// Quick neutral BB-8 chatter.
    Chatter,
    /// Bright high-register yelp.
    Yelp,
    /// Lower, slower negative-valence moan.
    Moan,
    /// Compound complexity-driven stutter/burst.
    StutterBurst,
    /// Nervous high-arousal tremble.
    Tremble,
}

/// Gives one selected archetype row for a voiced syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchetypeSelection {
    syllable_index: usize,
    archetype: GestureArchetype,
    servo_seasoning: bool,
    noise_tail: bool,
}

#[derive(Debug, Clone, Copy)]
struct PendingSyllable {
    punctuation: Option<ProsodicPunctuation>,
}

/// Plans deterministic gesture archetypes from sequencer events.
pub fn plan_gesture_archetypes(events: &[SequenceEvent]) -> Vec<ArchetypeSelection> {
    let mood = mood_from_events(events);
    let complexity = complexity_from_events(events);
    let pending = pending_syllables(events);
    let mut selections = Vec::with_capacity(pending.len());
    let mut phrase_position = 0_usize;

    for (syllable_index, syllable) in pending.iter().copied().enumerate() {
        selections.push(select_archetype(
            syllable_index,
            phrase_position,
            mood,
            complexity,
            syllable.punctuation,
        ));

        if matches!(
            syllable.punctuation,
            Some(
                ProsodicPunctuation::Question
                    | ProsodicPunctuation::Period
                    | ProsodicPunctuation::Exclamation,
            ),
        ) {
            phrase_position = 0;
        } else {
            phrase_position = phrase_position.saturating_add(1);
        }
    }

    selections
}

impl ArchetypeSelection {
    /// Returns the voiced syllable index this selection belongs to.
    pub fn syllable_index(&self) -> usize {
        self.syllable_index
    }

    /// Returns the selected gesture family.
    pub fn archetype(&self) -> GestureArchetype {
        self.archetype
    }

    /// Returns true when sparse servo texture should be layered in.
    pub fn servo_seasoning(&self) -> bool {
        self.servo_seasoning
    }

    /// Returns true when sparse noise-tail texture should be layered in.
    pub fn noise_tail(&self) -> bool {
        self.noise_tail
    }
}

fn select_archetype(
    syllable_index: usize,
    phrase_position: usize,
    mood: UtteranceMood,
    complexity: f64,
    punctuation: Option<ProsodicPunctuation>,
) -> ArchetypeSelection {
    let archetype = if mood.valence() < NEGATIVE_VALENCE_THRESHOLD
        && mood.arousal() >= HIGH_AROUSAL_THRESHOLD
    {
        GestureArchetype::Tremble
    } else if matches!(punctuation, Some(ProsodicPunctuation::Exclamation))
        || (mood.valence() > POSITIVE_VALENCE_THRESHOLD && mood.arousal() >= YELP_AROUSAL_THRESHOLD)
    {
        GestureArchetype::Yelp
    } else if complexity >= STUTTER_COMPLEXITY_THRESHOLD {
        GestureArchetype::StutterBurst
    } else if mood.valence() < NEGATIVE_VALENCE_THRESHOLD {
        GestureArchetype::Moan
    } else if mood.arousal() >= TREMBLE_AROUSAL_THRESHOLD {
        GestureArchetype::Tremble
    } else {
        GestureArchetype::Chatter
    };
    let servo_seasoning =
        complexity >= SEASONING_COMPLEXITY_THRESHOLD && phrase_position.is_multiple_of(2);
    let noise_tail =
        is_sentence_punctuation(punctuation) && mood.arousal() >= HIGH_AROUSAL_THRESHOLD;

    ArchetypeSelection {
        syllable_index,
        archetype,
        servo_seasoning,
        noise_tail,
    }
}

fn pending_syllables(events: &[SequenceEvent]) -> Vec<PendingSyllable> {
    let mut syllables = Vec::new();

    for event in events {
        match event {
            SequenceEvent::Mood(_) | SequenceEvent::Complexity(_) => {}
            SequenceEvent::Syllable(_) => syllables.push(PendingSyllable { punctuation: None }),
            SequenceEvent::Punctuation(punctuation) => {
                if let Some(syllable) = syllables.last_mut()
                    && syllable.punctuation.is_none()
                {
                    syllable.punctuation = Some(*punctuation);
                }
            }
        }
    }

    syllables
}

fn mood_from_events(events: &[SequenceEvent]) -> UtteranceMood {
    events
        .iter()
        .find_map(|event| match event {
            SequenceEvent::Mood(mood) => Some(*mood),
            SequenceEvent::Complexity(_)
            | SequenceEvent::Syllable(_)
            | SequenceEvent::Punctuation(_) => None,
        })
        .unwrap_or_else(|| UtteranceMood::new(0.0, 0.0))
}

fn complexity_from_events(events: &[SequenceEvent]) -> f64 {
    events
        .iter()
        .find_map(|event| match event {
            SequenceEvent::Complexity(complexity) => Some(complexity.scalar()),
            SequenceEvent::Mood(_) | SequenceEvent::Syllable(_) | SequenceEvent::Punctuation(_) => {
                None
            }
        })
        .unwrap_or(0.0)
}

fn is_sentence_punctuation(punctuation: Option<ProsodicPunctuation>) -> bool {
    matches!(
        punctuation,
        Some(
            ProsodicPunctuation::Question
                | ProsodicPunctuation::Period
                | ProsodicPunctuation::Exclamation,
        ),
    )
}
