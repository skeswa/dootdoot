//! Borrowed source files for manifest validation.

/// Borrows the source files that must match a source manifest.
#[derive(Debug, Clone, Copy)]
pub struct SourceFiles<'a> {
    pub(crate) revision: &'a str,
    pub(crate) model: &'a [u8],
    pub(crate) tokenizer: &'a [u8],
    pub(crate) config: &'a [u8],
}

impl<'a> SourceFiles<'a> {
    /// Creates source-file bytes for validation.
    #[must_use]
    pub fn new(revision: &'a str, model: &'a [u8], tokenizer: &'a [u8], config: &'a [u8]) -> Self {
        Self {
            revision,
            model,
            tokenizer,
            config,
        }
    }
}
