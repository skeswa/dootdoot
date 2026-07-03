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

/// Gives the spike policy for noun/verb-ambiguous lemmas.
///
/// The research §5.1 corpus check found ambiguity — not coverage — is the
/// dominant cost on coding text (`build`, `fix`, `run`, `update`, `sync` are
/// all >40% minority-class use), so T-118 A/Bs both policies by ear before the
/// contract locks one in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AmbiguityPolicy {
    /// Mark an ambiguous lemma with its dominant class — consistent, sometimes
    /// grammatically wrong.
    DominantClass,
    /// Leave ambiguous lemmas unmarked (`Other`) — conservative, but unmarks
    /// much of the highest-frequency coding vocabulary.
    #[allow(
        dead_code,
        reason = "the T-118 A/B lever: constructed only when \
                  SPIKE_AMBIGUITY_POLICY is flipped locally (and in tests)"
    )]
    FallBackToOther,
}

/// Gives the active spike ambiguity policy — the local A/B lever for T-118.
///
/// Flip to `FallBackToOther` and re-render to hear the conservative leg.
const SPIKE_AMBIGUITY_POLICY: AmbiguityPolicy = AmbiguityPolicy::DominantClass;

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
