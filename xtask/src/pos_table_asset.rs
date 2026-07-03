//! Serialization of the baked `VOICE_V12` sidecar class-table asset.
//!
//! Mirrors `dootdoot-core`'s [`dootdoot_core::PosTable`] layout exactly (the
//! constants are imported from the core crate so writer and reader cannot
//! drift): magic, spec version, the pinned corpus and tagged-counts SHA-256
//! provenance hashes from the `[pos]` manifest section, and the strictly
//! surface-sorted entry list.

use dootdoot_core::{POS_TABLE_HASH_BYTES, POS_TABLE_MAGIC, POS_TABLE_SPEC_VERSION};

use crate::{PosClassEntry, PosSourceManifest, PosTableClass, Result, SourceManifestError};

/// Serializes derived class-table entries into the sidecar asset payload.
///
/// # Errors
///
/// Returns an error when the manifest hashes are not valid hex, the entries
/// are not strictly sorted by surface, or an entry does not fit the layout's
/// field widths.
pub fn serialize_pos_table(
    entries: &[PosClassEntry],
    manifest: &PosSourceManifest,
) -> Result<Vec<u8>> {
    let corpus_hash = decode_hash("corpus_sha256", manifest.corpus_sha256())?;
    let tagged_counts_hash = decode_hash("tagged_counts_sha256", manifest.tagged_counts_sha256())?;
    let entry_count = u32::try_from(entries.len()).map_err(|error| {
        SourceManifestError::new(format!("pos table entry count does not fit u32: {error}"))
    })?;
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&POS_TABLE_MAGIC);
    bytes.extend_from_slice(&POS_TABLE_SPEC_VERSION.to_le_bytes());
    bytes.extend_from_slice(&corpus_hash);
    bytes.extend_from_slice(&tagged_counts_hash);
    bytes.extend_from_slice(&entry_count.to_le_bytes());

    let mut previous_surface: Option<&str> = None;

    for entry in entries {
        if previous_surface.is_some_and(|previous| previous >= entry.surface()) {
            return Err(SourceManifestError::new(
                "pos table entries should be strictly sorted by surface",
            ));
        }

        previous_surface = Some(entry.surface());

        let class_byte = match entry.pos_class() {
            PosTableClass::Noun => 0_u8,
            PosTableClass::Verb => 1_u8,
        };
        let length = u16::try_from(entry.surface().len()).map_err(|error| {
            SourceManifestError::new(format!(
                "pos table surface {} does not fit u16 length: {error}",
                entry.surface(),
            ))
        })?;

        bytes.push(class_byte);
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(entry.surface().as_bytes());
    }

    Ok(bytes)
}

fn decode_hash(field: &str, hex_hash: &str) -> Result<[u8; POS_TABLE_HASH_BYTES]> {
    let decoded = hex::decode(hex_hash)
        .map_err(|error| SourceManifestError::new(format!("{field} is not valid hex: {error}")))?;

    <[u8; POS_TABLE_HASH_BYTES]>::try_from(decoded).map_err(|_decoded| {
        SourceManifestError::new(format!("{field} should be {POS_TABLE_HASH_BYTES} bytes"))
    })
}
