//! Pinned `[pos]` source-manifest section for the `VOICE_V12` class table.
//!
//! FR-114 pins the POS pipeline the same way the semantic model is pinned:
//! the public ranking-corpus shard by repo/revision/file/SHA-256, the tagger
//! by name/version, and the committed tagged-counts snapshot
//! (`assets/pos/tagged_counts.tsv`) by SHA-256. `xtask` validates the
//! snapshot hash before deriving the baked class table.

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{Result, SourceManifestError};

/// Holds the pinned POS source contract from `[pos]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosSourceManifest {
    corpus_hf_repo: String,
    corpus_revision: String,
    corpus_file: String,
    corpus_sha256: String,
    tagger: String,
    tagger_version: String,
    tagged_counts_sha256: String,
    derivation: String,
}

impl PosSourceManifest {
    /// Parses the `[pos]` section out of the source manifest TOML.
    ///
    /// # Errors
    ///
    /// Returns an error when the TOML is malformed or the `[pos]` section is
    /// missing or incomplete.
    pub fn parse(input: &str) -> Result<Self> {
        let manifest: ManifestToml = toml::from_str(input).map_err(|error| {
            SourceManifestError::new(format!("invalid source manifest: {error}"))
        })?;
        let pos = manifest.pos.ok_or_else(|| {
            SourceManifestError::new("source manifest is missing its [pos] section")
        })?;

        Ok(Self {
            corpus_hf_repo: pos.corpus_hf_repo,
            corpus_revision: pos.corpus_revision,
            corpus_file: pos.corpus_file,
            corpus_sha256: pos.corpus_sha256,
            tagger: pos.tagger,
            tagger_version: pos.tagger_version,
            tagged_counts_sha256: pos.tagged_counts_sha256,
            derivation: pos.derivation,
        })
    }

    /// Validates the committed tagged-counts snapshot against its pin.
    ///
    /// # Errors
    ///
    /// Returns an error when the snapshot bytes do not hash to the pinned
    /// SHA-256.
    pub fn validate_tagged_counts(&self, snapshot: &[u8]) -> Result<()> {
        let actual = hex::encode(Sha256::digest(snapshot));

        if actual == self.tagged_counts_sha256 {
            Ok(())
        } else {
            Err(SourceManifestError::new(format!(
                "tagged_counts.tsv sha256 mismatch: expected {}, got {actual} \
                 (regenerate via `{}` and update the manifest)",
                self.tagged_counts_sha256, self.derivation,
            )))
        }
    }

    /// Returns the ranking-corpus Hugging Face dataset repository.
    #[must_use]
    pub fn corpus_hf_repo(&self) -> &str {
        &self.corpus_hf_repo
    }

    /// Returns the immutable ranking-corpus revision.
    #[must_use]
    pub fn corpus_revision(&self) -> &str {
        &self.corpus_revision
    }

    /// Returns the pinned corpus shard path inside the dataset repository.
    #[must_use]
    pub fn corpus_file(&self) -> &str {
        &self.corpus_file
    }

    /// Returns the expected SHA-256 of the corpus shard.
    #[must_use]
    pub fn corpus_sha256(&self) -> &str {
        &self.corpus_sha256
    }

    /// Returns the pinned POS tagger identifier.
    #[must_use]
    pub fn tagger(&self) -> &str {
        &self.tagger
    }

    /// Returns the pinned POS tagger version.
    #[must_use]
    pub fn tagger_version(&self) -> &str {
        &self.tagger_version
    }

    /// Returns the expected SHA-256 of the committed tagged-counts snapshot.
    #[must_use]
    pub fn tagged_counts_sha256(&self) -> &str {
        &self.tagged_counts_sha256
    }

    /// Returns the documented derivation command.
    #[must_use]
    pub fn derivation(&self) -> &str {
        &self.derivation
    }
}

#[derive(Debug, Deserialize)]
struct ManifestToml {
    pos: Option<PosToml>,
}

#[derive(Debug, Deserialize)]
struct PosToml {
    corpus_hf_repo: String,
    corpus_revision: String,
    corpus_file: String,
    corpus_sha256: String,
    tagger: String,
    tagger_version: String,
    tagged_counts_sha256: String,
    derivation: String,
}
