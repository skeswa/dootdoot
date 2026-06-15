//! Fixed synthesis constant tests.

use dootdoot_core::{
    BASE_SYLLABLE_SAMPLES, BASE_SYLLABLE_SECONDS, EMPTY_CHIRP_CONTOUR, EMPTY_CHIRP_PITCH_CENTER,
    EMPTY_CHIRP_START_PITCH_CENTER, EMPTY_CHIRP_VOWEL_POSITION, EMPTY_CHIRP_WARBLE_DEPTH,
    ENVELOPE_ATTACK_SECONDS, ENVELOPE_DECAY_SECONDS, ENVELOPE_RELEASE_SECONDS, FORMANT_AH_HZ,
    FORMANT_COUNT, FORMANT_EE_HZ, FORMANT_GAINS, FORMANT_OO_HZ, FORMANT_Q,
    INTERNAL_PITCH_ARCH_CENTS, INTERNAL_PITCH_SWEEP_CENTS, LEADING_SILENCE_SAMPLES,
    LEADING_SILENCE_SECONDS, LONG_PUNCTUATION_PAUSE_SAMPLES, LONG_PUNCTUATION_PAUSE_SECONDS,
    MEDIUM_PUNCTUATION_PAUSE_SAMPLES, MEDIUM_PUNCTUATION_PAUSE_SECONDS, PITCH_REGISTER_BIAS_HZ,
    PITCH_SEMITONE_SPAN, PORTAMENTO_SECONDS, PUNCTUATION_GLIDE_SEMITONES, RING_MOD_FREQUENCY_HZ,
    RING_MOD_MIX, SOURCE_PULSE_MIX, SOURCE_PULSE_WIDTH, SOURCE_SAW_MIX, SYNTH_SAMPLE_RATE_HZ,
    TRAILING_SILENCE_SAMPLES, TRAILING_SILENCE_SECONDS, VOWEL_LOCUS_COUNT, VOWEL_TRAJECTORY_BLOOM,
    VOWEL_TRAJECTORY_SWEEP, WARBLE_DEPTH_CENTS, WARBLE_RATE_HZ, WORD_PAUSE_SAMPLES,
    WORD_PAUSE_SECONDS,
};

const DESIGN: &str = include_str!("../../docs/design.md");

