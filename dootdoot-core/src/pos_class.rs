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

/// Classifies one whole lexical word (case-insensitively).
///
/// With the `spike-noun-verb` gate off this is the constant
/// [`PosClass::Other`], keeping every rendered path byte-identical. The
/// `cfg!` keeps the lexicon compiled either way; the branch constant-folds.
pub(crate) fn word_pos_class(word: &str) -> PosClass {
    if cfg!(feature = "spike-noun-verb") {
        spike_lexicon_class(word, SPIKE_AMBIGUITY_POLICY)
    } else {
        PosClass::Other
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

/// Gives the spike policy for noun/verb-ambiguous lemmas.
///
/// The research §5.1 corpus check found ambiguity — not coverage — is the
/// dominant cost on coding text (`build`, `fix`, `run`, `update`, `sync` are
/// all >40% minority-class use), so T-118 A/Bs both policies by ear before the
/// contract locks one in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AmbiguityPolicy {
    /// Mark an ambiguous lemma with its dominant class — consistent, sometimes
    /// grammatically wrong. The T-118 round-1 A/B preferred the conservative
    /// policy; this variant stays as the flip-back lever (and for tests).
    #[allow(
        dead_code,
        reason = "the T-118 A/B lever: constructed only when \
                  SPIKE_AMBIGUITY_POLICY is flipped locally (and in tests)"
    )]
    DominantClass,
    /// Leave ambiguous lemmas unmarked (`Other`) — conservative, but unmarks
    /// much of the highest-frequency coding vocabulary.
    FallBackToOther,
}

/// Gives the active spike ambiguity policy.
///
/// The T-118 round-1 by-ear A/B preferred the conservative leg: ambiguous
/// lemmas stay unmarked (`Other`) even though that leaves much of the
/// highest-frequency coding vocabulary (`build`, `fix`, `run`, `update`,
/// `sync`) as plain blips. Flip to `DominantClass` to re-hear the other leg.
const SPIKE_AMBIGUITY_POLICY: AmbiguityPolicy = AmbiguityPolicy::FallBackToOther;

/// Gives the spike's unambiguous coding-domain nouns, sorted for binary
/// search.
///
/// Drawn from the research §5.1 commit-corpus top lemmas: the register is
/// noun-heavy, so nouns and verbs are roughly balanced here rather than
/// verb-weighted.
const SPIKE_NOUNS: [&str; 30] = [
    "api",
    "asset",
    "branch",
    "bug",
    "client",
    "code",
    "config",
    "crate",
    "data",
    "database",
    "dependency",
    "doc",
    "endpoint",
    "error",
    "feature",
    "file",
    "function",
    "module",
    "output",
    "package",
    "path",
    "schema",
    "script",
    "server",
    "service",
    "token",
    "user",
    "value",
    "version",
    "workflow",
];

/// Gives the spike's unambiguous coding-domain verbs, sorted for binary
/// search.
const SPIKE_VERBS: [&str; 30] = [
    "add",
    "avoid",
    "create",
    "delete",
    "deploy",
    "ensure",
    "extract",
    "fail",
    "fetch",
    "generate",
    "handle",
    "implement",
    "improve",
    "install",
    "load",
    "parse",
    "prevent",
    "refactor",
    "remove",
    "rename",
    "render",
    "replace",
    "resolve",
    "revert",
    "send",
    "simplify",
    "skip",
    "validate",
    "verify",
    "write",
];

/// Gives the spike's noun/verb-ambiguous coding lemmas with their dominant
/// class, sorted by lemma for binary search.
///
/// These are exactly the zero-derivation words the §5.1 corpus check flagged
/// as the core of the register ("the build" / "to build").
const SPIKE_AMBIGUOUS: [(&str, PosClass); 20] = [
    ("build", PosClass::Verb),
    ("bump", PosClass::Verb),
    ("cache", PosClass::Noun),
    ("check", PosClass::Verb),
    ("commit", PosClass::Noun),
    ("filter", PosClass::Noun),
    ("fix", PosClass::Verb),
    ("flag", PosClass::Noun),
    ("gate", PosClass::Noun),
    ("index", PosClass::Noun),
    ("log", PosClass::Noun),
    ("merge", PosClass::Verb),
    ("move", PosClass::Verb),
    ("queue", PosClass::Noun),
    ("release", PosClass::Noun),
    ("run", PosClass::Verb),
    ("share", PosClass::Verb),
    ("sync", PosClass::Verb),
    ("test", PosClass::Noun),
    ("update", PosClass::Verb),
];

/// Looks one lowercased word up in the hard-coded spike lexicon.
fn spike_lexicon_class(word: &str, policy: AmbiguityPolicy) -> PosClass {
    let word = word.to_ascii_lowercase();

    if let Ok(index) = SPIKE_AMBIGUOUS.binary_search_by(|(lemma, _)| lemma.cmp(&word.as_str())) {
        return match policy {
            AmbiguityPolicy::DominantClass => SPIKE_AMBIGUOUS[index].1,
            AmbiguityPolicy::FallBackToOther => PosClass::Other,
        };
    }

    if SPIKE_NOUNS.binary_search(&word.as_str()).is_ok() {
        return PosClass::Noun;
    }

    if SPIKE_VERBS.binary_search(&word.as_str()).is_ok() {
        return PosClass::Verb;
    }

    PosClass::Other
}

#[cfg(test)]
mod tests;
