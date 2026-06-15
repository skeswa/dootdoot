//! Formant synthesis voice engine for droid syllables.

use core::f64::consts::{LN_2, PI};

use crate::{ArchetypeSelection, GestureArchetype, cos, exp, sin};

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
pub const ENVELOPE_SUSTAIN_LEVEL: f64 = 0.34;

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

const TRANSITION_BRIDGE_GAIN: f64 = 0.180;
const CONNECTED_PARAMETER_BLEND_SECONDS: f64 = 0.036;

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
    complexity: f64,
    archetype: ArchetypeSelection,
    starts_connected: bool,
    ends_connected: bool,
}

#[derive(Debug, Clone, Copy)]
struct SyllableRenderControls {
    knobs: crate::KnobSet,
    start_pitch_hz: f64,
    target_pitch_hz: f64,
    duration_seconds: f64,
    contour: f64,
    vowel_bias: f64,
    warble_depth: f64,
    brightness_gain: f64,
    subgesture_density: f64,
    subgesture_count: u32,
    final_glide: SyllableFinalGlide,
    warble_phase_offset: f64,
    performance: SyllablePerformance,
}

#[derive(Debug, Clone, Copy)]
struct PerformanceSampleParameters {
    pitch_hz: f64,
    vowel_position: f64,
    contour: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct SyllableRenderState {
    phase: f64,
    body_phase: f64,
    sparkle_phase: f64,
    last_pitch_hz: Option<f64>,
    last_vowel_position: Option<f64>,
    formants: FormantFilterBank,
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

    let minimum_body = ENVELOPE_SUSTAIN_LEVEL * 0.70;

    (body + pulse - dip).clamp(minimum_body, 1.0)
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
    ) -> Self {
        Self {
            duration_samples,
            pitch_offset_semitones,
            final_lowering_semitones,
            emphasized,
            mood_valence: 0.0,
            mood_arousal: 0.0,
            complexity: 0.0,
            archetype: ArchetypeSelection::chatter(0),
            starts_connected: false,
            ends_connected: false,
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
            complexity: 0.0,
            archetype: ArchetypeSelection::chatter(0),
            starts_connected: false,
            ends_connected: false,
        }
    }

    fn pitch_offset_with_emphasis(self) -> f64 {
        self.pitch_offset_semitones
            + if self.emphasized {
                PHRASE_EMPHASIS_PITCH_SEMITONES
            } else {
                0.0
            }
    }

    pub(crate) fn with_expression(
        mut self,
        mood_valence: f64,
        mood_arousal: f64,
        complexity: f64,
        archetype: ArchetypeSelection,
    ) -> Self {
        self.mood_valence = mood_valence.clamp(-1.0, 1.0);
        self.mood_arousal = mood_arousal.clamp(0.0, 1.0);
        self.complexity = complexity.clamp(0.0, 1.0);
        self.archetype = archetype;

        self
    }

    pub(crate) fn with_connections(mut self, starts_connected: bool, ends_connected: bool) -> Self {
        self.starts_connected = starts_connected;
        self.ends_connected = ends_connected;

        self
    }
}

impl SyllableRenderControls {
    fn new(
        knobs: crate::KnobSet,
        start_pitch_hz: f64,
        final_glide: SyllableFinalGlide,
        warble_phase_offset: f64,
        performance: SyllablePerformance,
        duration_samples: u32,
    ) -> Self {
        let duration_seconds = f64::from(duration_samples) / f64::from(SYNTH_SAMPLE_RATE_HZ);
        let contour = (knobs.contour() + (performance.mood_valence * 0.35)).clamp(-1.0, 1.0);
        let vowel_bias = (performance.mood_valence * 0.16) + (performance.mood_arousal * 0.10);
        let warble_depth =
            (knobs.warble_depth() + (performance.mood_arousal * 0.35)).clamp(-1.0, 1.0);
        let brightness_gain =
            (1.0 + (performance.mood_arousal * 0.45) + (performance.mood_valence * 0.12))
                .clamp(0.78, 1.55);
        let subgesture_density =
            1.0 + (performance.mood_arousal * 0.80) + (performance.complexity * 1.20);
        let subgesture_count = complexity_subgesture_count(performance.complexity);
        let target_pitch_hz = pitch_center_hz(knobs.pitch_center())
            * semitone_multiplier(performance.pitch_offset_with_emphasis());
        let start_pitch_hz = if start_pitch_hz.is_finite() && start_pitch_hz > 0.0 {
            start_pitch_hz
        } else {
            target_pitch_hz
        };

        Self {
            knobs,
            start_pitch_hz,
            target_pitch_hz,
            duration_seconds,
            contour,
            vowel_bias,
            warble_depth,
            brightness_gain,
            subgesture_density,
            subgesture_count,
            final_glide,
            warble_phase_offset,
            performance,
        }
    }
}

