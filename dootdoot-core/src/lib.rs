//! Pure deterministic engine for dootdoot.

mod format;
mod mapping;
mod mathx;
mod synth;
mod tokenizer;
mod wav;

pub use format::{FORMAT_V1, Format};
pub use mapping::Mapping;
pub use mathx::{
    EXP_POLYNOMIAL_DEGREE, EXP_TABLE_BITS, EXP_TABLE_LEN, MATHX_VERSION, Mathx,
    SIN_COS_POLYNOMIAL_DEGREE, SIN_COS_TABLE_BITS, SIN_COS_TABLE_LEN, TANH_EXP_CLAMP, cos, sin,
};
pub use synth::Synth;
pub use tokenizer::Tokenizer;
pub use wav::WavWriter;
