//! Pure deterministic engine for dootdoot.

mod engine;
mod format;
mod mapping;
mod mathx;
mod sequence;
mod synth;
mod tokenizer;
mod wav;

pub use engine::{
    EngineError, ExplainPunctuationRow, ExplainRow, ExplainTokenRow, explain_rows_for_text,
    render_text_canonical_buffer, sequence_events_for_text,
};
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
pub use sequence::{
    EMPTY_CHIRP_CONTOUR, EMPTY_CHIRP_PITCH_CENTER, EMPTY_CHIRP_START_PITCH_CENTER,
    EMPTY_CHIRP_VOWEL_POSITION, EMPTY_CHIRP_WARBLE_DEPTH, ProsodicPunctuation, SequenceEvent,
    SequencedUtterance, SyllableEvent, estimate_utterance_sample_count, render_empty_chirp,
    sequence_utterance,
};
pub use synth::{
    BASE_SYLLABLE_SAMPLES, BASE_SYLLABLE_SECONDS, ENVELOPE_ATTACK_SECONDS, ENVELOPE_DECAY_SECONDS,
    ENVELOPE_RELEASE_SECONDS, ENVELOPE_SUSTAIN_LEVEL, FORMANT_AH_HZ, FORMANT_COUNT, FORMANT_EE_HZ,
    FORMANT_GAINS, FORMANT_OO_HZ, FORMANT_Q, FormantFilterBank, INTERNAL_PITCH_ARCH_CENTS,
    INTERNAL_PITCH_SWEEP_CENTS, LEADING_SILENCE_SAMPLES, LEADING_SILENCE_SECONDS,
    LONG_PUNCTUATION_PAUSE_SAMPLES, LONG_PUNCTUATION_PAUSE_SECONDS,
    MEDIUM_PUNCTUATION_PAUSE_SAMPLES, MEDIUM_PUNCTUATION_PAUSE_SECONDS, PITCH_REGISTER_BIAS_HZ,
    PITCH_SEMITONE_SPAN, PORTAMENTO_SECONDS, PUNCTUATION_GLIDE_SEMITONES, RING_MOD_FREQUENCY_HZ,
    RING_MOD_MIX, SOURCE_MAX_HARMONICS, SOURCE_PULSE_MIX, SOURCE_PULSE_WIDTH, SOURCE_SAW_MIX,
    SYNTH_SAMPLE_RATE_HZ, Synth, TRAILING_SILENCE_SAMPLES, TRAILING_SILENCE_SECONDS,
    VOWEL_LOCUS_COUNT, VOWEL_TRAJECTORY_BLOOM, VOWEL_TRAJECTORY_SWEEP, WARBLE_DEPTH_CENTS,
    WARBLE_RATE_HZ, WORD_PAUSE_SAMPLES, WORD_PAUSE_SECONDS, amplitude_envelope,
    apply_amplitude_envelope, apply_internal_pitch_swoop_hz, apply_warble_hz, formant_frequencies,
    internal_pitch_offset_cents, pitch_center_hz, portamento_pitch_hz, portamento_progress,
    render_syllable, ring_modulate, source_harmonic_count, source_oscillator_sample,
    vowel_trajectory_position, warble_depth_cents, warble_offset_cents,
};
pub use tokenizer::{
    TokenizedInput, TokenizedToken, Tokenizer, TokenizerError, embedded_tokenizer,
};
pub use wav::{
    PCM_I16_SCALE, WavError, WavWriter, quantize_sample, render_canonical_buffer, wav_bytes,
    write_wav,
};
