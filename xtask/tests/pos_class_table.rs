//! Tests for the `VOICE_V12` class-table derivation policy (T-120, FR-114/115).
//!
//! The committed tagged-counts snapshot carries raw per-word statistics; this
//! policy turns them into the baked noun/verb table: dominant class per lemma,
//! closed-class override, the conservative ambiguity fallback the T-118 A/B
//! locked, coding-domain ranking, and lemma→surface expansion.

use xtask::{PosPolicyConfig, PosTableClass, derive_pos_class_table, parse_tagged_counts};

fn snapshot(rows: &str) -> xtask::PosSnapshot {
    parse_tagged_counts(rows).expect("test snapshot parses")
}

#[test]
fn parse_skips_comments_and_rejects_malformed_rows() {
    let parsed = parse_tagged_counts("# comment\nbug\tbug\tnoun\t100\n");

    assert_eq!(parsed.expect("valid TSV parses").row_count(), 1);
    assert!(parse_tagged_counts("bug\tbug\tnoun\n").is_err());
    assert!(parse_tagged_counts("bug\tbug\tadjective\t3\n").is_err());
    assert!(parse_tagged_counts("bug\tbug\tnoun\tmany\n").is_err());
}

#[test]
fn dominant_content_lemmas_classify_by_their_dominant_class() {
    let table = derive_pos_class_table(
        &snapshot("bug\tbug\tnoun\t90\nbug\tbug\tverb\t5\nverify\tverify\tverb\t80\n"),
        &PosPolicyConfig::default(),
    );
    let classes = table
        .iter()
        .map(|entry| (entry.surface(), entry.pos_class()))
        .collect::<Vec<_>>();

    assert_eq!(
        classes,
        [
            ("bug", PosTableClass::Noun),
            ("verify", PosTableClass::Verb),
        ]
    );
}

#[test]
fn ambiguous_lemmas_fall_back_to_unclassified() {
    // 48/52 noun-verb split (the `build` case): conservative policy drops it.
    let table = derive_pos_class_table(
        &snapshot("build\tbuild\tnoun\t48\nbuild\tbuild\tverb\t52\n"),
        &PosPolicyConfig::default(),
    );

    assert!(table.is_empty());
}

#[test]
fn closed_class_words_are_always_excluded() {
    // Even with (mis-)tagged content-dominant counts, function words stay out.
    let table = derive_pos_class_table(
        &snapshot("can\tcan\tverb\t500\nwill\twill\tverb\t400\nthe\tthe\tnoun\t50\n"),
        &PosPolicyConfig::default(),
    );

    assert!(table.is_empty());
}

#[test]
fn words_dominated_by_other_usage_are_excluded() {
    // `light` used mostly as an adjective: content share too low to class.
    let table = derive_pos_class_table(
        &snapshot("light\tlight\tnoun\t20\nlight\tlight\tother\t80\n"),
        &PosPolicyConfig::default(),
    );

    assert!(table.is_empty());
}

#[test]
fn surfaces_inherit_the_lemma_class() {
    let table = derive_pos_class_table(
        &snapshot("bug\tbug\tnoun\t90\nbugs\tbug\tnoun\t40\n"),
        &PosPolicyConfig::default(),
    );
    let surfaces = table
        .iter()
        .map(xtask::PosClassEntry::surface)
        .collect::<Vec<_>>();

    assert_eq!(surfaces, ["bug", "bugs"]);
    assert!(
        table
            .iter()
            .all(|entry| entry.pos_class() == PosTableClass::Noun)
    );
}

#[test]
fn surfaces_contradicting_their_lemma_are_dropped() {
    // The lemma is a clear verb overall, but one surface's own evidence is
    // ambiguous (`builds` as third-person verb vs plural noun): that surface
    // is dropped while the clean surface stays.
    let table = derive_pos_class_table(
        &snapshot("build\tbuild\tverb\t90\nbuilds\tbuild\tverb\t30\nbuilds\tbuild\tnoun\t20\n"),
        &PosPolicyConfig::default(),
    );
    let surfaces = table
        .iter()
        .map(xtask::PosClassEntry::surface)
        .collect::<Vec<_>>();

    assert_eq!(surfaces, ["build"]);
}

#[test]
fn surfaces_claimed_by_conflicting_lemmas_are_dropped() {
    // One surface reachable from a noun lemma and a verb lemma: conservative
    // policy drops the surface rather than guessing.
    let table = derive_pos_class_table(
        &snapshot("left\tleave\tverb\t80\nleft\tleft\tnoun\t70\n"),
        &PosPolicyConfig::default(),
    );

    assert!(table.iter().all(|entry| entry.surface() != "left"));
}

#[test]
fn ranking_caps_each_class_by_content_frequency() {
    let config = PosPolicyConfig::default().with_top_lemmas_per_class(1);
    let table = derive_pos_class_table(
        &snapshot(
            "bug\tbug\tnoun\t90\nerror\terror\tnoun\t50\n\
             verify\tverify\tverb\t80\ndeploy\tdeploy\tverb\t40\n",
        ),
        &config,
    );
    let surfaces = table
        .iter()
        .map(xtask::PosClassEntry::surface)
        .collect::<Vec<_>>();

    // Only the highest-frequency lemma of each class survives the cap.
    assert_eq!(surfaces, ["bug", "verify"]);
}

#[test]
fn derivation_is_reproducible_and_sorted() {
    let rows = "verify\tverify\tverb\t80\nbug\tbug\tnoun\t90\nbugs\tbug\tnoun\t40\n";
    let first = derive_pos_class_table(&snapshot(rows), &PosPolicyConfig::default());
    let second = derive_pos_class_table(&snapshot(rows), &PosPolicyConfig::default());

    assert_eq!(first, second);

    let surfaces = first
        .iter()
        .map(xtask::PosClassEntry::surface)
        .collect::<Vec<_>>();
    let mut sorted = surfaces.clone();

    sorted.sort_unstable();
    assert_eq!(surfaces, sorted);
}
