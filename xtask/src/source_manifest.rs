//! Source manifest parsing and validation.

use std::str;

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{Result, SourceFiles, SourceManifestError};

/// Holds the pinned upstream model source contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceManifest {
    hf_repo: String,
    revision: String,
    model_sha256: String,
    tokenizer_sha256: String,
    config_sha256: String,
    hidden_dim: u16,
    normalize: bool,
    dtype: String,
    acquisition: String,
}

impl SourceManifest {
    /// Parses a source manifest from TOML.
    ///
    /// # Errors
    ///
    /// Returns an error when the TOML is malformed or required fields are
    /// missing.
    pub fn parse(input: &str) -> Result<Self> {
        let manifest: SourceManifestToml = toml::from_str(input).map_err(|error| {
            SourceManifestError::new(format!("invalid source manifest: {error}"))
        })?;

        Ok(Self {
            hf_repo: manifest.hf_repo,
            revision: manifest.revision,
            model_sha256: manifest.model_sha256,
            tokenizer_sha256: manifest.tokenizer_sha256,
            config_sha256: manifest.config_sha256,
            hidden_dim: manifest.hidden_dim,
            normalize: manifest.normalize,
            dtype: manifest.dtype,
            acquisition: manifest.acquisition,
        })
    }

    /// Validates source bytes against this manifest.
    ///
    /// # Errors
    ///
    /// Returns an error when the revision, file hashes, config fields, or
    /// safetensors dtype do not match the manifest.
    pub fn validate_sources(&self, sources: SourceFiles<'_>) -> Result<()> {
        if sources.revision != self.revision {
            return Err(SourceManifestError::new(format!(
                "revision mismatch: expected {}, got {}",
                self.revision, sources.revision,
            )));
        }

        validate_hash("model.safetensors", self.model_sha256(), sources.model)?;
        validate_hash("tokenizer.json", self.tokenizer_sha256(), sources.tokenizer)?;
        validate_hash("config.json", self.config_sha256(), sources.config)?;
        self.validate_config(sources.config)?;
        self.validate_safetensors_dtype(sources.model)
    }

    /// Returns the Hugging Face repository identifier.
    #[must_use]
    pub fn hf_repo(&self) -> &str {
        &self.hf_repo
    }

    /// Returns the immutable upstream repository revision.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Returns the expected SHA-256 for `model.safetensors`.
    #[must_use]
    pub fn model_sha256(&self) -> &str {
        &self.model_sha256
    }

    /// Returns the expected SHA-256 for `tokenizer.json`.
    #[must_use]
    pub fn tokenizer_sha256(&self) -> &str {
        &self.tokenizer_sha256
    }

    /// Returns the expected SHA-256 for `config.json`.
    #[must_use]
    pub fn config_sha256(&self) -> &str {
        &self.config_sha256
    }

    /// Returns the expected embedding dimensionality.
    #[must_use]
    pub fn hidden_dim(&self) -> u16 {
        self.hidden_dim
    }

    /// Returns whether upstream sequence embeddings are normalized.
    #[must_use]
    pub fn normalize(&self) -> bool {
        self.normalize
    }

    /// Returns the expected safetensors dtype.
    #[must_use]
    pub fn dtype(&self) -> &str {
        &self.dtype
    }

    /// Returns the chosen source acquisition strategy.
    #[must_use]
    pub fn acquisition(&self) -> &str {
        &self.acquisition
    }

    fn validate_config(&self, config: &[u8]) -> Result<()> {
        let config: ModelConfig = serde_json::from_slice(config)
            .map_err(|error| SourceManifestError::new(format!("invalid config.json: {error}")))?;

        if config.hidden_dim != self.hidden_dim {
            return Err(SourceManifestError::new(format!(
                "config hidden_dim mismatch: expected {}, got {}",
                self.hidden_dim, config.hidden_dim,
            )));
        }

        if config.normalize != self.normalize {
            return Err(SourceManifestError::new(format!(
                "config normalize mismatch: expected {}, got {}",
                self.normalize, config.normalize,
            )));
        }

        Ok(())
    }

    fn validate_safetensors_dtype(&self, model: &[u8]) -> Result<()> {
        let header = safetensors_header(model)?;
        let value: Value = serde_json::from_str(header).map_err(|error| {
            SourceManifestError::new(format!("invalid model.safetensors header: {error}"))
        })?;
        let object = value.as_object().ok_or_else(|| {
            SourceManifestError::new("model.safetensors header should be a JSON object")
        })?;
        let mut found_tensor = false;

        for (name, tensor) in object {
            if name == "__metadata__" {
                continue;
            }

            found_tensor = true;
            let dtype = tensor.get("dtype").and_then(Value::as_str).ok_or_else(|| {
                SourceManifestError::new(format!(
                    "model.safetensors tensor {name} is missing dtype",
                ))
            })?;

            if dtype != self.dtype {
                return Err(SourceManifestError::new(format!(
                    "model.safetensors dtype mismatch for {name}: expected {}, got {dtype}",
                    self.dtype,
                )));
            }
        }

        if !found_tensor {
            return Err(SourceManifestError::new(
                "model.safetensors should contain at least one tensor",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SourceManifestToml {
    hf_repo: String,
    revision: String,
    model_sha256: String,
    tokenizer_sha256: String,
    config_sha256: String,
    hidden_dim: u16,
    normalize: bool,
    dtype: String,
    acquisition: String,
}

#[derive(Debug, Deserialize)]
struct ModelConfig {
    hidden_dim: u16,
    normalize: bool,
}

fn validate_hash(file_name: &str, expected: &str, bytes: &[u8]) -> Result<()> {
    let actual = sha256_hex(bytes);

    if actual == expected {
        Ok(())
    } else {
        Err(SourceManifestError::new(format!(
            "{file_name} sha256 mismatch: expected {expected}, got {actual}",
        )))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);

    hex::encode(digest)
}

fn safetensors_header(model: &[u8]) -> Result<&str> {
    let length_bytes = model.get(..8).ok_or_else(|| {
        SourceManifestError::new("model.safetensors is missing its header length")
    })?;
    let length = u64::from_le_bytes(<[u8; 8]>::try_from(length_bytes).map_err(|error| {
        SourceManifestError::new(format!("invalid safetensors length: {error}"))
    })?);
    let length = usize::try_from(length).map_err(|error| {
        SourceManifestError::new(format!(
            "safetensors header length does not fit usize: {error}"
        ))
    })?;
    let header_start = 8_usize;
    let header_end = header_start
        .checked_add(length)
        .ok_or_else(|| SourceManifestError::new("safetensors header length overflowed"))?;
    let header = model
        .get(header_start..header_end)
        .ok_or_else(|| SourceManifestError::new("model.safetensors header is truncated"))?;

    str::from_utf8(header).map_err(|error| {
        SourceManifestError::new(format!("model.safetensors header is not utf-8: {error}"))
    })
}
