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
pub const FORMANT_EE_HZ: [f64; FORMANT_COUNT] = [300.0, 2_360.0, 3_260.0];

/// Gives the fixed `ah` vowel formant centers in hertz.
pub const FORMANT_AH_HZ: [f64; FORMANT_COUNT] = [620.0, 1_280.0, 2_700.0];

/// Gives the fixed `oo` vowel formant centers in hertz.
pub const FORMANT_OO_HZ: [f64; FORMANT_COUNT] = [280.0, 760.0, 2_500.0];

/// Gives the fixed formant resonance Q values.
pub const FORMANT_Q: [f64; FORMANT_COUNT] = [5.5, 7.0, 8.0];

/// Gives the fixed per-formant gains.
pub const FORMANT_GAINS: [f64; FORMANT_COUNT] = [0.52, 0.42, 0.78];

/// Gives the fixed base syllable duration in seconds.
pub const BASE_SYLLABLE_SECONDS: f64 = 0.170;

/// Gives the fixed base syllable duration in samples.
pub const BASE_SYLLABLE_SAMPLES: u32 = 7_497;

/// Gives the lengthened syllable samples before a clause boundary.
pub const CLAUSE_SYLLABLE_SAMPLES: u32 = 8_397;

/// Gives the lengthened syllable samples before a sentence boundary.
pub const SENTENCE_SYLLABLE_SAMPLES: u32 = 9_371;

/// Gives the fixed pause between separate words in seconds.
pub const WORD_PAUSE_SECONDS: f64 = 0.110;

/// Gives the fixed pause between separate words in samples.
pub const WORD_PAUSE_SAMPLES: u32 = 4_851;

/// Gives the pause for comma/semicolon/colon prosody in seconds.
pub const MEDIUM_PUNCTUATION_PAUSE_SECONDS: f64 = 0.150;

/// Gives the pause for comma/semicolon/colon prosody in samples.
pub const MEDIUM_PUNCTUATION_PAUSE_SAMPLES: u32 = 6_615;

/// Gives the pause for question/period/exclamation prosody in seconds.
pub const LONG_PUNCTUATION_PAUSE_SECONDS: f64 = 0.240;

/// Gives the pause for question/period/exclamation prosody in samples.
pub const LONG_PUNCTUATION_PAUSE_SAMPLES: u32 = 10_584;

/// Gives the fixed leading silence in seconds.
pub const LEADING_SILENCE_SECONDS: f64 = 0.030;

/// Gives the fixed leading silence in samples.
pub const LEADING_SILENCE_SAMPLES: u32 = 1_323;

/// Gives the fixed trailing silence in seconds.
pub const TRAILING_SILENCE_SECONDS: f64 = 0.090;

/// Gives the fixed trailing silence in samples.
pub const TRAILING_SILENCE_SAMPLES: u32 = 3_969;

/// Gives the fixed portamento glide time in seconds.
pub const PORTAMENTO_SECONDS: f64 = 0.045;

/// Gives the fixed warble LFO rate in hertz.
pub const WARBLE_RATE_HZ: f64 = 8.5;

/// Gives the fixed slow warble drift rate in hertz.
pub const WARBLE_DRIFT_RATE_HZ: f64 = 3.1;

/// Gives the fixed fast warble flutter rate in hertz.
pub const WARBLE_FLUTTER_RATE_HZ: f64 = 15.7;

/// Gives the maximum semantic warble depth in cents.
pub const WARBLE_DEPTH_CENTS: f64 = 45.0;

/// Gives the fixed ring-modulator frequency in hertz.
pub const RING_MOD_FREQUENCY_HZ: f64 = 72.0;

/// Gives the fixed ring-modulator wet mix.
pub const RING_MOD_MIX: f64 = 0.08;

/// Gives the fixed envelope attack in seconds.
pub const ENVELOPE_ATTACK_SECONDS: f64 = 0.006;

/// Gives the fixed envelope decay in seconds.
pub const ENVELOPE_DECAY_SECONDS: f64 = 0.050;

/// Gives the fixed envelope release in seconds.
pub const ENVELOPE_RELEASE_SECONDS: f64 = 0.060;

/// Gives the fixed envelope sustain level after decay.
pub const ENVELOPE_SUSTAIN_LEVEL: f64 = 0.24;

/// Gives the high-register pitch bias in hertz.
pub const PITCH_REGISTER_BIAS_HZ: f64 = 760.0;

/// Gives the semantic pitch span around the register bias in semitones.
pub const PITCH_SEMITONE_SPAN: f64 = 10.0;

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
pub const SOURCE_SAW_MIX: f64 = 0.55;

