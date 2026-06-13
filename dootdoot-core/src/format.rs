//! Format contract identifiers for dootdoot outputs.

use thiserror::Error;

/// Identifies the first frozen sample-affecting output contract.
pub const FORMAT_V1: &str = "FORMAT_V1";

/// Starts every `format_v1.bin` artifact.
pub const FORMAT_MAGIC: [u8; 8] = *b"DOOTV1\0\0";

/// Gives the numeric version stored in the binary artifact header.
pub const FORMAT_VERSION_NUMBER: u32 = 1;

/// Gives the number of PCA/perceptual axes stored per token.
pub const FORMAT_AXIS_COUNT: usize = 4;

/// Gives the number of SHA-256 bytes stored for each provenance hash.
pub const FORMAT_HASH_BYTES: usize = 32;

/// Gives the number of dequantization scales in the header.
pub const FORMAT_SCALE_COUNT: usize = FORMAT_AXIS_COUNT + 1;

/// Gives the number of squash statistics stored per axis.
pub const FORMAT_SQUASH_STATS_PER_AXIS: usize = 2;

/// Gives the byte length of the `format_v1.bin` header.
pub const FORMAT_HEADER_BYTES: usize = FORMAT_MAGIC.len()
    + 4
    + 4
    + 4
    + 4
    + (FORMAT_SCALE_COUNT * 4)
    + 4
    + (FORMAT_AXIS_COUNT * FORMAT_SQUASH_STATS_PER_AXIS * 8)
    + (FORMAT_HASH_BYTES * 3);

/// Gives the byte length of one per-token record.
pub const FORMAT_TOKEN_RECORD_BYTES: usize = (FORMAT_AXIS_COUNT * 2) + 2;

const EMBEDDED_FORMAT_V1: &[u8] = include_bytes!("../../assets/format_v1.bin");
const HEADER_BYTE_LEN_OFFSET: usize = 8;
const VERSION_OFFSET: usize = 12;
const TOKEN_COUNT_OFFSET: usize = 16;
const AXIS_COUNT_OFFSET: usize = 20;
const AXIS_SCALES_OFFSET: usize = 24;
const WEIGHT_SCALE_OFFSET: usize = AXIS_SCALES_OFFSET + (FORMAT_AXIS_COUNT * 4);
const SQUASH_FUNCTION_OFFSET: usize = WEIGHT_SCALE_OFFSET + 4;
const SQUASH_STATS_OFFSET: usize = SQUASH_FUNCTION_OFFSET + 4;
const MODEL_HASH_OFFSET: usize =
    SQUASH_STATS_OFFSET + (FORMAT_AXIS_COUNT * FORMAT_SQUASH_STATS_PER_AXIS * 8);
const TOKENIZER_HASH_OFFSET: usize = MODEL_HASH_OFFSET + FORMAT_HASH_BYTES;
const PCA_HASH_OFFSET: usize = TOKENIZER_HASH_OFFSET + FORMAT_HASH_BYTES;

/// Marks the format contract module in the public facade.
#[derive(Debug)]
pub struct Format;

/// Gives a parsed `format_v1.bin` artifact.
#[derive(Debug, Clone)]
pub struct FormatArtifact<'a> {
    header: FormatHeader,
    record_bytes: &'a [u8],
}

/// Describes the squash function frozen into the format artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatSquashFunction {
    /// Applies tanh to z-scored projected axis values.
    TanhZScore,
}

/// Gives the per-axis squash statistics frozen into the format artifact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquashAxisStats {
    mean: f64,
    standard_deviation: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct FormatHeader {
    format_id: &'static str,
    header_byte_len: usize,
    token_count: usize,
    axis_scales: [f32; FORMAT_AXIS_COUNT],
    weight_scale: f32,
    squash_function: FormatSquashFunction,
    squash_stats: [SquashAxisStats; FORMAT_AXIS_COUNT],
    model_hash: [u8; FORMAT_HASH_BYTES],
    tokenizer_hash: [u8; FORMAT_HASH_BYTES],
    pca_hash: [u8; FORMAT_HASH_BYTES],
}

/// Reports why a format artifact could not be parsed.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct FormatError {
    message: String,
}