impl SyllableRenderState {
    pub(crate) fn new() -> Self {
        Self {
            phase: 0.0,
            body_phase: 0.0,
            sparkle_phase: 0.0,
            last_pitch_hz: None,
            last_vowel_position: None,
            formants: FormantFilterBank::new(),
        }
    }

    fn advance(&mut self, pitch_hz: f64, vowel_position: f64, warble_depth: f64, contour: f64) {
        self.phase = wrap_phase(self.phase + (pitch_hz / f64::from(SYNTH_SAMPLE_RATE_HZ)));
        self.body_phase = wrap_phase(
            self.body_phase + (body_layer_frequency_hz(pitch_hz) / f64::from(SYNTH_SAMPLE_RATE_HZ)),
        );
        self.sparkle_phase = wrap_phase(
            self.sparkle_phase
                + (upper_mid_sparkle_frequency_hz(warble_depth, contour)
                    / f64::from(SYNTH_SAMPLE_RATE_HZ)),
        );
        self.last_pitch_hz = Some(pitch_hz);
        self.last_vowel_position = Some(vowel_position);
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
    let mut state = SyllableRenderState::new();
    let mut samples = Vec::new();

    render_syllable_with_performance_into(
        knobs,
        start_pitch_hz,
        final_glide,
        warble_phase_offset,
        performance,
        &mut state,
        &mut samples,
    );

    samples
}

pub(crate) fn render_syllable_with_performance_into(
    knobs: crate::KnobSet,
    start_pitch_hz: f64,
    final_glide: SyllableFinalGlide,
    warble_phase_offset: f64,
    performance: SyllablePerformance,
    state: &mut SyllableRenderState,
    samples: &mut Vec<f64>,
) {
    let duration_samples = performance.duration_samples.max(1);
    let controls = SyllableRenderControls::new(
        knobs,
        start_pitch_hz,
        final_glide,
        warble_phase_offset,
        performance,
        duration_samples,
    );

    for sample_index in 0..duration_samples {
        samples.push(render_performance_sample(sample_index, controls, state));
    }
}

pub(crate) fn render_transition_bridge(
    knobs: crate::KnobSet,
    start_pitch_hz: f64,
    target_pitch_hz: f64,
    duration_samples: u32,
    state: &mut SyllableRenderState,
    samples: &mut Vec<f64>,
) {
    if duration_samples == 0 {
        return;
    }

    let contour = knobs.contour().clamp(-1.0, 1.0);
    let vowel_position = knobs.vowel_position().clamp(-1.0, 1.0);
    let warble_depth = (knobs.warble_depth() * 0.55).clamp(-1.0, 1.0);

    for sample_index in 0..duration_samples {
        let elapsed_seconds = f64::from(sample_index) / f64::from(SYNTH_SAMPLE_RATE_HZ);
        let progress = bridge_progress(sample_index, duration_samples);
        let pitch_hz = start_pitch_hz + ((target_pitch_hz - start_pitch_hz) * progress);
        let pitch_hz = apply_warble_hz_with_phase(pitch_hz, warble_depth, elapsed_seconds, 0.25);
        let source = source_oscillator_sample(state.phase, pitch_hz);
        let voiced = state.formants.process_sample(source, vowel_position);
        let layer = voiced
            + (0.35 * source)
            + (0.35 * body_layer_sample(state.body_phase, vowel_position))
            + (0.25 * upper_mid_sparkle_sample(state.sparkle_phase, elapsed_seconds, warble_depth));
        let bridge_envelope = 0.35 + (0.65 * sin(PI * progress));
        let sample = ring_modulate(
            layer * TRANSITION_BRIDGE_GAIN * bridge_envelope,
            elapsed_seconds,
        );

        state.advance(pitch_hz, vowel_position, warble_depth, contour);
        samples.push(sample);
    }
}

fn render_performance_sample(
    sample_index: u32,
    controls: SyllableRenderControls,
    state: &mut SyllableRenderState,
) -> f64 {
    let elapsed_seconds = f64::from(sample_index) / f64::from(SYNTH_SAMPLE_RATE_HZ);
    let parameters = performance_sample_parameters(controls, state, elapsed_seconds);
    let source = source_oscillator_sample(state.phase, parameters.pitch_hz);
    let voiced = state
        .formants
        .process_sample(source, parameters.vowel_position);
    let attack_transient = if controls.performance.starts_connected {
        0.0
    } else {
        attack_transient_sample(elapsed_seconds, parameters.contour)
    };
    let layered = voiced
        + body_layer_sample(state.body_phase, parameters.vowel_position)
        + attack_transient
        + (controls.brightness_gain
            * upper_mid_sparkle_sample(
                state.sparkle_phase,
                elapsed_seconds * controls.subgesture_density,
                controls.warble_depth,
            ));
    let layered = layered
        + archetype_texture_sample(
            controls.performance.archetype,
            elapsed_seconds,
            controls.duration_seconds,
        );
    let layered =
        layered * archetype_amplitude_gain(controls.performance.archetype, elapsed_seconds);
    let layered = if controls.performance.emphasized {
        layered * PHRASE_EMPHASIS_GAIN
    } else {
        layered
    };
    let electronic = ring_modulate(layered, elapsed_seconds);

    state.advance(
        parameters.pitch_hz,
        parameters.vowel_position,
        controls.warble_depth,
        parameters.contour,
    );

    apply_connected_amplitude_envelope(
        electronic,
        elapsed_seconds,
        controls.duration_seconds,
        controls.performance.starts_connected,
        controls.performance.ends_connected,
    )
}

fn performance_sample_parameters(
    controls: SyllableRenderControls,
    state: &SyllableRenderState,
    elapsed_seconds: f64,
) -> PerformanceSampleParameters {
    let articulation = complexity_articulation_offset(
        controls.performance.complexity,
        controls.subgesture_count,
        elapsed_seconds,
        controls.duration_seconds,
    );
    let contour = (controls.contour + articulation).clamp(-1.0, 1.0);
    let glided_pitch_hz = portamento_pitch_hz(
        controls.start_pitch_hz,
        controls.target_pitch_hz,
        contour,
        elapsed_seconds,
    );
    let final_glide_pitch_hz = apply_final_glide_hz(
        glided_pitch_hz,
        controls.target_pitch_hz,
        controls.final_glide,
        elapsed_seconds,
        controls.duration_seconds,
    );
    let phrase_final_pitch_hz = apply_phrase_final_shift_hz(
        final_glide_pitch_hz,
        controls.target_pitch_hz,
        controls.performance.final_lowering_semitones,
        elapsed_seconds,
        controls.duration_seconds,
    );
    let internal_pitch_hz = apply_internal_pitch_swoop_hz_for_duration(
        phrase_final_pitch_hz,
        contour,
        elapsed_seconds,
        controls.duration_seconds,
    );
    let pitch_hz = apply_warble_hz_with_phase(
        internal_pitch_hz,
        controls.warble_depth,
        elapsed_seconds,
        controls.warble_phase_offset,
    );
    let pitch_hz = apply_archetype_pitch_hz(
        pitch_hz,
        controls.performance.archetype.archetype(),
        elapsed_seconds,
        controls.duration_seconds,
    );
    let vowel_position = vowel_trajectory_position_for_duration(
        (controls.knobs.vowel_position() + controls.vowel_bias + (articulation * 0.65))
            .clamp(-1.0, 1.0),
        contour,
        elapsed_seconds,
        controls.duration_seconds,
    );

    PerformanceSampleParameters {
        pitch_hz: connected_parameter_value(
            pitch_hz,
            state.last_pitch_hz,
            elapsed_seconds,
            controls.performance.starts_connected,
        ),
        vowel_position: connected_parameter_value(
            vowel_position,
            state.last_vowel_position,
            elapsed_seconds,
            controls.performance.starts_connected,
        ),
        contour,
    }
}

fn connected_parameter_value(
    current: f64,
    previous: Option<f64>,
    elapsed_seconds: f64,
    starts_connected: bool,
) -> f64 {
    let Some(previous) = previous else {
        return current;
    };

    if !starts_connected
        || !elapsed_seconds.is_finite()
        || elapsed_seconds >= CONNECTED_PARAMETER_BLEND_SECONDS
    {
        return current;
    }

    let linear = (elapsed_seconds / CONNECTED_PARAMETER_BLEND_SECONDS).clamp(0.0, 1.0);
    let progress = linear * linear * (3.0 - (2.0 * linear));

    previous + ((current - previous) * progress)
}

fn apply_connected_amplitude_envelope(
    sample: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
    starts_connected: bool,
    ends_connected: bool,
) -> f64 {
    sample
        * connected_amplitude_envelope(
            elapsed_seconds,
            duration_seconds,
            starts_connected,
            ends_connected,
        )
}

fn connected_amplitude_envelope(
    elapsed_seconds: f64,
    duration_seconds: f64,
    starts_connected: bool,
    ends_connected: bool,
) -> f64 {
    let base = amplitude_envelope(elapsed_seconds, duration_seconds);
    let connection_floor = ENVELOPE_SUSTAIN_LEVEL * 0.95;
    let attack_connection_end = ENVELOPE_ATTACK_SECONDS + 0.018;
    let release_start = (duration_seconds - ENVELOPE_RELEASE_SECONDS)
        .max(ENVELOPE_ATTACK_SECONDS + ENVELOPE_DECAY_SECONDS);

    if starts_connected && elapsed_seconds <= attack_connection_end {
        let target = amplitude_envelope(attack_connection_end, duration_seconds);
        let progress = smoothstep(elapsed_seconds / attack_connection_end);

        return connection_floor + ((target - connection_floor) * progress);
    }

    if ends_connected && elapsed_seconds >= release_start {
        return base.max(connection_floor);
    }

    base
}

fn apply_archetype_pitch_hz(
    pitch_hz: f64,
    archetype: GestureArchetype,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    let progress = syllable_progress(elapsed_seconds, duration_seconds);
    let semitones = match archetype {
        GestureArchetype::Chatter => 0.0,
        GestureArchetype::Yelp => 0.65 * progress,
        GestureArchetype::Moan => -0.55 * (1.0 - (0.35 * progress)),
        GestureArchetype::StutterBurst => 0.28 * stutter_pulse(progress),
        GestureArchetype::Tremble => 0.18 * sin(2.0 * PI * 29.0 * elapsed_seconds),
    };

    pitch_hz * semitone_multiplier(semitones)
}

fn archetype_amplitude_gain(selection: ArchetypeSelection, elapsed_seconds: f64) -> f64 {
    match selection.archetype() {
        GestureArchetype::Chatter => 1.0,
        GestureArchetype::Yelp => 1.08,
        GestureArchetype::Moan => 0.92,
        GestureArchetype::StutterBurst => {
            let pulse = 0.82 + (0.28 * stutter_pulse(elapsed_seconds * 9.0));

            pulse.clamp(0.74, 1.18)
        }
        GestureArchetype::Tremble => {
            let tremolo = 1.0 + (0.08 * sin(2.0 * PI * 23.0 * elapsed_seconds));

            tremolo.clamp(0.88, 1.12)
        }
    }
}

fn archetype_texture_sample(
    selection: ArchetypeSelection,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    let mut texture = match selection.archetype() {
        GestureArchetype::Chatter => 0.0,
        GestureArchetype::Yelp => yelp_texture(elapsed_seconds),
        GestureArchetype::Moan => moan_texture(elapsed_seconds),
        GestureArchetype::StutterBurst => stutter_texture(elapsed_seconds, duration_seconds),
        GestureArchetype::Tremble => tremble_texture(elapsed_seconds),
    };

    if selection.servo_seasoning() {
        texture += servo_texture(selection.syllable_index(), elapsed_seconds);
    }

    if selection.noise_tail() {
        texture += noise_tail_texture(
            selection.syllable_index(),
            elapsed_seconds,
            duration_seconds,
        );
    }

    texture.clamp(-0.16, 0.16)
}

fn yelp_texture(elapsed_seconds: f64) -> f64 {
    0.030
        * triangle_window(elapsed_seconds, 0.018, 0.014)
        * sin(2.0 * PI * 3_880.0 * elapsed_seconds)
}

fn moan_texture(elapsed_seconds: f64) -> f64 {
    0.026 * sin(2.0 * PI * 410.0 * elapsed_seconds)
}

fn stutter_texture(elapsed_seconds: f64, duration_seconds: f64) -> f64 {
    let progress = syllable_progress(elapsed_seconds, duration_seconds);
    let pulse = stutter_pulse(progress);

    0.045 * pulse * sin(2.0 * PI * 2_940.0 * elapsed_seconds)
}

fn tremble_texture(elapsed_seconds: f64) -> f64 {
    0.024 * sin(2.0 * PI * 31.0 * elapsed_seconds) * sin(2.0 * PI * 3_520.0 * elapsed_seconds)
}

fn servo_texture(syllable_index: usize, elapsed_seconds: f64) -> f64 {
    let index = u32::try_from(syllable_index).unwrap_or(u32::MAX);
    let frequency_hz = 980.0 + (37.0 * f64::from(index % 7));

    0.024
        * triangle_window(elapsed_seconds, 0.032, 0.020)
        * sin(2.0 * PI * frequency_hz * elapsed_seconds)
}

fn noise_tail_texture(syllable_index: usize, elapsed_seconds: f64, duration_seconds: f64) -> f64 {
    let index = u32::try_from(syllable_index).unwrap_or(u32::MAX);
    let release_start = (duration_seconds - ENVELOPE_RELEASE_SECONDS).max(0.0);

    if elapsed_seconds < release_start {
        return 0.0;
    }

    let progress = ((elapsed_seconds - release_start) / ENVELOPE_RELEASE_SECONDS).clamp(0.0, 1.0);
    let envelope = (1.0 - progress) * (1.0 - progress);
    let modulator = sin(2.0 * PI * (173.0 + f64::from(index % 5)) * elapsed_seconds);
    let carrier = sin((2.0 * PI * 4_650.0 * elapsed_seconds) + (0.65 * modulator));

    0.018 * envelope * carrier
}

fn stutter_pulse(progress: f64) -> f64 {
    let phase = (progress * 4.0) % 1.0;

    triangle_window(phase, 0.18, 0.18)
}

fn complexity_subgesture_count(complexity: f64) -> u32 {
    if complexity <= 0.0 {
        1
    } else if complexity < 0.34 {
        2
    } else if complexity < 0.67 {
        3
    } else {
        4
    }
}

fn complexity_articulation_offset(
    complexity: f64,
    subgesture_count: u32,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    if complexity <= 0.0 || !elapsed_seconds.is_finite() {
        return 0.0;
    }

    let progress = syllable_progress(elapsed_seconds, duration_seconds);
    let phase = f64::from(subgesture_count) * progress;
    let primary = sin(2.0 * PI * phase);
    let secondary = sin(2.0 * PI * ((phase * 2.0) + 0.17));

    (complexity.clamp(0.0, 1.0) * 0.105 * ((0.72 * primary) + (0.28 * secondary)))
        .clamp(-0.14, 0.14)
}

fn bridge_progress(sample_index: u32, duration_samples: u32) -> f64 {
    if duration_samples <= 1 {
        return 1.0;
    }

    let denominator = f64::from(duration_samples - 1);
    let linear = (f64::from(sample_index) / denominator).clamp(0.0, 1.0);

    linear * linear * (3.0 - (2.0 * linear))
}

fn smoothstep(progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);

    progress * progress * (3.0 - (2.0 * progress))
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
