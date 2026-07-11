//! Word-level part-of-speech classes for the `VOICE_V12` noun/verb spike.
//!
//! The POS class is a property of the **lexical word**, not of independent
//! `WordPiece` tokens: the word-initial token establishes the class and
//! continuation tokens inherit it, so a multi-subword word receives one
//! coherent onset mark and one final resolution shape.
//!
//! Classification is behind the local, default-off `spike-noun-verb` compile
//! gate (T-115). With the gate off, [`word_pos_class`] returns
//! [`PosClass::Other`] for every word, every class-conditioned path downstream
//! stays inert, and rendered audio is byte-identical to `VOICE_V11`. The gate
//! is a spike aid, not a user-facing alternate voice; the shipped follow-up
//! (T-120/T-121) replaces the hard-coded lexicon with a baked class table.

use crate::KnobSet;

/// Gives one word-level part-of-speech class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PosClass {
    /// A content noun: marked with the broadband click/pop onset and the
    /// settling resolution silhouette.
    Noun,
    /// A content verb: marked with the up-swept chirp onset and the pushing
    /// resolution silhouette.
    Verb,
    /// Everything else — function words, punctuation, unknown vocabulary.
    /// Renders exactly as `VOICE_V11` does.
    #[default]
    Other,
}

impl PosClass {
    /// Returns true when this class marks a content word (noun or verb).
    pub fn is_content(self) -> bool {
        matches!(self, Self::Noun | Self::Verb)
    }
}

/// Gives the noun settle's pitch step down in knob space.
const NOUN_SETTLE_PITCH_STEP: f64 = -0.28;

/// Gives how much of the stem vowel survives the noun settle.
const NOUN_SETTLE_VOWEL_KEEP: f64 = 0.35;

/// Gives the noun settle's pull toward the rounded `oo` locus (`+1`).
const NOUN_SETTLE_VOWEL_ROUND: f64 = 0.65;

/// Gives the noun settle's contour flattening factor.
const NOUN_SETTLE_CONTOUR_FLATTEN: f64 = 0.15;

/// Gives the noun settle's warble calming factor (a steadier, sustained
/// tail).
const NOUN_SETTLE_WARBLE_CALM: f64 = 0.5;

/// Gives the verb push's pitch step up in knob space.
const VERB_PUSH_PITCH_STEP: f64 = 0.18;

/// Gives how much of the stem vowel survives the verb push.
const VERB_PUSH_VOWEL_KEEP: f64 = 0.35;

/// Gives the verb push's pull toward the bright `ee` locus (`-1`).
const VERB_PUSH_VOWEL_BRIGHTEN: f64 = -0.65;

/// Gives the verb push's guaranteed rising-contour floor.
const VERB_PUSH_CONTOUR_BASE: f64 = 0.70;

/// Gives how much of the stem contour survives the verb push.
const VERB_PUSH_CONTOUR_KEEP: f64 = 0.30;

/// Gives how much of the stem warble survives the verb push.
const VERB_PUSH_WARBLE_KEEP: f64 = 0.6;

/// Gives the verb push's added liveliness on the warble axis.
const VERB_PUSH_WARBLE_LIVEN: f64 = 0.25;

/// Derives the class-resolution syllable's knobs from the stem's own knobs
/// (`VOICE_V12`, T-117).
///
/// The resolution is a **frozen per-class transform** — never random padding —
/// so every noun shares a "settling" shape (vowel rounds toward `oo`, pitch
/// steps down, contour flattens, steadier tail) and every verb a "pushing" one
/// (brighter toward `ee`, rising/gliding continuation), while individual words
/// still differ by their stem knobs. `Other` returns the stem unchanged. All
/// axes clamp to [`crate::KNOB_BOUNDS`].
pub fn class_resolution_knobs(stem: KnobSet, pos_class: PosClass) -> KnobSet {
    match pos_class {
        PosClass::Noun => KnobSet::from_axes([
            stem.pitch_center() + NOUN_SETTLE_PITCH_STEP,
            (stem.vowel_position() * NOUN_SETTLE_VOWEL_KEEP) + NOUN_SETTLE_VOWEL_ROUND,
            stem.contour() * NOUN_SETTLE_CONTOUR_FLATTEN,
            stem.warble_depth() * NOUN_SETTLE_WARBLE_CALM,
        ]),
        PosClass::Verb => KnobSet::from_axes([
            stem.pitch_center() + VERB_PUSH_PITCH_STEP,
            (stem.vowel_position() * VERB_PUSH_VOWEL_KEEP) + VERB_PUSH_VOWEL_BRIGHTEN,
            VERB_PUSH_CONTOUR_BASE + (stem.contour() * VERB_PUSH_CONTOUR_KEEP),
            (stem.warble_depth() * VERB_PUSH_WARBLE_KEEP) + VERB_PUSH_WARBLE_LIVEN,
        ]),
        PosClass::Other => stem,
    }
}

#[cfg(test)]
mod tests;