impl FormatArtifact<'_> {
    /// Parses a `format_v1.bin` artifact.
    ///
    /// # Errors
    ///
    /// Returns an error when the artifact is truncated, has an unexpected
    /// magic/version, declares a layout inconsistent with `FORMAT_V1`, or
    /// contains invalid numeric fields.
    pub fn parse(bytes: &[u8]) -> Result<FormatArtifact<'_>, FormatError> {
        let header = parse_header(bytes)?;
        let expected_len = FORMAT_HEADER_BYTES
            .checked_add(
                header
                    .token_count
                    .checked_mul(FORMAT_TOKEN_RECORD_BYTES)
                    .ok_or_else(|| FormatError::new("format artifact record length overflow"))?,
            )
            .ok_or_else(|| FormatError::new("format artifact length overflow"))?;

        if bytes.len() != expected_len {
            return Err(FormatError::new(format!(
                "format artifact length mismatch: expected {expected_len} bytes, got {}",
                bytes.len(),
            )));
        }

        Ok(FormatArtifact {
            header,
            record_bytes: &bytes[FORMAT_HEADER_BYTES..],
        })
    }

    /// Returns the string identifier for this format contract.
    pub fn format_id(&self) -> &'static str {
        self.header.format_id
    }

    /// Returns the header byte length.
    pub fn header_byte_len(&self) -> usize {
        self.header.header_byte_len
    }

    /// Returns the number of token records in the artifact.
    pub fn token_count(&self) -> usize {
        self.header.token_count
    }

    /// Returns the per-axis dequantization scales for projected values.
    pub fn axis_scales(&self) -> &[f32; FORMAT_AXIS_COUNT] {
        &self.header.axis_scales
    }

    /// Returns the dequantization scale for token pooling weights.
    pub fn weight_scale(&self) -> f32 {
        self.header.weight_scale
    }

    /// Returns the squash function frozen into the artifact.
    pub fn squash_function(&self) -> FormatSquashFunction {
        self.header.squash_function
    }

    /// Returns the per-axis squash statistics frozen into the artifact.
    pub fn squash_stats(&self) -> &[SquashAxisStats; FORMAT_AXIS_COUNT] {
        &self.header.squash_stats
    }

    /// Returns the SHA-256 hash of the source model.
    pub fn model_hash(&self) -> [u8; FORMAT_HASH_BYTES] {
        self.header.model_hash
    }

    /// Returns the SHA-256 hash of the source tokenizer.
    pub fn tokenizer_hash(&self) -> [u8; FORMAT_HASH_BYTES] {
        self.header.tokenizer_hash
    }

    /// Returns the SHA-256 hash of the PCA matrix used to project the model.
    pub fn pca_hash(&self) -> [u8; FORMAT_HASH_BYTES] {
        self.header.pca_hash
    }

    /// Returns the raw per-token record bytes.
    pub fn record_bytes(&self) -> &'_ [u8] {
        self.record_bytes
    }
}

impl SquashAxisStats {
    /// Returns the projected-axis mean used by the squash function.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Returns the projected-axis standard deviation used by the squash
    /// function.
    pub fn standard_deviation(&self) -> f64 {
        self.standard_deviation
    }
}

impl FormatError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parses the embedded `FORMAT_V1` artifact.
///
/// # Errors
///
/// Returns an error if the committed artifact is malformed or inconsistent with
/// the format constants compiled into the crate.
pub fn embedded_format_v1() -> Result<FormatArtifact<'static>, FormatError> {
    FormatArtifact::parse(EMBEDDED_FORMAT_V1)
}

