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
    KNOB_BOUNDS, KNOB_MODULATION_DEPTHS, KnobBounds, KnobSet, Mapping, MappingError, PooledVector,
    SquashedVector, TokenVector, assemble_knob_sequence, assemble_knobs, embedded_mapping,
    pool_sequence,
};
pub use mathx::{
    EXP_POLYNOMIAL_DEGREE, EXP_TABLE_BITS, EXP_TABLE_LEN, MATHX_VERSION, Mathx,
    SIN_COS_POLYNOMIAL_DEGREE, SIN_COS_TABLE_BITS, SIN_COS_TABLE_LEN, TANH_EXP_CLAMP, cos, exp,
    sin, tanh,
};
pub use synth::{
    BASE_SYLLABLE_SECONDS, ENVELOPE_ATTACK_SECONDS, ENVELOPE_DECAY_SECONDS,
    ENVELOPE_RELEASE_SECONDS, FORMANT_AH_HZ, FORMANT_COUNT, FORMANT_EE_HZ, FORMANT_GAINS,
    FORMANT_OO_HZ, FORMANT_Q, FormantFilterBank, LEADING_SILENCE_SECONDS,
    LONG_PUNCTUATION_PAUSE_SECONDS, MEDIUM_PUNCTUATION_PAUSE_SECONDS, PITCH_REGISTER_BIAS_HZ,
    PITCH_SEMITONE_SPAN, PORTAMENTO_SECONDS, RING_MOD_FREQUENCY_HZ, RING_MOD_MIX,
    SOURCE_MAX_HARMONICS, SOURCE_PULSE_MIX, SOURCE_PULSE_WIDTH, SOURCE_SAW_MIX,
    SYNTH_SAMPLE_RATE_HZ, Synth, TRAILING_SILENCE_SECONDS, VOWEL_LOCUS_COUNT, WARBLE_DEPTH_CENTS,
    WARBLE_RATE_HZ, WORD_PAUSE_SECONDS, formant_frequencies, source_harmonic_count,
    source_oscillator_sample,
};
pub use tokenizer::{
    TokenizedInput, TokenizedToken, Tokenizer, TokenizerError, embedded_tokenizer,
};
pub use wav::WavWriter;