/// Gives the fixed pulse contribution in the source oscillator.
pub const SOURCE_PULSE_MIX: f64 = 0.45;

/// Gives the fixed pulse width in the source oscillator.
pub const SOURCE_PULSE_WIDTH: f64 = 0.38;

/// Gives the maximum additive harmonics used by the source oscillator.
pub const SOURCE_MAX_HARMONICS: u32 = 48;

/// Gives the fixed attack transient duration in seconds.
pub const ATTACK_TRANSIENT_SECONDS: f64 = 0.020;

/// Gives the fixed attack transient wet mix.
pub const ATTACK_TRANSIENT_MIX: f64 = 0.07;

/// Gives the fixed low-body layer wet mix.
pub const BODY_LAYER_MIX: f64 = 0.18;

/// Gives the fixed upper-mid sparkle wet mix.
pub const UPPER_MID_SPARKLE_MIX: f64 = 0.045;

/// Gives the fixed pitch lift for sparse phrase emphasis.
pub const PHRASE_EMPHASIS_PITCH_SEMITONES: f64 = 0.35;

/// Gives the fixed amplitude lift for sparse phrase emphasis.
pub const PHRASE_EMPHASIS_GAIN: f64 = 1.08;

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

/// Gives phrase-level controls applied while rendering one syllable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SyllablePerformance {
    duration_samples: u32,
    pitch_offset_semitones: f64,
    final_lowering_semitones: f64,
    emphasized: bool,
    mood_valence: f64,
    mood_arousal: f64,
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
    internal_pitch_offset_cents_for_duration(contour, elapsed_seconds, BASE_SYLLABLE_SECONDS)
}

fn internal_pitch_offset_cents_for_duration(
    contour: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    let progress = syllable_progress(elapsed_seconds, duration_seconds);
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

/// Computes the compound deterministic warble pitch offset in cents.
pub fn compound_warble_offset_cents(
    warble_depth: f64,
    elapsed_seconds: f64,
    phase_offset: f64,
) -> f64 {
    if !elapsed_seconds.is_finite() {
        return 0.0;
    }

    let amount = (warble_depth.clamp(-1.0, 1.0) + 1.0) * 0.5;

    if amount <= 0.0 {
        return 0.0;
    }

    let phase_offset = wrap_phase(phase_offset);
    let slow = sin(2.0 * PI * ((WARBLE_DRIFT_RATE_HZ * elapsed_seconds) + phase_offset));
    let primary = sin(2.0 * PI * ((WARBLE_RATE_HZ * elapsed_seconds) + (phase_offset * 0.61)));
    let flutter =
        sin(2.0 * PI * ((WARBLE_FLUTTER_RATE_HZ * elapsed_seconds) + (phase_offset * 1.37)));
    let complexity = 0.55 + (0.45 * amount);
    let shape = (0.44 * slow) + (0.42 * primary) + (0.14 * complexity * flutter);

    (WARBLE_DEPTH_CENTS * amount * shape).clamp(-WARBLE_DEPTH_CENTS, WARBLE_DEPTH_CENTS)
}

/// Computes the default compound warble pitch offset in cents.
pub fn warble_offset_cents(warble_depth: f64, elapsed_seconds: f64) -> f64 {
    compound_warble_offset_cents(warble_depth, elapsed_seconds, 0.0)
}

/// Gives the deterministic warble phase offset for a syllable index.
pub fn warble_phase_offset_for_syllable(syllable_index: usize) -> f64 {
    let index = u32::try_from(syllable_index).unwrap_or(u32::MAX);

    wrap_phase(f64::from(index) * 0.381_966_011_250_105_1)
}

/// Applies compound warble to a pitch in hertz.
pub fn apply_warble_hz_with_phase(
    pitch_hz: f64,
    warble_depth: f64,
    elapsed_seconds: f64,
    phase_offset: f64,
) -> f64 {
    let cents = compound_warble_offset_cents(warble_depth, elapsed_seconds, phase_offset);

    pitch_hz * exp(LN_2 * (cents / 1_200.0))
}

/// Applies default compound warble to a pitch in hertz.
pub fn apply_warble_hz(pitch_hz: f64, warble_depth: f64, elapsed_seconds: f64) -> f64 {
    apply_warble_hz_with_phase(pitch_hz, warble_depth, elapsed_seconds, 0.0)
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
        let progress = (elapsed_seconds / ENVELOPE_ATTACK_SECONDS).clamp(0.0, 1.0);

        return progress * progress;
    }

    let release_start = (duration_seconds - ENVELOPE_RELEASE_SECONDS)
        .max(ENVELOPE_ATTACK_SECONDS + ENVELOPE_DECAY_SECONDS);

    if elapsed_seconds >= release_start {
        let release_seconds = duration_seconds - release_start;

        if release_seconds <= 0.0 {
            return 0.0;
        }

        let progress = ((duration_seconds - elapsed_seconds) / release_seconds).clamp(0.0, 1.0);

        return ENVELOPE_SUSTAIN_LEVEL * progress * progress;
    }

    let decay_progress =
        ((elapsed_seconds - ENVELOPE_ATTACK_SECONDS) / ENVELOPE_DECAY_SECONDS).clamp(0.0, 1.0);
    let inverse_decay = 1.0 - decay_progress;
    let body =
        ENVELOPE_SUSTAIN_LEVEL + ((1.0 - ENVELOPE_SUSTAIN_LEVEL) * inverse_decay * inverse_decay);
    let pulse = 0.22 * triangle_window(elapsed_seconds, ENVELOPE_ATTACK_SECONDS + 0.012, 0.018);
    let dip = 0.30
        * triangle_window(
            elapsed_seconds,
            ENVELOPE_ATTACK_SECONDS + ENVELOPE_DECAY_SECONDS,
            0.020,
        );

    (body + pulse - dip).clamp(0.0, 1.0)
}

