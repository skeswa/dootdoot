//! Pure deterministic engine for dootdoot.

mod affect;
mod archetype;
mod asset;
mod complexity;
mod engine;
mod mapping;
mod mathx;
mod performance;
mod phrase;
mod sequence;
mod synth;
mod tokenizer;
mod wav;

pub use affect::{AffectAnalysis, AffectTokenScore, UtteranceMood, analyze_affect_for_text};
pub use archetype::{
    ArchetypeSelection, GESTURE_ARCHETYPE_PALETTE, GestureArchetype, archetype_for_role,
    plan_gesture_archetypes,
};
pub use asset::{
    ACTIVE_VOICE, DOOT_ASSET_AXIS_COUNT, DOOT_ASSET_FILE_V1, DOOT_ASSET_HASH_BYTES,
    DOOT_ASSET_SCALE_COUNT, DOOT_ASSET_SPEC_VERSION, DOOT_ASSET_SQUASH_STATS_PER_AXIS,
    DOOT_ASSET_TOKEN_RECORD_BYTES, DootAsset, DootAssetError, DootAssetHashes, DootAssetParts,
    DootAssetScales, DootAssetSpec, DootAssetSquashAxisStats, DootAssetSquashFunction, VOICE_V1,
    VOICE_V2, VOICE_V3, VOICE_V4, VOICE_V5, VOICE_V6, VOICE_V7, VOICE_V8, VOICE_V9,
    embedded_doot_asset,
};
pub use complexity::{ComplexityAnalysis, analyze_complexity_for_text};
pub use engine::{
    EngineError, ExplainComplexityRow, ExplainHesitationRow, ExplainMoodRow, ExplainPunctuationRow,
    ExplainRow, ExplainTokenRow, explain_rows_for_text, render_text_canonical_buffer,
    sequence_events_for_text,
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
pub use performance::{
    PerformanceCurves, PerformancePlan, PerformanceSyllable, PhraseRole, plan_discourse_performance,
};
pub use phrase::{PhraseBoundaryStrength, PhrasePlan, PhraseSyllablePlan, plan_phrase_prosody};
pub use sequence::{
    DASH_HESITATION_PAUSE_SAMPLES, ELLIPSIS_HESITATION_PAUSE_SAMPLES, EMPTY_CHIRP_CONTOUR,
    EMPTY_CHIRP_PITCH_CENTER, EMPTY_CHIRP_START_PITCH_CENTER, EMPTY_CHIRP_VOWEL_POSITION,
    EMPTY_CHIRP_WARBLE_DEPTH, HesitationMarker, ProsodicPunctuation, ROLE_LONG_PAUSE_MAX_SAMPLES,
    ROLE_LONG_PAUSE_MIN_SAMPLES, STAGED_REPLY_REST_MAX_SAMPLES, STAGED_REPLY_REST_MIN_SAMPLES,
    SequenceEvent, SequencedUtterance, SyllableEvent, SyllableTiming, TailShape,
    estimate_utterance_sample_count, render_empty_chirp, role_long_pause_samples,
    sequence_utterance, staged_reply_rest_samples,
};
pub use synth::{
    ATTACK_TRANSIENT_MIX, ATTACK_TRANSIENT_SECONDS, BASE_SYLLABLE_SAMPLES, BASE_SYLLABLE_SECONDS,
    BODY_LAYER_MIX, CLAUSE_SYLLABLE_SAMPLES, ENVELOPE_ATTACK_SECONDS, ENVELOPE_DECAY_SECONDS,
    ENVELOPE_RELEASE_SECONDS, ENVELOPE_SUSTAIN_LEVEL, FORMANT_AH_HZ, FORMANT_COUNT, FORMANT_EE_HZ,
    FORMANT_GAINS, FORMANT_OO_HZ, FORMANT_Q, FormantFilterBank, INTERNAL_PITCH_ARCH_CENTS,
    INTERNAL_PITCH_SWEEP_CENTS, LEADING_SILENCE_SAMPLES, LEADING_SILENCE_SECONDS,
    LONG_PUNCTUATION_PAUSE_SAMPLES, LONG_PUNCTUATION_PAUSE_SECONDS,
    MEDIUM_PUNCTUATION_PAUSE_SAMPLES, MEDIUM_PUNCTUATION_PAUSE_SECONDS, MOUTH_RESONANCE_COUNT,
    MOUTH_STAGE_MAX_MIX, MouthDrive, MouthStage, NOISE_BREATH_MAX_MIX, PHRASE_EMPHASIS_GAIN,
    PHRASE_EMPHASIS_PITCH_SEMITONES, PITCH_REGISTER_BIAS_HZ, PITCH_SEMITONE_SPAN,
    PORTAMENTO_SECONDS, PUNCTUATION_GLIDE_SEMITONES, QUESTION_RISE_SEMITONES,
    RING_MOD_FREQUENCY_HZ, RING_MOD_MIX, SENTENCE_SYLLABLE_SAMPLES, SOURCE_MAX_HARMONICS,
    SOURCE_PULSE_MIX, SOURCE_PULSE_WIDTH, SOURCE_SAW_MIX, SYNTH_SAMPLE_RATE_HZ, Synth,
    TRAILING_SILENCE_SAMPLES, TRAILING_SILENCE_SECONDS, UPPER_MID_SPARKLE_MIX, VOWEL_LOCUS_COUNT,
    VOWEL_TRAJECTORY_BLOOM, VOWEL_TRAJECTORY_SWEEP, WARBLE_DEPTH_CENTS, WARBLE_DRIFT_RATE_HZ,
    WARBLE_FLUTTER_RATE_HZ, WARBLE_RATE_HZ, WHISTLE_FLOOR_HZ, WHISTLE_PITCH_CEILING_HZ,
    WHISTLE_TARGET_HZ,
    WIDE_GESTURE_PITCH_SPAN_SEMITONES, WORD_PAUSE_SAMPLES, WORD_PAUSE_SECONDS, amplitude_envelope,
    apply_amplitude_envelope, apply_internal_pitch_swoop_hz, apply_warble_hz,
    apply_warble_hz_with_phase, attack_transient_sample, blend_noise_excitation,
    body_layer_frequency_hz, body_layer_sample, compound_warble_offset_cents, formant_frequencies,
    imperfection_detune_cents, internal_pitch_offset_cents, mouth_open_envelope,
    mouth_resonance_hz, noise_breath_sample, pitch_center_hz, pitch_center_hz_with_span,
    apply_whistle_sweep_hz, portamento_pitch_hz, portamento_progress, render_syllable,
    ring_modulate, source_harmonic_count, source_oscillator_sample, sparkle_event_gain,
    upper_mid_sparkle_frequency_hz, upper_mid_sparkle_sample, vowel_trajectory_position,
    warble_depth_cents, warble_offset_cents, warble_phase_offset_for_syllable,
    whistle_sweep_amount, whistle_sweep_pitch_hz,
};
pub use tokenizer::{
    TokenizedInput, TokenizedToken, Tokenizer, TokenizerError, embedded_tokenizer,
};
pub use wav::{
    PCM_I16_SCALE, WavError, WavWriter, quantize_sample, render_canonical_buffer, wav_bytes,
    write_wav,
};