fn parse_header(bytes: &[u8]) -> Result<FormatHeader, FormatError> {
    let magic = read_bytes::<8>(bytes, 0)?;

    if magic != FORMAT_MAGIC {
        return Err(FormatError::new("format artifact magic mismatch"));
    }

    let header_byte_len = u32_to_usize(read_u32(bytes, HEADER_BYTE_LEN_OFFSET)?)?;

    if header_byte_len != FORMAT_HEADER_BYTES {
        return Err(FormatError::new(format!(
            "format artifact header length mismatch: expected {FORMAT_HEADER_BYTES}, got {header_byte_len}",
        )));
    }

    let version = read_u32(bytes, VERSION_OFFSET)?;

    if version != FORMAT_VERSION_NUMBER {
        return Err(FormatError::new(format!(
            "format artifact version mismatch: expected {FORMAT_VERSION_NUMBER}, got {version}",
        )));
    }

    let axis_count = u32_to_usize(read_u32(bytes, AXIS_COUNT_OFFSET)?)?;

    if axis_count != FORMAT_AXIS_COUNT {
        return Err(FormatError::new(format!(
            "format artifact axis count mismatch: expected {FORMAT_AXIS_COUNT}, got {axis_count}",
        )));
    }

    let axis_scales = [
        read_f32(bytes, AXIS_SCALES_OFFSET)?,
        read_f32(bytes, AXIS_SCALES_OFFSET + 4)?,
        read_f32(bytes, AXIS_SCALES_OFFSET + 8)?,
        read_f32(bytes, AXIS_SCALES_OFFSET + 12)?,
    ];
    let weight_scale = read_f32(bytes, WEIGHT_SCALE_OFFSET)?;

    validate_scale("axis 0", axis_scales[0])?;
    validate_scale("axis 1", axis_scales[1])?;
    validate_scale("axis 2", axis_scales[2])?;
    validate_scale("axis 3", axis_scales[3])?;
    validate_scale("weight", weight_scale)?;

    let squash_function = parse_squash_function(read_u32(bytes, SQUASH_FUNCTION_OFFSET)?)?;
    let squash_stats = [
        read_squash_axis_stats(bytes, SQUASH_STATS_OFFSET)?,
        read_squash_axis_stats(bytes, SQUASH_STATS_OFFSET + 16)?,
        read_squash_axis_stats(bytes, SQUASH_STATS_OFFSET + 32)?,
        read_squash_axis_stats(bytes, SQUASH_STATS_OFFSET + 48)?,
    ];

    Ok(FormatHeader {
        format_id: FORMAT_V1,
        header_byte_len,
        token_count: u32_to_usize(read_u32(bytes, TOKEN_COUNT_OFFSET)?)?,
        axis_scales,
        weight_scale,
        squash_function,
        squash_stats,
        model_hash: read_bytes::<FORMAT_HASH_BYTES>(bytes, MODEL_HASH_OFFSET)?,
        tokenizer_hash: read_bytes::<FORMAT_HASH_BYTES>(bytes, TOKENIZER_HASH_OFFSET)?,
        pca_hash: read_bytes::<FORMAT_HASH_BYTES>(bytes, PCA_HASH_OFFSET)?,
    })
}

fn read_squash_axis_stats(bytes: &[u8], offset: usize) -> Result<SquashAxisStats, FormatError> {
    let mean = read_f64(bytes, offset)?;
    let standard_deviation = read_f64(bytes, offset + 8)?;

    if !mean.is_finite() {
        return Err(FormatError::new(format!(
            "format artifact squash mean is not finite: {mean}",
        )));
    }

    if !standard_deviation.is_finite() || standard_deviation <= 0.0 {
        return Err(FormatError::new(format!(
            "format artifact squash standard deviation must be positive and finite: {standard_deviation}",
        )));
    }

    Ok(SquashAxisStats {
        mean,
        standard_deviation,
    })
}

fn parse_squash_function(value: u32) -> Result<FormatSquashFunction, FormatError> {
    match value {
        1 => Ok(FormatSquashFunction::TanhZScore),
        other => Err(FormatError::new(format!(
            "unknown format artifact squash function id: {other}",
        ))),
    }
}

fn validate_scale(name: &str, scale: f32) -> Result<(), FormatError> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(FormatError::new(format!(
            "format artifact {name} scale must be positive and finite: {scale}",
        )));
    }

    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, FormatError> {
    Ok(u32::from_le_bytes(read_bytes::<4>(bytes, offset)?))
}

fn read_f32(bytes: &[u8], offset: usize) -> Result<f32, FormatError> {
    Ok(f32::from_le_bytes(read_bytes::<4>(bytes, offset)?))
}

fn read_f64(bytes: &[u8], offset: usize) -> Result<f64, FormatError> {
    Ok(f64::from_le_bytes(read_bytes::<8>(bytes, offset)?))
}

fn read_bytes<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N], FormatError> {
    let end = offset
        .checked_add(N)
        .ok_or_else(|| FormatError::new("format artifact offset overflow"))?;
    let slice = bytes.get(offset..end).ok_or_else(|| {
        FormatError::new(format!(
            "format artifact is truncated: needed bytes {offset}..{end}, got {} bytes",
            bytes.len(),
        ))
    })?;
    let mut result = [0_u8; N];
    result.copy_from_slice(slice);

    Ok(result)
}

fn u32_to_usize(value: u32) -> Result<usize, FormatError> {
    usize::try_from(value)
        .map_err(|error| FormatError::new(format!("format integer does not fit usize: {error}")))
}