/// Applies the fixed per-syllable amplitude envelope to a sample.
pub fn apply_amplitude_envelope(sample: f64, elapsed_seconds: f64, duration_seconds: f64) -> f64 {
    sample * amplitude_envelope(elapsed_seconds, duration_seconds)
}

/// Computes the deterministic attack transient layer sample.
pub fn attack_transient_sample(elapsed_seconds: f64, contour: f64) -> f64 {
    if !elapsed_seconds.is_finite()
        || elapsed_seconds <= 0.0
        || elapsed_seconds >= ATTACK_TRANSIENT_SECONDS
    {
        return 0.0;
    }

    let progress = (elapsed_seconds / ATTACK_TRANSIENT_SECONDS).clamp(0.0, 1.0);
    let envelope = (1.0 - progress) * (1.0 - progress);
    let contour_shift_hz = contour.clamp(-1.0, 1.0) * 180.0;
    let first = sin(2.0 * PI * (3_170.0 + contour_shift_hz) * elapsed_seconds);
    let second = sin(2.0 * PI * (4_310.0 - contour_shift_hz) * elapsed_seconds);
    let transient = ((0.65 * first) + (0.35 * second)) * envelope * ATTACK_TRANSIENT_MIX;

    transient.clamp(-ATTACK_TRANSIENT_MIX, ATTACK_TRANSIENT_MIX)
}

/// Computes the low-body layer frequency in hertz.
pub fn body_layer_frequency_hz(pitch_hz: f64) -> f64 {
    if !pitch_hz.is_finite() || pitch_hz <= 0.0 {
        return 440.0;
    }

    (pitch_hz * 0.5).clamp(300.0, 700.0)
}

/// Computes the deterministic low-body layer sample.
pub fn body_layer_sample(phase: f64, vowel_position: f64) -> f64 {
    let vowel_weight = 1.0 - (0.25 * vowel_position.clamp(-1.0, 1.0).abs());

    BODY_LAYER_MIX * vowel_weight * sin(2.0 * PI * wrap_phase(phase))
}

/// Computes the upper-mid sparkle layer frequency in hertz.
pub fn upper_mid_sparkle_frequency_hz(warble_depth: f64, contour: f64) -> f64 {
    let warble_amount = (warble_depth.clamp(-1.0, 1.0) + 1.0) * 0.5;

    (3_150.0 + (650.0 * warble_amount) + (280.0 * contour.clamp(-1.0, 1.0))).clamp(2_000.0, 5_000.0)
}

/// Computes the deterministic upper-mid sparkle layer sample.
pub fn upper_mid_sparkle_sample(phase: f64, elapsed_seconds: f64, warble_depth: f64) -> f64 {
    let warble_amount = (warble_depth.clamp(-1.0, 1.0) + 1.0) * 0.5;
    let amount = 0.35 + (0.65 * warble_amount);
    let gesture = 0.75 + (0.25 * sin(2.0 * PI * 19.0 * elapsed_seconds));
    let sparkle = UPPER_MID_SPARKLE_MIX * amount * gesture * sin(2.0 * PI * wrap_phase(phase));

    sparkle.clamp(-UPPER_MID_SPARKLE_MIX, UPPER_MID_SPARKLE_MIX)
}