#[test]
fn format_v1_synthesis_constants_are_pinned() {
    assert_eq!(runtime(SYNTH_SAMPLE_RATE_HZ), 44_100);
    assert_eq!(runtime(FORMANT_COUNT), 3);
    assert_eq!(runtime(VOWEL_LOCUS_COUNT), 3);
    assert_eq!(bits(runtime(BASE_SYLLABLE_SECONDS)), bits(0.150));
    assert_eq!(bits(runtime(WORD_PAUSE_SECONDS)), bits(0.080));
    assert_eq!(bits(runtime(MEDIUM_PUNCTUATION_PAUSE_SECONDS)), bits(0.120));
    assert_eq!(bits(runtime(LONG_PUNCTUATION_PAUSE_SECONDS)), bits(0.180));
    assert_eq!(bits(runtime(LEADING_SILENCE_SECONDS)), bits(0.030));
    assert_eq!(bits(runtime(TRAILING_SILENCE_SECONDS)), bits(0.060));
    assert_eq!(runtime(BASE_SYLLABLE_SAMPLES), 6_615);
    assert_eq!(runtime(WORD_PAUSE_SAMPLES), 3_528);
    assert_eq!(runtime(MEDIUM_PUNCTUATION_PAUSE_SAMPLES), 5_292);
    assert_eq!(runtime(LONG_PUNCTUATION_PAUSE_SAMPLES), 7_938);
    assert_eq!(runtime(LEADING_SILENCE_SAMPLES), 1_323);
    assert_eq!(runtime(TRAILING_SILENCE_SAMPLES), 2_646);
    assert_eq!(bits(runtime(PORTAMENTO_SECONDS)), bits(0.045));
    assert_eq!(bits(runtime(WARBLE_RATE_HZ)), bits(8.5));
    assert_eq!(bits(runtime(WARBLE_DEPTH_CENTS)), bits(45.0));
    assert_eq!(bits(runtime(INTERNAL_PITCH_SWEEP_CENTS)), bits(220.0));
    assert_eq!(bits(runtime(INTERNAL_PITCH_ARCH_CENTS)), bits(90.0));
    assert_eq!(bits(runtime(VOWEL_TRAJECTORY_SWEEP)), bits(0.18));
    assert_eq!(bits(runtime(VOWEL_TRAJECTORY_BLOOM)), bits(0.12));
    assert_eq!(bits(runtime(PUNCTUATION_GLIDE_SEMITONES)), bits(3.0));
    assert_eq!(bits(runtime(RING_MOD_FREQUENCY_HZ)), bits(72.0));
    assert_eq!(bits(runtime(RING_MOD_MIX)), bits(0.08));
    assert_eq!(bits(runtime(ENVELOPE_ATTACK_SECONDS)), bits(0.012));
    assert_eq!(bits(runtime(ENVELOPE_DECAY_SECONDS)), bits(0.080));
    assert_eq!(bits(runtime(ENVELOPE_RELEASE_SECONDS)), bits(0.025));
    assert_eq!(bits(runtime(PITCH_REGISTER_BIAS_HZ)), bits(880.0));
    assert_eq!(bits(runtime(PITCH_SEMITONE_SPAN)), bits(7.0));
    assert_eq!(bits(runtime(EMPTY_CHIRP_START_PITCH_CENTER)), bits(-0.35));
    assert_eq!(bits(runtime(EMPTY_CHIRP_PITCH_CENTER)), bits(0.45));
    assert_eq!(bits(runtime(EMPTY_CHIRP_VOWEL_POSITION)), bits(0.15));
    assert_eq!(bits(runtime(EMPTY_CHIRP_CONTOUR)), bits(1.0));
    assert_eq!(bits(runtime(EMPTY_CHIRP_WARBLE_DEPTH)), bits(0.85));
    assert_eq!(bits(runtime(SOURCE_SAW_MIX)), bits(0.65));
    assert_eq!(bits(runtime(SOURCE_PULSE_MIX)), bits(0.35));
    assert_eq!(bits(runtime(SOURCE_PULSE_WIDTH)), bits(0.42));
}

#[test]
fn vowel_locus_and_formant_shape_are_pinned() {
    assert_eq!(
        runtime(FORMANT_EE_HZ).map(bits),
        [bits(270.0), bits(2_290.0), bits(3_010.0)]
    );
    assert_eq!(
        runtime(FORMANT_AH_HZ).map(bits),
        [bits(730.0), bits(1_090.0), bits(2_440.0)]
    );
    assert_eq!(
        runtime(FORMANT_OO_HZ).map(bits),
        [bits(300.0), bits(870.0), bits(2_240.0)]
    );
    assert_eq!(
        runtime(FORMANT_Q).map(bits),
        [bits(8.0), bits(10.0), bits(12.0)]
    );
    assert_eq!(
        runtime(FORMANT_GAINS).map(bits),
        [bits(1.0), bits(0.55), bits(0.35)]
    );
}

#[test]
fn design_documents_the_frozen_synthesis_constants() {
    for expected in [
        "FORMAT_V1 synthesis constants",
        "base syllable = 150 ms",
        "word pause = 80 ms",
        "warble rate = 8.5 Hz",
        "ring-mod = 72 Hz at 8% mix",
    ] {
        assert!(
            DESIGN.contains(expected),
            "design.md should mention {expected}"
        );
    }
}

fn runtime<T>(value: T) -> T {
    std::hint::black_box(value)
}

fn bits(value: f64) -> u64 {
    value.to_bits()
}
