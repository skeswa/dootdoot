//! Semantic mapping from tokenizer IDs to perceptual axes.

use thiserror::Error;

use crate::{FORMAT_AXIS_COUNT, FORMAT_TOKEN_RECORD_BYTES, FormatArtifact, embedded_format_v1};

/// Maps tokenizer IDs to baked semantic vectors and pooling weights.
#[derive(Debug, Clone)]
pub struct Mapping<'a> {
    format: FormatArtifact<'a>,
}

/// Gives one dequantized token vector and its pooling weight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenVector {
    axes: [f64; FORMAT_AXIS_COUNT],
    weight: f64,
}

/// Reports why a token ID could not be mapped.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct MappingError {
    message: String,
}

impl<'a> Mapping<'a> {
    /// Builds a mapping from a parsed format artifact.
    pub fn from_format(format: FormatArtifact<'a>) -> Self {
        Self { format }
    }

    /// Looks up and dequantizes one tokenizer ID.
    ///
    /// # Errors
    ///
    /// Returns an error when `token_id` is outside the baked record table or
    /// when record offsets overflow platform pointer sizes.
    pub fn lookup(&self, token_id: u32) -> Result<TokenVector, MappingError> {
        let token_index = usize::try_from(token_id)
            .map_err(|error| MappingError::new(format!("token ID does not fit usize: {error}")))?;

        if token_index >= self.format.token_count() {
            return Err(MappingError::new(format!(
                "token ID {token_id} is outside mapping table with {} records",
                self.format.token_count(),
            )));
        }

        let start = token_index
            .checked_mul(FORMAT_TOKEN_RECORD_BYTES)
            .ok_or_else(|| MappingError::new("mapping record offset overflow"))?;
        let end = start
            .checked_add(FORMAT_TOKEN_RECORD_BYTES)
            .ok_or_else(|| MappingError::new("mapping record end overflow"))?;
        let record = self
            .format
            .record_bytes()
            .get(start..end)
            .ok_or_else(|| MappingError::new("mapping record is missing from artifact"))?;
        let axes = [
            dequantize(read_i16(record, 0)?, self.format.axis_scales()[0]),
            dequantize(read_i16(record, 2)?, self.format.axis_scales()[1]),
            dequantize(read_i16(record, 4)?, self.format.axis_scales()[2]),
            dequantize(read_i16(record, 6)?, self.format.axis_scales()[3]),
        ];
        let weight = dequantize(read_i16(record, 8)?, self.format.weight_scale());

        Ok(TokenVector { axes, weight })
    }

    /// Returns the number of token records in the mapping table.
    pub fn token_count(&self) -> usize {
        self.format.token_count()
    }
}

impl TokenVector {
    /// Builds a token vector from dequantized axis and weight values.
    pub fn new(axes: [f64; FORMAT_AXIS_COUNT], weight: f64) -> Self {
        Self { axes, weight }
    }

    /// Returns dequantized semantic axes in fixed `FORMAT_V1` order.
    pub fn axes(&self) -> [f64; FORMAT_AXIS_COUNT] {
        self.axes
    }

    /// Returns the dequantized pooling weight.
    pub fn weight(&self) -> f64 {
        self.weight
    }
}

impl MappingError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Loads the embedded `FORMAT_V1` mapping table.
///
/// # Errors
///
/// Returns an error if the committed format artifact cannot be parsed.
pub fn embedded_mapping() -> Result<Mapping<'static>, MappingError> {
    embedded_format_v1()
        .map(Mapping::from_format)
        .map_err(|error| {
            MappingError::new(format!("failed to load embedded mapping format: {error}"))
        })
}

fn dequantize(code: i16, scale: f32) -> f64 {
    f64::from(code) * f64::from(scale)
}

fn read_i16(bytes: &[u8], offset: usize) -> Result<i16, MappingError> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| MappingError::new("mapping record field offset overflow"))?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| MappingError::new("mapping record field is missing"))?;
    let mut raw = [0_u8; 2];
    raw.copy_from_slice(field);

    Ok(i16::from_le_bytes(raw))
}
