//! Formant synthesis voice engine for droid syllables.

use core::f64::consts::{LN_2, PI};

use crate::{cos, exp, sin};

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

/// Gives the fixed base syllable duration in samples.
pub const BASE_SYLLABLE_SAMPLES: u32 = 6_615;

/// Gives the fixed pause between separate words in seconds.
pub const WORD_PAUSE_SECONDS: f64 = 0.080;

/// Gives the fixed pause between separate words in samples.
pub const WORD_PAUSE_SAMPLES: u32 = 3_528;

/// Gives the pause for comma/semicolon/colon prosody in seconds.
pub const MEDIUM_PUNCTUATION_PAUSE_SECONDS: f64 = 0.120;

/// Gives the pause for comma/semicolon/colon prosody in samples.
pub const MEDIUM_PUNCTUATION_PAUSE_SAMPLES: u32 = 5_292;

/// Gives the pause for question/period/exclamation prosody in seconds.
pub const LONG_PUNCTUATION_PAUSE_SECONDS: f64 = 0.180;

/// Gives the pause for question/period/exclamation prosody in samples.
pub const LONG_PUNCTUATION_PAUSE_SAMPLES: u32 = 7_938;

/// Gives the fixed leading silence in seconds.
pub const LEADING_SILENCE_SECONDS: f64 = 0.030;

/// Gives the fixed leading silence in samples.
pub const LEADING_SILENCE_SAMPLES: u32 = 1_323;

/// Gives the fixed trailing silence in seconds.
pub const TRAILING_SILENCE_SECONDS: f64 = 0.060;

/// Gives the fixed trailing silence in samples.
pub const TRAILING_SILENCE_SAMPLES: u32 = 2_646;

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

/// Gives the fixed envelope sustain level after decay.
pub const ENVELOPE_SUSTAIN_LEVEL: f64 = 0.35;

/// Gives the high-register pitch bias in hertz.
pub const PITCH_REGISTER_BIAS_HZ: f64 = 880.0;

/// Gives the semantic pitch span around the register bias in semitones.
pub const PITCH_SEMITONE_SPAN: f64 = 7.0;

/// Gives the fixed contour-steered internal pitch sweep in cents.
pub const INTERNAL_PITCH_SWEEP_CENTS: f64 = 220.0;

/// Gives the fixed internal pitch arch in cents.
pub const INTERNAL_PITCH_ARCH_CENTS: f64 = 90.0;

/// Gives the fixed contour-steered internal vowel sweep.
pub const VOWEL_TRAJECTORY_SWEEP: f64 = 0.18;

/// Gives the fixed internal vowel opening bloom.
pub const VOWEL_TRAJECTORY_BLOOM: f64 = 0.12;

/// Gives the fixed final-glide span for prosodic punctuation in semitones.
pub const PUNCTUATION_GLIDE_SEMITONES: f64 = 3.0;

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

/// Gives the prosodic final-glide shape applied to one syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyllableFinalGlide {
    Neutral,
    Rising,
    Falling,
}

/// Filters samples through the fixed formant bank.
#[derive(Debug, Clone, Default)]
pub struct FormantFilterBank {
    filters: [BandpassFilter; FORMANT_COUNT],
}

#[derive(Debug, Clone, Copy, Default)]
struct BandpassFilter {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl FormantFilterBank {
    /// Builds a silent formant filter bank.
    pub fn new() -> Self {
        Self::default()
    }

    /// Processes one sample through vowel-position-steered formants.
    pub fn process_sample(&mut self, input: f64, vowel_position: f64) -> f64 {
        let frequencies = formant_frequencies(vowel_position);
        let mut output = 0.0;

        for (((filter, frequency_hz), q), gain) in self
            .filters
            .iter_mut()
            .zip(frequencies)
            .zip(FORMANT_Q)
            .zip(FORMANT_GAINS)
        {
            output += filter.process_sample(input, frequency_hz, q) * gain;
        }

        output
    }

