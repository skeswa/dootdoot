//! Baked `VOICE_V12` sidecar POS class-table asset parsing.
//!
//! The class table is the runtime half of the FR-114 pipeline: `xtask`
//! derives it from the pinned tagged-counts snapshot and serializes this
//! deterministic little-endian layout; the shipped binary embeds and parses
//! it — a pure lookup, no tagger, no tensor runtime. The sidecar keeps the
//! semantic `.doot` asset untouched at spec v1.
//!
//! Layout: [`POS_TABLE_MAGIC`] (8 bytes) · [`POS_TABLE_SPEC_VERSION`]
//! (`u16` LE) · corpus SHA-256 (32 bytes) · tagged-counts SHA-256 (32 bytes) ·
//! entry count (`u32` LE) · entries. Each entry is a class byte (`0` noun,
//! `1` verb), a `u16` LE surface length, and the UTF-8 surface bytes; entries
//! are strictly sorted by surface so lookups can binary-search.

use thiserror::Error;

use crate::PosClass;

/// Gives the sidecar asset's eight-byte magic.
pub const POS_TABLE_MAGIC: [u8; 8] = *b"DOOTPOS1";

/// Gives the sidecar asset spec version this build reads and writes.
pub const POS_TABLE_SPEC_VERSION: u16 = 1;

/// Gives the width of each provenance hash in the header.
pub const POS_TABLE_HASH_BYTES: usize = 32;

/// Gives the committed sidecar asset file name.
pub const POS_TABLE_FILE_V1: &str = "dootdoot_pos_v1.doot";

/// Holds the committed sidecar asset bytes shipped inside the binary.
const EMBEDDED_POS_TABLE: &[u8] = include_bytes!("../../assets/dootdoot_pos_v1.doot");

/// Gives the parsed baked class table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosTable {
    corpus_hash: [u8; POS_TABLE_HASH_BYTES],
    tagged_counts_hash: [u8; POS_TABLE_HASH_BYTES],
    entries: Vec<(String, PosClass)>,
}

/// Reports why a sidecar class-table payload could not be parsed.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct PosTableError {
    message: String,
}

impl PosTable {
    /// Parses a sidecar class-table payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the magic, spec version, entry encoding, sort
    /// order, or total length is invalid.
    pub fn parse(bytes: &[u8]) -> Result<Self, PosTableError> {
        let mut reader = Reader::new(bytes);
        let magic = reader.take(POS_TABLE_MAGIC.len(), "magic")?;

        if magic != POS_TABLE_MAGIC {
            return Err(PosTableError::new("pos table magic mismatch"));
        }

        let version = u16::from_le_bytes(fixed(reader.take(2, "spec version")?)?);

        if version != POS_TABLE_SPEC_VERSION {
            return Err(PosTableError::new(format!(
                "unsupported pos table spec version {version}, expected {POS_TABLE_SPEC_VERSION}",
            )));
        }

        let corpus_hash = fixed(reader.take(POS_TABLE_HASH_BYTES, "corpus hash")?)?;
        let tagged_counts_hash = fixed(reader.take(POS_TABLE_HASH_BYTES, "tagged-counts hash")?)?;
        let entry_count = u32::from_le_bytes(fixed(reader.take(4, "entry count")?)?);
        let mut entries: Vec<(String, PosClass)> =
            Vec::with_capacity(usize::try_from(entry_count).unwrap_or_default());

        for _ in 0..entry_count {
            let class = match reader.take(1, "entry class")? {
                [0] => PosClass::Noun,
                [1] => PosClass::Verb,
                other => {
                    return Err(PosTableError::new(format!(
                        "unknown pos table class byte {}",
                        other[0],
                    )));
                }
            };
            let length = u16::from_le_bytes(fixed(reader.take(2, "surface length")?)?);
            let surface = str::from_utf8(reader.take(usize::from(length), "surface bytes")?)
                .map_err(|error| {
                    PosTableError::new(format!("pos table surface is not utf-8: {error}"))
                })?;

            if let Some((last_surface, _class)) = entries.last()
                && last_surface.as_str() >= surface
            {
                return Err(PosTableError::new(
                    "pos table surfaces should be strictly sorted",
                ));
            }

            entries.push((surface.to_owned(), class));
        }

        if !reader.is_empty() {
            return Err(PosTableError::new(
                "pos table has trailing bytes after its entries",
            ));
        }

        Ok(Self {
            corpus_hash,
            tagged_counts_hash,
            entries,
        })
    }

    /// Looks one word up (ASCII-case-insensitively); absent words are
    /// [`PosClass::Other`].
    pub fn class_of(&self, word: &str) -> PosClass {
        let word = word.to_ascii_lowercase();

        self.entries
            .binary_search_by(|(surface, _class)| surface.as_str().cmp(word.as_str()))
            .map_or(PosClass::Other, |index| self.entries[index].1)
    }

    /// Returns the number of baked entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the pinned ranking-corpus SHA-256 carried in the header.
    pub fn corpus_hash(&self) -> [u8; POS_TABLE_HASH_BYTES] {
        self.corpus_hash
    }

    /// Returns the pinned tagged-counts snapshot SHA-256 carried in the
    /// header.
    pub fn tagged_counts_hash(&self) -> [u8; POS_TABLE_HASH_BYTES] {
        self.tagged_counts_hash
    }
}

impl PosTableError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parses the embedded committed sidecar class table.
///
/// # Errors
///
/// Returns an error when the committed asset does not parse — a build
/// integrity failure, not an input condition.
pub fn embedded_pos_table() -> Result<PosTable, PosTableError> {
    PosTable::parse(EMBEDDED_POS_TABLE)
}

#[derive(Debug)]
struct Reader<'bytes> {
    bytes: &'bytes [u8],
}

impl<'bytes> Reader<'bytes> {
    fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes }
    }

    fn take(&mut self, count: usize, what: &str) -> Result<&'bytes [u8], PosTableError> {
        let (taken, rest) = self
            .bytes
            .split_at_checked(count)
            .ok_or_else(|| PosTableError::new(format!("pos table is truncated at its {what}")))?;

        self.bytes = rest;

        Ok(taken)
    }

    fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

fn fixed<const N: usize>(bytes: &[u8]) -> Result<[u8; N], PosTableError> {
    <[u8; N]>::try_from(bytes)
        .map_err(|error| PosTableError::new(format!("pos table field width mismatch: {error}")))
}