/// Computes the deterministic per-syllable vowel trajectory.
pub fn vowel_trajectory_position(vowel_position: f64, contour: f64, elapsed_seconds: f64) -> f64 {
    vowel_trajectory_position_for_duration(
        vowel_position,
        contour,
        elapsed_seconds,
        BASE_SYLLABLE_SECONDS,
    )
}

fn vowel_trajectory_position_for_duration(
    vowel_position: f64,
    contour: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    let progress = syllable_progress(elapsed_seconds, duration_seconds);
    let sweep = contour.clamp(-1.0, 1.0) * VOWEL_TRAJECTORY_SWEEP * ((2.0 * progress) - 1.0);
    let bloom = VOWEL_TRAJECTORY_BLOOM * sin(PI * progress);

    (vowel_position + sweep + bloom).clamp(-1.0, 1.0)
}

/// Renders one fixed-duration syllable from semantic knobs.
pub fn render_syllable(knobs: crate::KnobSet, start_pitch_hz: f64) -> Vec<f64> {
    render_syllable_with_final_glide(knobs, start_pitch_hz, SyllableFinalGlide::Neutral, 0.0)
}

impl SyllablePerformance {
    pub(crate) fn new(
        duration_samples: u32,
        pitch_offset_semitones: f64,
        final_lowering_semitones: f64,
        emphasized: bool,
        mood_valence: f64,
        mood_arousal: f64,
    ) -> Self {
        Self {
            duration_samples,
            pitch_offset_semitones,
            final_lowering_semitones,
            emphasized,
            mood_valence: mood_valence.clamp(-1.0, 1.0),
            mood_arousal: mood_arousal.clamp(0.0, 1.0),
        }
    }

    fn default_v1() -> Self {
        Self {
            duration_samples: BASE_SYLLABLE_SAMPLES,
            pitch_offset_semitones: 0.0,
            final_lowering_semitones: 0.0,
            emphasized: false,
            mood_valence: 0.0,
            mood_arousal: 0.0,
        }
    }
}

pub(crate) fn render_syllable_with_final_glide(
    knobs: crate::KnobSet,
    start_pitch_hz: f64,
    final_glide: SyllableFinalGlide,
    warble_phase_offset: f64,
) -> Vec<f64> {
    render_syllable_with_performance(
        knobs,
        start_pitch_hz,
        final_glide,
        warble_phase_offset,
        SyllablePerformance::default_v1(),
    )
}

pub(crate) fn render_syllable_with_performance(
    knobs: crate::KnobSet,
    start_pitch_hz: f64,
    final_glide: SyllableFinalGlide,
    warble_phase_offset: f64,
    performance: SyllablePerformance,
) -> Vec<f64> {
    let duration_samples = performance.duration_samples.max(1);
    let duration_seconds = f64::from(duration_samples) / f64::from(SYNTH_SAMPLE_RATE_HZ);
    let contour = (knobs.contour() + (performance.mood_valence * 0.35)).clamp(-1.0, 1.0);
    let vowel_bias = (performance.mood_valence * 0.16) + (performance.mood_arousal * 0.10);
    let warble_depth = (knobs.warble_depth() + (performance.mood_arousal * 0.35)).clamp(-1.0, 1.0);
    let brightness_gain =
        (1.0 + (performance.mood_arousal * 0.45) + (performance.mood_valence * 0.12))
            .clamp(0.78, 1.55);
    let subgesture_density = 1.0 + (performance.mood_arousal * 0.80);
    let pitch_offset_semitones = performance.pitch_offset_semitones
        + if performance.emphasized {
            PHRASE_EMPHASIS_PITCH_SEMITONES
        } else {
            0.0
        };
    let target_pitch_hz =
        pitch_center_hz(knobs.pitch_center()) * semitone_multiplier(pitch_offset_semitones);
    let start_pitch_hz = if start_pitch_hz.is_finite() && start_pitch_hz > 0.0 {
        start_pitch_hz
    } else {
        target_pitch_hz
    };
    let mut phase = 0.0;
    let mut body_phase = 0.0;
    let mut sparkle_phase = 0.0;
    let mut formants = FormantFilterBank::new();
    let mut samples = Vec::new();

    for sample_index in 0..duration_samples {
        let elapsed_seconds = f64::from(sample_index) / f64::from(SYNTH_SAMPLE_RATE_HZ);
        let glided_pitch_hz =
            portamento_pitch_hz(start_pitch_hz, target_pitch_hz, contour, elapsed_seconds);
        let final_glide_pitch_hz = apply_final_glide_hz(
            glided_pitch_hz,
            target_pitch_hz,
            final_glide,
            elapsed_seconds,
            duration_seconds,
        );
        let phrase_final_pitch_hz = apply_phrase_final_shift_hz(
            final_glide_pitch_hz,
            target_pitch_hz,
            performance.final_lowering_semitones,
            elapsed_seconds,
            duration_seconds,
        );
        let internal_pitch_hz = apply_internal_pitch_swoop_hz_for_duration(
            phrase_final_pitch_hz,
            contour,
            elapsed_seconds,
            duration_seconds,
        );
        let pitch_hz = apply_warble_hz_with_phase(
            internal_pitch_hz,
            warble_depth,
            elapsed_seconds,
            warble_phase_offset,
        );
        let source = source_oscillator_sample(phase, pitch_hz);
        let vowel_position = vowel_trajectory_position_for_duration(
            (knobs.vowel_position() + vowel_bias).clamp(-1.0, 1.0),
            contour,
            elapsed_seconds,
            duration_seconds,
        );
        let voiced = formants.process_sample(source, vowel_position);
        let layered = voiced
            + body_layer_sample(body_phase, vowel_position)
            + attack_transient_sample(elapsed_seconds, contour)
            + (brightness_gain
                * upper_mid_sparkle_sample(
                    sparkle_phase,
                    elapsed_seconds * subgesture_density,
                    warble_depth,
                ));
        let layered = if performance.emphasized {
            layered * PHRASE_EMPHASIS_GAIN
        } else {
            layered
        };
        let electronic = ring_modulate(layered, elapsed_seconds);

        samples.push(apply_amplitude_envelope(
            electronic,
            elapsed_seconds,
            duration_seconds,
        ));

        phase = wrap_phase(phase + (pitch_hz / f64::from(SYNTH_SAMPLE_RATE_HZ)));
        body_phase = wrap_phase(
            body_phase + (body_layer_frequency_hz(pitch_hz) / f64::from(SYNTH_SAMPLE_RATE_HZ)),
        );
        sparkle_phase = wrap_phase(
            sparkle_phase
                + (upper_mid_sparkle_frequency_hz(warble_depth, contour)
                    / f64::from(SYNTH_SAMPLE_RATE_HZ)),
        );
    }

    samples
}