    /// Clears all filter delay state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

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

/// Maps a pitch-center knob to hertz in the fixed high register.
pub fn pitch_center_hz(pitch_center: f64) -> f64 {
    let semitones = pitch_center.clamp(-1.0, 1.0) * PITCH_SEMITONE_SPAN;

    PITCH_REGISTER_BIAS_HZ * exp((LN_2 * semitones) / 12.0)
}

/// Computes shaped portamento progress for elapsed time.
pub fn portamento_progress(elapsed_seconds: f64, contour: f64) -> f64 {
    if !elapsed_seconds.is_finite() || elapsed_seconds <= 0.0 {
        return 0.0;
    }

    let linear = (elapsed_seconds / PORTAMENTO_SECONDS).clamp(0.0, 1.0);
    let smooth = linear * linear * (3.0 - (2.0 * linear));
    let bend = linear * (1.0 - linear) * ((2.0 * linear) - 1.0);

    (smooth + (contour.clamp(-1.0, 1.0) * bend)).clamp(0.0, 1.0)
}

/// Interpolates pitch through portamento from one center to the next.
pub fn portamento_pitch_hz(
    start_hz: f64,
    target_hz: f64,
    contour: f64,
    elapsed_seconds: f64,
) -> f64 {
    let progress = portamento_progress(elapsed_seconds, contour);

    start_hz + ((target_hz - start_hz) * progress)
}

/// Computes the deterministic per-syllable pitch micro-gesture in cents.
pub fn internal_pitch_offset_cents(contour: f64, elapsed_seconds: f64) -> f64 {
    let progress = syllable_progress(elapsed_seconds);
    let sweep = contour.clamp(-1.0, 1.0) * INTERNAL_PITCH_SWEEP_CENTS * (1.0 - (2.0 * progress));
    let arch = INTERNAL_PITCH_ARCH_CENTS * sin(PI * progress);

    sweep + arch
}

/// Applies the deterministic per-syllable pitch micro-gesture.
pub fn apply_internal_pitch_swoop_hz(pitch_hz: f64, contour: f64, elapsed_seconds: f64) -> f64 {
    let cents = internal_pitch_offset_cents(contour, elapsed_seconds);

    pitch_hz * exp(LN_2 * (cents / 1_200.0))
}

/// Maps a warble-depth knob to a nonnegative vibrato depth in cents.
pub fn warble_depth_cents(warble_depth: f64) -> f64 {
    ((warble_depth.clamp(-1.0, 1.0) + 1.0) * 0.5) * WARBLE_DEPTH_CENTS
}

/// Computes the fixed-rate warble pitch offset in cents.
pub fn warble_offset_cents(warble_depth: f64, elapsed_seconds: f64) -> f64 {
    if !elapsed_seconds.is_finite() {
        return 0.0;
    }

    let phase = 2.0 * PI * WARBLE_RATE_HZ * elapsed_seconds;

    warble_depth_cents(warble_depth) * sin(phase)
}

/// Applies fixed-rate warble to a pitch in hertz.
pub fn apply_warble_hz(pitch_hz: f64, warble_depth: f64, elapsed_seconds: f64) -> f64 {
    let cents = warble_offset_cents(warble_depth, elapsed_seconds);

    pitch_hz * exp(LN_2 * (cents / 1_200.0))
}

/// Applies the fixed faint ring modulator to a sample.
pub fn ring_modulate(sample: f64, elapsed_seconds: f64) -> f64 {
    if !elapsed_seconds.is_finite() {
        return sample;
    }

    let phase = 2.0 * PI * RING_MOD_FREQUENCY_HZ * elapsed_seconds;
    let carrier = sin(phase);

    sample * ((1.0 - RING_MOD_MIX) + (RING_MOD_MIX * carrier))
}

/// Computes the fixed per-syllable amplitude envelope.
pub fn amplitude_envelope(elapsed_seconds: f64, duration_seconds: f64) -> f64 {
    if !elapsed_seconds.is_finite()
        || !duration_seconds.is_finite()
        || elapsed_seconds <= 0.0
        || duration_seconds <= 0.0
        || elapsed_seconds >= duration_seconds
    {
        return 0.0;
    }

    if elapsed_seconds <= ENVELOPE_ATTACK_SECONDS {
        return (elapsed_seconds / ENVELOPE_ATTACK_SECONDS).clamp(0.0, 1.0);
    }

    let decay_end = ENVELOPE_ATTACK_SECONDS + ENVELOPE_DECAY_SECONDS;

    if elapsed_seconds <= decay_end {
        let progress =
            ((elapsed_seconds - ENVELOPE_ATTACK_SECONDS) / ENVELOPE_DECAY_SECONDS).clamp(0.0, 1.0);

        return 1.0 + ((ENVELOPE_SUSTAIN_LEVEL - 1.0) * progress);
    }

    let release_start = (duration_seconds - ENVELOPE_RELEASE_SECONDS).max(decay_end);

    if elapsed_seconds >= release_start {
        let release_seconds = duration_seconds - release_start;

        if release_seconds <= 0.0 {
            return 0.0;
        }

        return ENVELOPE_SUSTAIN_LEVEL
            * ((duration_seconds - elapsed_seconds) / release_seconds).clamp(0.0, 1.0);
    }

    ENVELOPE_SUSTAIN_LEVEL
}

/// Applies the fixed per-syllable amplitude envelope to a sample.
pub fn apply_amplitude_envelope(sample: f64, elapsed_seconds: f64, duration_seconds: f64) -> f64 {
    sample * amplitude_envelope(elapsed_seconds, duration_seconds)
}

/// Computes the deterministic per-syllable vowel trajectory.
pub fn vowel_trajectory_position(vowel_position: f64, contour: f64, elapsed_seconds: f64) -> f64 {
    let progress = syllable_progress(elapsed_seconds);
    let sweep = contour.clamp(-1.0, 1.0) * VOWEL_TRAJECTORY_SWEEP * ((2.0 * progress) - 1.0);
    let bloom = VOWEL_TRAJECTORY_BLOOM * sin(PI * progress);

    (vowel_position + sweep + bloom).clamp(-1.0, 1.0)
}

/// Renders one fixed-duration syllable from semantic knobs.
pub fn render_syllable(knobs: crate::KnobSet, start_pitch_hz: f64) -> Vec<f64> {
    render_syllable_with_final_glide(knobs, start_pitch_hz, SyllableFinalGlide::Neutral)
}

pub(crate) fn render_syllable_with_final_glide(
    knobs: crate::KnobSet,
    start_pitch_hz: f64,
    final_glide: SyllableFinalGlide,
) -> Vec<f64> {
    let target_pitch_hz = pitch_center_hz(knobs.pitch_center());
    let start_pitch_hz = if start_pitch_hz.is_finite() && start_pitch_hz > 0.0 {
        start_pitch_hz
    } else {
        target_pitch_hz
    };
    let mut phase = 0.0;
    let mut formants = FormantFilterBank::new();
    let mut samples = Vec::new();

    for sample_index in 0..BASE_SYLLABLE_SAMPLES {
        let elapsed_seconds = f64::from(sample_index) / f64::from(SYNTH_SAMPLE_RATE_HZ);
        let glided_pitch_hz = portamento_pitch_hz(
            start_pitch_hz,
            target_pitch_hz,
            knobs.contour(),
            elapsed_seconds,
        );
        let final_glide_pitch_hz = apply_final_glide_hz(
            glided_pitch_hz,
            target_pitch_hz,
            final_glide,
            elapsed_seconds,
        );
        let internal_pitch_hz =
            apply_internal_pitch_swoop_hz(final_glide_pitch_hz, knobs.contour(), elapsed_seconds);
        let pitch_hz = apply_warble_hz(internal_pitch_hz, knobs.warble_depth(), elapsed_seconds);
        let source = source_oscillator_sample(phase, pitch_hz);
        let vowel_position =
            vowel_trajectory_position(knobs.vowel_position(), knobs.contour(), elapsed_seconds);
        let voiced = formants.process_sample(source, vowel_position);
        let electronic = ring_modulate(voiced, elapsed_seconds);

        samples.push(apply_amplitude_envelope(
            electronic,
            elapsed_seconds,
            BASE_SYLLABLE_SECONDS,
        ));

        phase = wrap_phase(phase + (pitch_hz / f64::from(SYNTH_SAMPLE_RATE_HZ)));
    }

    samples
}

fn apply_final_glide_hz(
    pitch_hz: f64,
    target_pitch_hz: f64,
    final_glide: SyllableFinalGlide,
    elapsed_seconds: f64,
) -> f64 {
    let semitones = match final_glide {
        SyllableFinalGlide::Neutral => return pitch_hz,
        SyllableFinalGlide::Rising => PUNCTUATION_GLIDE_SEMITONES,
        SyllableFinalGlide::Falling => -PUNCTUATION_GLIDE_SEMITONES,
    };
    let final_glide_start_seconds = BASE_SYLLABLE_SECONDS - PORTAMENTO_SECONDS;

    if elapsed_seconds < final_glide_start_seconds {
        return pitch_hz;
    }

    let progress = portamento_progress(elapsed_seconds - final_glide_start_seconds, semitones);
    let final_target_hz = target_pitch_hz * exp((LN_2 * semitones) / 12.0);

    pitch_hz + ((final_target_hz - pitch_hz) * progress)
}

/// Returns steered formant centers for a vowel-position knob.
pub fn formant_frequencies(vowel_position: f64) -> [f64; FORMANT_COUNT] {
    let position = vowel_position.clamp(-1.0, 1.0);

    if position <= 0.0 {
        interpolate_formants(FORMANT_EE_HZ, FORMANT_AH_HZ, position + 1.0)
    } else {
        interpolate_formants(FORMANT_AH_HZ, FORMANT_OO_HZ, position)
    }
}

fn interpolate_formants(
    from: [f64; FORMANT_COUNT],
    to: [f64; FORMANT_COUNT],
    amount: f64,
) -> [f64; FORMANT_COUNT] {
    std::array::from_fn(|index| from[index] + ((to[index] - from[index]) * amount))
}

impl BandpassFilter {
    fn process_sample(&mut self, input: f64, center_hz: f64, q: f64) -> f64 {
        let omega = (2.0 * PI * center_hz) / f64::from(SYNTH_SAMPLE_RATE_HZ);
        let sin_omega = sin(omega);
        let cos_omega = cos(omega);
        let alpha = sin_omega / (2.0 * q);
        let a0 = 1.0 + alpha;
        let b0 = alpha / a0;
        let b2 = -alpha / a0;
        let a1 = (-2.0 * cos_omega) / a0;
        let a2 = (1.0 - alpha) / a0;
        let output = (b0 * input) + (b2 * self.x2) - (a1 * self.y1) - (a2 * self.y2);

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
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

fn syllable_progress(elapsed_seconds: f64) -> f64 {
    if !elapsed_seconds.is_finite() {
        return 0.0;
    }

    (elapsed_seconds / BASE_SYLLABLE_SECONDS).clamp(0.0, 1.0)
}
