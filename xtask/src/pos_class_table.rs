//! `VOICE_V12` class-table derivation from the pinned tagged-counts snapshot.
//!
//! The committed snapshot (`assets/pos/tagged_counts.tsv`, produced by
//! `scripts/derive_pos_table.py` from the pinned `CommitChronicle` shard)
//! carries raw per-word statistics: `surface<TAB>lemma<TAB>bucket<TAB>count`.
//! This module applies the classification **policy** the `VOICE_V12` contract
//! locked (FR-114/FR-115): dominant class per lemma, the closed-class
//! override, the conservative ambiguity fallback (the T-118 A/B winner),
//! coding-domain frequency ranking, and lemma→surface expansion. Everything is
//! deterministic and pure so regeneration is reproducible.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Result, SourceManifestError};

/// Gives one word class carried by the baked table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PosTableClass {
    /// A content noun.
    Noun,
    /// A content verb.
    Verb,
}

/// Gives one derived class-table entry: a surface word and its class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosClassEntry {
    surface: String,
    pos_class: PosTableClass,
}

/// Holds the parsed tagged-counts snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosSnapshot {
    rows: Vec<SnapshotRow>,
}

/// Configures the derivation policy thresholds.
///
/// Shares are expressed in whole percent and compared with exact integer
/// arithmetic, keeping the policy free of float rounding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosPolicyConfig {
    top_lemmas_per_class: usize,
    ambiguity_minority_max_percent: u64,
    content_dominance_min_percent: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnapshotRow {
    surface: String,
    lemma: String,
    bucket: Bucket,
    count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bucket {
    Noun,
    Verb,
    Other,
}

#[derive(Debug, Clone, Copy, Default)]
struct BucketCounts {
    noun: u64,
    verb: u64,
    other: u64,
}

/// Gives the closed-class/function words that never enter the table, whatever
/// the tagger reported (FR-115). Sorted for binary search.
const CLOSED_CLASS_WORDS: [&str; 97] = [
    "a", "about", "above", "after", "again", "against", "all", "also", "am", "an", "and", "any",
    "are", "as", "at", "be", "because", "been", "before", "being", "below", "between", "both",
    "but", "by", "can", "could", "did", "do", "does", "done", "down", "during", "each", "else",
    "few", "for", "from", "further", "had", "has", "have", "having", "he", "her", "here", "his",
    "how", "i", "if", "in", "into", "is", "it", "its", "just", "may", "me", "might", "more",
    "most", "must", "my", "no", "nor", "not", "of", "off", "on", "once", "only", "or", "other",
    "our", "out", "over", "own", "shall", "she", "should", "so", "some", "such", "than", "that",
    "the", "their", "them", "then", "there", "these", "they", "this", "those", "through", "to",
    "too",
];

/// Gives the closed-class words above `t` (split keeps each array literal
/// readable). Sorted for binary search.
const CLOSED_CLASS_WORDS_TAIL: [&str; 16] = [
    "under", "until", "up", "us", "very", "was", "we", "were", "what", "when", "where", "which",
    "while", "who", "why", "will",
];

impl PosClassEntry {
    /// Returns the surface word this entry classifies.
    #[must_use]
    pub fn surface(&self) -> &str {
        &self.surface
    }

    /// Returns the class this entry carries.
    #[must_use]
    pub fn pos_class(&self) -> PosTableClass {
        self.pos_class
    }
}

impl PosSnapshot {
    /// Returns the number of parsed statistic rows.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

impl PosPolicyConfig {
    /// Returns a copy with a different per-class lemma cap.
    #[must_use]
    pub fn with_top_lemmas_per_class(mut self, top_lemmas_per_class: usize) -> Self {
        self.top_lemmas_per_class = top_lemmas_per_class;

        self
    }
}

impl Default for PosPolicyConfig {
    /// Gives the shipped `VOICE_V12` policy: top 2,000 lemmas per class
    /// (research §5.1: domain top-2,000-each ≈ 95% coverage), ambiguous
    /// lemmas excluded when the minority class exceeds 25% of content use,
    /// and words classed only when noun+verb use dominates overall use.
    fn default() -> Self {
        Self {
            top_lemmas_per_class: 2_000,
            ambiguity_minority_max_percent: 25,
            content_dominance_min_percent: 50,
        }
    }
}

/// Parses the tagged-counts snapshot TSV.
///
/// # Errors
///
/// Returns an error when a row is not `surface<TAB>lemma<TAB>bucket<TAB>count`
/// with a known bucket and an integer count.
pub fn parse_tagged_counts(input: &str) -> Result<PosSnapshot> {
    let mut rows = Vec::new();

    for (line_number, line) in input.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut fields = line.split('\t');
        let (Some(surface), Some(lemma), Some(bucket), Some(count), None) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            return Err(SourceManifestError::new(format!(
                "tagged-counts line {} should have 4 tab-separated fields",
                line_number + 1,
            )));
        };
        let bucket = match bucket {
            "noun" => Bucket::Noun,
            "verb" => Bucket::Verb,
            "other" => Bucket::Other,
            unknown => {
                return Err(SourceManifestError::new(format!(
                    "tagged-counts line {} has unknown bucket {unknown}",
                    line_number + 1,
                )));
            }
        };
        let count = count.parse::<u64>().map_err(|error| {
            SourceManifestError::new(format!(
                "tagged-counts line {} has a non-integer count: {error}",
                line_number + 1,
            ))
        })?;

        rows.push(SnapshotRow {
            surface: surface.to_owned(),
            lemma: lemma.to_owned(),
            bucket,
            count,
        });
    }

    Ok(PosSnapshot { rows })
}

