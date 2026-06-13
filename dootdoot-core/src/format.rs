//! Format contract identifiers for dootdoot outputs.

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

/// Marks the format contract module in the public facade.
#[derive(Debug)]
pub struct Format;
