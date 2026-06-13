//! Pure deterministic engine for dootdoot.

mod format;
mod mapping;
mod mathx;
mod synth;
mod tokenizer;
mod wav;

pub use format::{
    FORMAT_AXIS_COUNT, FORMAT_HASH_BYTES, FORMAT_HEADER_BYTES, FORMAT_MAGIC, FORMAT_SCALE_COUNT,
    FORMAT_SQUASH_STATS_PER_AXIS, FORMAT_TOKEN_RECORD_BYTES, FORMAT_V1, FORMAT_VERSION_NUMBER,
    Format, FormatArtifact, FormatError, FormatSquashFunction, SquashAxisStats, embedded_format_v1,
};
pub use mapping::{
    Mapping, MappingError, PooledVector, TokenVector, embedded_mapping, pool_sequence,
};
pub use mathx::{
    EXP_POLYNOMIAL_DEGREE, EXP_TABLE_BITS, EXP_TABLE_LEN, MATHX_VERSION, Mathx,
    SIN_COS_POLYNOMIAL_DEGREE, SIN_COS_TABLE_BITS, SIN_COS_TABLE_LEN, TANH_EXP_CLAMP, cos, exp,
    sin, tanh,
};
pub use synth::Synth;
pub use tokenizer::{
    TokenizedInput, TokenizedToken, Tokenizer, TokenizerError, embedded_tokenizer,
};
pub use wav::WavWriter;
