//! Layout tests for the `VOICE_V12` sidecar POS class-table asset (T-121).
//!
//! The sidecar is a deterministic little-endian binary: an eight-byte magic,
//! a spec version, the pinned corpus + tagged-counts SHA-256 provenance
//! hashes, and a strictly surface-sorted entry list. Parsing validates every
//! structural invariant so a corrupted embed fails loudly.

use dootdoot_core::{
    POS_TABLE_HASH_BYTES, POS_TABLE_MAGIC, POS_TABLE_SPEC_VERSION, PosClass, PosTable,
    embedded_pos_table,
};

fn entry(class_byte: u8, surface: &str) -> Vec<u8> {
    let mut bytes = vec![class_byte];
    let length = u16::try_from(surface.len()).expect("test surface fits u16");

    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(surface.as_bytes());

    bytes
}

fn table_bytes(entries: &[(u8, &str)]) -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&POS_TABLE_MAGIC);
    bytes.extend_from_slice(&POS_TABLE_SPEC_VERSION.to_le_bytes());
    bytes.extend_from_slice(&[0xAA; POS_TABLE_HASH_BYTES]);
    bytes.extend_from_slice(&[0xBB; POS_TABLE_HASH_BYTES]);
    bytes.extend_from_slice(
        &u32::try_from(entries.len())
            .expect("test entry count fits u32")
            .to_le_bytes(),
    );

    for (class_byte, surface) in entries {
        bytes.extend_from_slice(&entry(*class_byte, surface));
    }

    bytes
}

#[test]
fn parses_a_valid_table_and_looks_up_classes() {
    let table =
        PosTable::parse(&table_bytes(&[(0, "bug"), (1, "verify")])).expect("valid table parses");

    assert_eq!(table.entry_count(), 2);
    assert_eq!(table.class_of("bug"), PosClass::Noun);
    assert_eq!(table.class_of("verify"), PosClass::Verb);
    assert_eq!(table.class_of("the"), PosClass::Other);
}

#[test]
fn lookup_is_ascii_case_insensitive() {
    let table = PosTable::parse(&table_bytes(&[(0, "bug")])).expect("valid table parses");

    assert_eq!(table.class_of("Bug"), PosClass::Noun);
    assert_eq!(table.class_of("BUG"), PosClass::Noun);
}

#[test]
fn parse_carries_the_provenance_hashes() {
    let table = PosTable::parse(&table_bytes(&[(0, "bug")])).expect("valid table parses");

    assert_eq!(table.corpus_hash(), [0xAA; POS_TABLE_HASH_BYTES]);
    assert_eq!(table.tagged_counts_hash(), [0xBB; POS_TABLE_HASH_BYTES]);
}

#[test]
fn embedded_table_parses_and_classifies_coding_vocabulary() {
    let table = embedded_pos_table().expect("the committed sidecar asset parses");

    assert!(table.entry_count() > 1_000);

    // Canonical baked outcomes: clean content words class, the conservative
    // ambiguity fallback and closed-class exclusion leave the rest `Other`.
    assert_eq!(table.class_of("bug"), PosClass::Noun);
    assert_eq!(table.class_of("bugs"), PosClass::Noun);
    assert_eq!(table.class_of("verify"), PosClass::Verb);
    assert_eq!(table.class_of("build"), PosClass::Other);
    assert_eq!(table.class_of("the"), PosClass::Other);
    assert_eq!(table.class_of("can"), PosClass::Other);
}

#[test]
fn rejects_a_bad_magic() {
    let mut bytes = table_bytes(&[(0, "bug")]);

    bytes[0] = b'X';
    assert!(PosTable::parse(&bytes).is_err());
}

#[test]
fn rejects_an_unknown_spec_version() {
    let mut bytes = table_bytes(&[(0, "bug")]);

    bytes[8] = 99;
    assert!(PosTable::parse(&bytes).is_err());
}

#[test]
fn rejects_an_unknown_class_byte() {
    assert!(PosTable::parse(&table_bytes(&[(7, "bug")])).is_err());
}

#[test]
fn rejects_unsorted_or_duplicate_surfaces() {
    assert!(PosTable::parse(&table_bytes(&[(1, "verify"), (0, "bug")])).is_err());
    assert!(PosTable::parse(&table_bytes(&[(0, "bug"), (1, "bug")])).is_err());
}

#[test]
fn rejects_truncated_and_oversized_payloads() {
    let bytes = table_bytes(&[(0, "bug")]);

    assert!(PosTable::parse(&bytes[..bytes.len() - 1]).is_err());

    let mut oversized = bytes;

    oversized.push(0);
    assert!(PosTable::parse(&oversized).is_err());
}