fn apply_internal_pitch_swoop_hz_for_duration(
    pitch_hz: f64,
    contour: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    let cents =
        internal_pitch_offset_cents_for_duration(contour, elapsed_seconds, duration_seconds);

    pitch_hz * exp(LN_2 * (cents / 1_200.0))
}

fn apply_final_glide_hz(
    pitch_hz: f64,
    target_pitch_hz: f64,
    final_glide: SyllableFinalGlide,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    let semitones = match final_glide {
        SyllableFinalGlide::Neutral => return pitch_hz,
        SyllableFinalGlide::Rising => PUNCTUATION_GLIDE_SEMITONES,
        SyllableFinalGlide::Falling => -PUNCTUATION_GLIDE_SEMITONES,
    };
    let final_glide_start_seconds = duration_seconds - PORTAMENTO_SECONDS;

    if elapsed_seconds < final_glide_start_seconds {
        return pitch_hz;
    }

    let progress = portamento_progress(elapsed_seconds - final_glide_start_seconds, semitones);
    let final_target_hz = target_pitch_hz * exp((LN_2 * semitones) / 12.0);

    pitch_hz + ((final_target_hz - pitch_hz) * progress)
}

fn apply_phrase_final_shift_hz(
    pitch_hz: f64,
    target_pitch_hz: f64,
    final_lowering_semitones: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    if final_lowering_semitones == 0.0 {
        return pitch_hz;
    }

    let final_shift_start_seconds = duration_seconds - PORTAMENTO_SECONDS;

    if elapsed_seconds < final_shift_start_seconds {
        return pitch_hz;
    }

    let progress = portamento_progress(
        elapsed_seconds - final_shift_start_seconds,
        final_lowering_semitones,
    );
    let final_target_hz = target_pitch_hz * semitone_multiplier(final_lowering_semitones);

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

fn triangle_window(elapsed_seconds: f64, center_seconds: f64, half_width_seconds: f64) -> f64 {
    if half_width_seconds <= 0.0 {
        return 0.0;
    }

    let distance = (elapsed_seconds - center_seconds).abs();

    (1.0 - (distance / half_width_seconds)).clamp(0.0, 1.0)
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

fn semitone_multiplier(semitones: f64) -> f64 {
    exp((LN_2 * semitones) / 12.0)
}

fn syllable_progress(elapsed_seconds: f64, duration_seconds: f64) -> f64 {
    if !elapsed_seconds.is_finite() || !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return 0.0;
    }

    (elapsed_seconds / duration_seconds).clamp(0.0, 1.0)
}
