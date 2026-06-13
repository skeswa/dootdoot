//! Formant synthesis voice engine for droid syllables.

use core::f64::consts::PI;

use crate::{cos, sin};

/// Gives the synthesis sample rate in hertz.
pub const SYNTH_SAMPLE_RATE_HZ: u32 = 44_100;

/// Gives the number of resonant formants in the fixed filter bank.
pub const FORMANT_COUNT: usize = 3;

/// Gives the number of vowel loci used to steer formants.
pub const VOWEL_LOCUS_COUNT: usize = 3;

/// Gives the fixed `ee` vowel formant centers in hertz.
pub const FORMANT_EE_HZ: [f64; FORMANT_COUNT] = [270.0, 2_290.0, 3_010.0];

/// Gives the fixed `ah` vowel formant centers in hertz.
pub const FORMANT_AH_HZ: [f64; FORMANT_COUNT] = [730.0, 1_090.0, 2_440.0];

/// Gives the fixed `oo` vowel formant centers in hertz.
pub const FORMANT_OO_HZ: [f64; FORMANT_COUNT] = [300.0, 870.0, 2_240.0];

/// Gives the fixed formant resonance Q values.
pub const FORMANT_Q: [f64; FORMANT_COUNT] = [8.0, 10.0, 12.0];

/// Gives the fixed per-formant gains.
pub const FORMANT_GAINS: [f64; FORMANT_COUNT] = [1.0, 0.55, 0.35];

/// Gives the fixed base syllable duration in seconds.
pub const BASE_SYLLABLE_SECONDS: f64 = 0.150;

/// Gives the fixed pause between separate words in seconds.
pub const WORD_PAUSE_SECONDS: f64 = 0.080;

/// Gives the pause for comma/semicolon/colon prosody in seconds.
pub const MEDIUM_PUNCTUATION_PAUSE_SECONDS: f64 = 0.120;

/// Gives the pause for question/period/exclamation prosody in seconds.
pub const LONG_PUNCTUATION_PAUSE_SECONDS: f64 = 0.180;

/// Gives the fixed leading silence in seconds.
pub const LEADING_SILENCE_SECONDS: f64 = 0.030;

/// Gives the fixed trailing silence in seconds.
pub const TRAILING_SILENCE_SECONDS: f64 = 0.060;

/// Gives the fixed portamento glide time in seconds.
pub const PORTAMENTO_SECONDS: f64 = 0.045;

/// Gives the fixed warble LFO rate in hertz.
pub const WARBLE_RATE_HZ: f64 = 8.5;

/// Gives the maximum semantic warble depth in cents.
pub const WARBLE_DEPTH_CENTS: f64 = 45.0;

/// Gives the fixed ring-modulator frequency in hertz.
pub const RING_MOD_FREQUENCY_HZ: f64 = 72.0;

/// Gives the fixed ring-modulator wet mix.
pub const RING_MOD_MIX: f64 = 0.08;

/// Gives the fixed envelope attack in seconds.
pub const ENVELOPE_ATTACK_SECONDS: f64 = 0.012;

/// Gives the fixed envelope decay in seconds.
pub const ENVELOPE_DECAY_SECONDS: f64 = 0.080;

/// Gives the fixed envelope release in seconds.
pub const ENVELOPE_RELEASE_SECONDS: f64 = 0.025;

/// Gives the high-register pitch bias in hertz.
pub const PITCH_REGISTER_BIAS_HZ: f64 = 880.0;

/// Gives the semantic pitch span around the register bias in semitones.
pub const PITCH_SEMITONE_SPAN: f64 = 7.0;

/// Gives the fixed saw contribution in the source oscillator.
pub const SOURCE_SAW_MIX: f64 = 0.65;

/// Gives the fixed pulse contribution in the source oscillator.
pub const SOURCE_PULSE_MIX: f64 = 0.35;

/// Gives the fixed pulse width in the source oscillator.
pub const SOURCE_PULSE_WIDTH: f64 = 0.42;

/// Gives the maximum additive harmonics used by the source oscillator.
pub const SOURCE_MAX_HARMONICS: u32 = 48;

/// Marks the synthesis module in the public facade.
#[derive(Debug)]
pub struct Synth;

/// Counts active source harmonics below Nyquist for a frequency.
pub fn source_harmonic_count(frequency_hz: f64) -> u32 {
    if !frequency_hz.is_finite() || frequency_hz <= 0.0 {
        return 0;
    }

    let nyquist_hz = f64::from(SYNTH_SAMPLE_RATE_HZ) / 2.0;
    let mut count = 0;

    for harmonic in 1..=SOURCE_MAX_HARMONICS {
        if f64::from(harmonic) * frequency_hz < nyquist_hz {
            count = harmonic;
        } else {
            break;
        }
    }

    count
}

/// Evaluates the fixed harmonically-rich source oscillator.
pub fn source_oscillator_sample(phase: f64, frequency_hz: f64) -> f64 {
    let harmonic_count = source_harmonic_count(frequency_hz);

    if harmonic_count == 0 {
        return 0.0;
    }

    let phase = wrap_phase(phase);
    let mut saw = 0.0;
    let mut pulse = (2.0 * SOURCE_PULSE_WIDTH) - 1.0;
    let two_pi = 2.0 * PI;

    for harmonic in 1..=harmonic_count {
        let harmonic_f64 = f64::from(harmonic);
        let angle = two_pi * harmonic_f64 * phase;
        let sign = if harmonic % 2 == 0 { -1.0 } else { 1.0 };
        saw += sign * sin(angle) / harmonic_f64;

        let duty_angle = two_pi * harmonic_f64 * SOURCE_PULSE_WIDTH;
        let cosine_coefficient = 2.0 * sin(duty_angle) / (PI * harmonic_f64);
        let sine_coefficient = 2.0 * (1.0 - cos(duty_angle)) / (PI * harmonic_f64);
        pulse += (cosine_coefficient * cos(angle)) + (sine_coefficient * sin(angle));
    }

    let saw = (2.0 / PI) * saw;

    (SOURCE_SAW_MIX * saw) + (SOURCE_PULSE_MIX * pulse)
}

fn wrap_phase(phase: f64) -> f64 {
    if !phase.is_finite() {
        return 0.0;
    }

    let wrapped = phase % 1.0;

    if wrapped < 0.0 {
        wrapped + 1.0
    } else {
        wrapped
    }
}