/// Derives the baked class table from a snapshot under the locked policy.
///
/// Entries come out sorted by surface, one class per surface; surfaces whose
/// evidence is ambiguous, contradicts their lemma, or is claimed by lemmas of
/// conflicting classes are dropped (the conservative FR-115 posture).
#[must_use]
pub fn derive_pos_class_table(
    snapshot: &PosSnapshot,
    config: &PosPolicyConfig,
) -> Vec<PosClassEntry> {
    let lemma_counts = aggregate(snapshot, |row| row.lemma.clone());
    let surface_counts = aggregate(snapshot, |row| row.surface.clone());
    let kept_lemmas = rank_lemmas(&lemma_counts, config);
    let mut surface_classes: BTreeMap<String, BTreeSet<PosTableClass>> = BTreeMap::new();

    for row in &snapshot.rows {
        let Some(lemma_class) = kept_lemmas.get(&row.lemma) else {
            continue;
        };
        let Some(surface_class) = classify(
            surface_counts
                .get(&row.surface)
                .copied()
                .unwrap_or_default(),
            config,
        ) else {
            continue;
        };

        if surface_class == *lemma_class && !is_closed_class(&row.surface) {
            surface_classes
                .entry(row.surface.clone())
                .or_default()
                .insert(surface_class);
        }
    }

    surface_classes
        .into_iter()
        .filter_map(|(surface, classes)| {
            // A surface reachable from lemmas of conflicting classes is
            // dropped rather than guessed.
            if classes.len() == 1 {
                classes.first().map(|pos_class| PosClassEntry {
                    surface,
                    pos_class: *pos_class,
                })
            } else {
                None
            }
        })
        .collect()
}

fn aggregate(
    snapshot: &PosSnapshot,
    key: impl Fn(&SnapshotRow) -> String,
) -> BTreeMap<String, BucketCounts> {
    let mut totals: BTreeMap<String, BucketCounts> = BTreeMap::new();

    for row in &snapshot.rows {
        let counts = totals.entry(key(row)).or_default();

        match row.bucket {
            Bucket::Noun => counts.noun += row.count,
            Bucket::Verb => counts.verb += row.count,
            Bucket::Other => counts.other += row.count,
        }
    }

    totals
}

/// Classifies one aggregate under the policy, or `None` when the evidence is
/// closed-class-free but insufficient (ambiguous or non-content-dominated).
fn classify(counts: BucketCounts, config: &PosPolicyConfig) -> Option<PosTableClass> {
    let content = counts.noun + counts.verb;

    if content == 0 {
        return None;
    }

    // content / total must exceed the dominance threshold (exact integers).
    let total = content + counts.other;

    if content * 100 <= total * config.content_dominance_min_percent {
        return None;
    }

    // minority / content must not exceed the ambiguity threshold.
    let minority = counts.noun.min(counts.verb);

    if minority * 100 > content * config.ambiguity_minority_max_percent {
        return None;
    }

    if counts.noun >= counts.verb {
        Some(PosTableClass::Noun)
    } else {
        Some(PosTableClass::Verb)
    }
}

/// Ranks classifiable lemmas by content frequency and keeps the top N of each
/// class. Ties break lexicographically so the ranking is deterministic.
fn rank_lemmas(
    lemma_counts: &BTreeMap<String, BucketCounts>,
    config: &PosPolicyConfig,
) -> BTreeMap<String, PosTableClass> {
    let mut ranked: Vec<(&String, BucketCounts, PosTableClass)> = lemma_counts
        .iter()
        .filter(|(lemma, _counts)| !is_closed_class(lemma))
        .filter_map(|(lemma, counts)| {
            classify(*counts, config).map(|pos_class| (lemma, *counts, pos_class))
        })
        .collect();

    ranked.sort_by(|left, right| {
        let left_content = left.1.noun + left.1.verb;
        let right_content = right.1.noun + right.1.verb;

        right_content.cmp(&left_content).then(left.0.cmp(right.0))
    });

    let mut kept = BTreeMap::new();
    let mut noun_count = 0_usize;
    let mut verb_count = 0_usize;

    for (lemma, _counts, pos_class) in ranked {
        let taken = match pos_class {
            PosTableClass::Noun => &mut noun_count,
            PosTableClass::Verb => &mut verb_count,
        };

        if *taken < config.top_lemmas_per_class {
            *taken += 1;
            kept.insert(lemma.clone(), pos_class);
        }
    }

    kept
}

fn is_closed_class(word: &str) -> bool {
    CLOSED_CLASS_WORDS.binary_search(&word).is_ok()
        || CLOSED_CLASS_WORDS_TAIL.binary_search(&word).is_ok()
}
