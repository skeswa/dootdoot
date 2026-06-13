//! Source manifest validation errors.

use std::fmt;

/// Result type for xtask source validation.
pub type Result<T> = std::result::Result<T, SourceManifestError>;

/// Describes a source manifest validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceManifestError {
    message: String,
}

impl SourceManifestError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SourceManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for SourceManifestError {}
