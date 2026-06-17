//! Formant synthesis voice engine for droid syllables.

use core::f64::consts::{LN_2, PI};

use crate::{
    ArchetypeSelection, GestureArchetype, PerformanceCurves, PhraseRole, TailShape, cos, exp, sin,
    tanh,
};

/// Gives the maximum `VOICE_V7` curve-driven pitch-center bias in semitones.
pub const CURVE_PITCH_BIAS_SEMITONES: f64 = 4.0;

const CURVE_BRIGHTNESS_GAIN: f64 = 0.40;
/// Gives the syllable progress at which a whistle sweep begins.
///
/// `VOICE_V10` starts the sweep earlier than the `VOICE_V7` 0.45 so the swept
/// pitch dwells in the whistle band for more of the gesture (the taxonomy gap
/// was that the whistle reached high too briefly to carry the dominant peak).
const CURVE_WHISTLE_START_FRACTION: f64 = 0.30;

/// Gives the archetype-tension above which a non-flourish body syllable (chatty
/// reply / probe) engages the whistle sweep. The planner drives tension this
/// high only on the one promoted semantic accent (which reaches ~0.80+), while
/// non-accent body syllables top out around 0.75, so this gate cleanly isolates
/// the accent and prevents a shrill every-syllable whistle.
const WHISTLE_ACCENT_TENSION_THRESHOLD: f64 = 0.76;

/// Gives the whistle amount a body accent sweeps with the instant it engages.
///
/// `VOICE_V10`: the `VOICE_V8` ramp started from zero at the gate, so a
/// just-engaged accent barely left the register and the dominant peak never
/// rode high. The engaged sweep now starts from this floor and ramps to
/// [`WHISTLE_ACCENT_SCALE`].
const WHISTLE_ACCENT_FLOOR: f64 = 0.55;

/// Gives the `VOICE_V8` whistle scale applied to body-syllable accents, keeping
/// them slightly under a dedicated terminal flourish.
const WHISTLE_ACCENT_SCALE: f64 = 0.85;

/// Gives the `VOICE_V8` always-on roughness floor for engaged (planner-driven)
/// body syllables, so neutral text is not pinned to a pure periodic tone. It is
/// gated on non-neutral curves, so hand-built events and the empty chirp stay
/// byte-identical (clean) under neutral curves.
const ROUGHNESS_BODY_FLOOR: f64 = 0.18;

/// Gives the utterance arousal above which a negative-valence accent can break
/// into a `VOICE_V10` rough/noisy burst.
const AGITATION_AROUSAL_THRESHOLD: f64 = 0.5;

/// Gives how far a fully agitated accent pushes its roughness toward a pure
/// noise burst (1.0) above its base. The taxonomy found BB-8 squawks rough on
/// agitation while dootdoot never left the tonal band; a fully agitated accent
/// now crosses into the noisy band, then recovers when the gesture ends.
const AGITATION_ROUGHNESS_GAIN: f64 = 0.95;

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

/// Gives the `VOICE_V7` wider per-gesture pitch span in semitones.
///
/// Selected gestures use this span so they can leave the established
/// ~0.5-1.1 kHz register, while ordinary syllables keep `PITCH_SEMITONE_SPAN`.
pub const WIDE_GESTURE_PITCH_SPAN_SEMITONES: f64 = 16.0;

/// Gives the `VOICE_V10` accent per-gesture pitch span in semitones.
///
/// The one promoted semantic accent per phrase swoops widest so a single
/// gesture approaches BB-8's multi-octave excursions (the taxonomy found
/// dootdoot capped near one octave where BB-8 routinely spans three to four).
/// It stays bounded inside the droid register and above
/// `WIDE_GESTURE_PITCH_SPAN_SEMITONES`.
pub const ACCENT_PITCH_SPAN_SEMITONES: f64 = 26.0;

/// Gives the nominal top of a `VOICE_V7` whistle sweep in hertz.
pub const WHISTLE_TARGET_HZ: f64 = 3_400.0;

/// Gives the hard ceiling for a `VOICE_V7` whistle sweep in hertz.
pub const WHISTLE_PITCH_CEILING_HZ: f64 = 4_200.0;

/// Gives the nominal bottom of a `VOICE_V10` *descending* whistle sweep in
/// hertz.
///
/// The taxonomy comparison found BB-8 falls as readily as it rises, but
/// dootdoot's whistle only ever climbed. A negative sweep amount now glides the
/// oscillator fundamental down toward this floor (the downward analogue of
/// `WHISTLE_TARGET_HZ`), kept well inside the bounded droid register so a
/// statement flourish lands with a droid descent rather than a sub-bass dive.
pub const WHISTLE_FLOOR_HZ: f64 = 300.0;

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

/// Gives the question's terminal rise span in semitones (`VOICE_V9`, R5). It is
/// wider than the generic punctuation glide so the inquisitive lift (`H-H%`) is
/// unmistakable even on a short final word.
pub const QUESTION_RISE_SEMITONES: f64 = 4.5;

/// Gives the depth of the question's pre-final dip (`L*`) in semitones, applied
/// mid-glide before the rise so the lift reads as a gather-then-rise.
const QUESTION_PREFINAL_DIP_SEMITONES: f64 = 1.2;

/// Gives the shallow continuation-rise span for clause punctuation in
/// semitones (`VOICE_V9`). A clause mark lifts the tail just enough to read as
/// "more coming" without reaching the full question rise.
const CONTINUATION_GLIDE_SEMITONES: f64 = 1.5;

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

const TRANSITION_BRIDGE_GAIN: f64 = 0.260;
const TRANSITION_BRIDGE_ENVELOPE_FLOOR: f64 = 0.30;
const TRANSITION_BRIDGE_ENVELOPE_SPAN: f64 = 0.08;
const CONNECTED_PARAMETER_BLEND_SECONDS: f64 = 0.036;
const WORD_CONNECTED_PARAMETER_BLEND_SECONDS: f64 = 0.120;
const WORD_CONNECTION_FLOOR: f64 = ENVELOPE_SUSTAIN_LEVEL * 0.30;
const WORD_CONNECTION_OPEN_SECONDS: f64 = 0.055;
const WORD_ONSET_VOWEL_OPEN_SECONDS: f64 = 0.060;
const WORD_ONSET_VOWEL_ROUNDING: f64 = 0.55;
const WORD_ONSET_TEXTURE_START_GAIN: f64 = 0.42;
const WORD_CONNECTED_MOTION_START_GAIN: f64 = 0.30;
const WORD_CONNECTED_MOTION_END_GAIN: f64 = 0.52;
const WORD_CONNECTED_TEXTURE_END_GAIN: f64 = 0.68;
const WORD_CONNECTED_ENVELOPE_LEVEL: f64 = ENVELOPE_SUSTAIN_LEVEL * 1.05;
const WORD_CONNECTED_ENVELOPE_CONTRAST: f64 = 0.16;

/// Marks the synthesis module in the public facade.
#[derive(Debug)]
pub struct Synth;

/// Gives the prosodic final-glide shape applied to one syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyllableFinalGlide {
    Neutral,
    Rising,
    Falling,
    /// A shallow continuation rise for clause punctuation (`VOICE_V9`).
    Continuation,
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
    start_connection: SyllableConnection,
    end_connection: SyllableConnection,
    role: PhraseRole,
    curves: PerformanceCurves,
    tail_shape: TailShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyllableConnection {
    /// Starts or ends against silence or a punctuation reset.
    Detached,
    /// Starts or ends against another subword in the same word.
    Subword,
    /// Starts or ends across a bridged word boundary.
    Word,
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
    whistle_amount: f64,
    roughness_amount: f64,
    mouth_openness: f64,
    mouth_front_back: f64,
    tail_shape: TailShape,
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
    mouth: MouthStage,
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
    pitch_center_hz_with_span(pitch_center, PITCH_SEMITONE_SPAN)
}

/// Maps a pitch-center knob to hertz using a chosen semitone span.
///
/// `VOICE_V7` selected gestures pass `WIDE_GESTURE_PITCH_SPAN_SEMITONES` so
/// they can climb above the default register; passing `PITCH_SEMITONE_SPAN`
/// reproduces `pitch_center_hz` exactly.
pub fn pitch_center_hz_with_span(pitch_center: f64, span_semitones: f64) -> f64 {
    let span = if span_semitones.is_finite() {
        span_semitones.abs()
    } else {
        PITCH_SEMITONE_SPAN
    };
    let knob = if pitch_center.is_finite() {
        pitch_center.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let semitones = knob * span;

    PITCH_REGISTER_BIAS_HZ * exp((LN_2 * semitones) / 12.0)
}

/// Sweeps the oscillator fundamental into the whistle band for a chirp gesture.
///
/// The sweep is *signed*: a positive `whistle_amount` rises toward
/// `WHISTLE_PITCH_CEILING_HZ` (the `VOICE_V7` climb), a negative amount
/// descends toward `WHISTLE_FLOOR_HZ` (the `VOICE_V10` fall), and `0` is a
/// no-op. At `progress == 0` the result is always `start_hz`; the magnitude
/// scales how far it travels by `progress == 1`. The sweep is pure IEEE
/// arithmetic (no transcendentals), so it is bit-exact across the verified
/// platforms, and is always finite, positive, and inside
/// `[1, WHISTLE_PITCH_CEILING_HZ]`. The positive path is byte-identical to
/// `VOICE_V7`–`V9`.
pub fn whistle_sweep_pitch_hz(start_hz: f64, whistle_amount: f64, progress: f64) -> f64 {
    let start = if start_hz.is_finite() && start_hz > 0.0 {
        start_hz
    } else {
        PITCH_REGISTER_BIAS_HZ
    };
    let signed = if whistle_amount.is_finite() {
        whistle_amount.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let amount = signed.abs();

    if amount <= 0.0 {
        return start.min(WHISTLE_PITCH_CEILING_HZ);
    }

    let progress = if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let shape = progress * progress * (3.0 - (2.0 * progress));
    let target = if signed >= 0.0 {
        (WHISTLE_TARGET_HZ + ((WHISTLE_PITCH_CEILING_HZ - WHISTLE_TARGET_HZ) * amount))
            .min(WHISTLE_PITCH_CEILING_HZ)
    } else {
        WHISTLE_FLOOR_HZ
    };
    let swept = start + ((target - start) * amount * shape);

    swept.clamp(1.0, WHISTLE_PITCH_CEILING_HZ)
}

/// Gives the signed whistle-sweep amount for a planned syllable.
///
/// Positive rises, negative descends (see [`whistle_sweep_pitch_hz`]), `0` is a
/// no-op. A terminal flourish sweeps by its full archetype tension; a body
/// accent (`ChattyReply`/`Probe`) engages only once tension clears
/// `WHISTLE_ACCENT_TENSION_THRESHOLD` — which the planner reaches only on the
/// one promoted accent, never on non-accent body syllables — then sweeps from
/// `WHISTLE_ACCENT_FLOOR` up to `WHISTLE_ACCENT_SCALE`. The sign follows
/// `pitch_velocity` so the exclamation flourish descends (`VOICE_V10`).
pub fn whistle_sweep_amount(role: PhraseRole, archetype_tension: f64, pitch_velocity: f64) -> f64 {
    let tension = if archetype_tension.is_finite() {
        archetype_tension.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let magnitude = match role {
        PhraseRole::TerminalFlourish => tension,
        // VOICE_V8: a semantic accent in the body of an utterance (the planner
        // drives tension past the threshold only on accents) reaches the whistle
        // band even without terminal punctuation.
        PhraseRole::ChattyReply | PhraseRole::Probe
            if tension >= WHISTLE_ACCENT_TENSION_THRESHOLD =>
        {
            let above = ((tension - WHISTLE_ACCENT_TENSION_THRESHOLD)
                / (1.0 - WHISTLE_ACCENT_TENSION_THRESHOLD))
                .clamp(0.0, 1.0);

            (WHISTLE_ACCENT_FLOOR + ((WHISTLE_ACCENT_SCALE - WHISTLE_ACCENT_FLOOR) * above))
                .clamp(0.0, 1.0)
        }
        _ => 0.0,
    };

    // VOICE_V10: the whistle is signed. A negative pitch velocity (the
    // exclamation flourish, set by the planner) descends toward
    // `WHISTLE_FLOOR_HZ`; zero/positive keeps the `VOICE_V7`-`V9` rising sweep.
    if pitch_velocity < 0.0 {
        -magnitude
    } else {
        magnitude
    }
}

/// Gives the per-gesture pitch span in semitones for a planned syllable.
///
/// A syllable with no whistle keeps the default [`PITCH_SEMITONE_SPAN`]. A
/// whistling gesture widens so it can leave the established register: the
/// terminal flourish uses [`WIDE_GESTURE_PITCH_SPAN_SEMITONES`], and a body
/// semantic accent (`ChattyReply`/`Probe`) uses the wider
/// [`ACCENT_PITCH_SPAN_SEMITONES`] so its excursion approaches BB-8's
/// multi-octave swoops.
pub fn gesture_pitch_span_semitones(role: PhraseRole, whistle_amount: f64) -> f64 {
    match role {
        _ if whistle_amount == 0.0 => PITCH_SEMITONE_SPAN,
        PhraseRole::ChattyReply | PhraseRole::Probe => ACCENT_PITCH_SPAN_SEMITONES,
        _ => WIDE_GESTURE_PITCH_SPAN_SEMITONES,
    }
}

/// Gives the noise/breath roughness amount for a planned syllable.
///
/// The role base is the `VOICE_V8` blend (a small always-on floor for engaged
/// body syllables; clean for hand-built/neutral curves). `VOICE_V10` adds a
/// transient burst: a body accent (`ChattyReply`/`Probe` past the whistle gate)
/// in an agitated utterance — high arousal *and* negative valence — pushes its
/// roughness toward a noise burst so a single gesture crosses into the noisy
/// band, then recovers when the gesture ends. Non-accent and calm syllables
/// keep the base, so the steady-state texture is unchanged.
pub fn syllable_roughness_amount(
    role: PhraseRole,
    archetype_tension: f64,
    brightness_pressure: f64,
    mood_valence: f64,
    mood_arousal: f64,
) -> f64 {
    let tension = archetype_tension.clamp(0.0, 1.0);
    // VOICE_V8: only engaged (planner-driven) body syllables carry the small
    // always-on roughness floor, so neutral text swings off pure periodicity.
    // Neutral curves (hand-built events, empty chirp) keep a zero floor, and the
    // terminal flourish keeps its V7 floor-free roughness so a trailing "!" does
    // not pile breath noise onto an already-intense whistle gesture.
    let engaged = brightness_pressure > 0.0 || tension > 0.0;

    let base = match role {
        PhraseRole::Hesitation => (0.45 + (0.20 * tension)).clamp(0.0, 0.7),
        PhraseRole::Aside => (0.30 + (0.20 * tension)).clamp(0.0, 0.6),
        PhraseRole::ChattyReply | PhraseRole::Probe => {
            let floor = if engaged { ROUGHNESS_BODY_FLOOR } else { 0.0 };

            (floor + (0.12 * tension)).clamp(0.0, 0.4)
        }
        PhraseRole::TerminalFlourish => (0.12 * tension).clamp(0.0, 0.3),
    };

    // VOICE_V10 agitation burst: only on a body accent (the promoted semantic
    // accent reaches the whistle gate), and only when the utterance is agitated.
    let is_accent = matches!(role, PhraseRole::ChattyReply | PhraseRole::Probe)
        && tension >= WHISTLE_ACCENT_TENSION_THRESHOLD;
    if !is_accent {
        return base;
    }

    let arousal_drive = ((mood_arousal.clamp(0.0, 1.0) - AGITATION_AROUSAL_THRESHOLD)
        / (1.0 - AGITATION_AROUSAL_THRESHOLD))
        .clamp(0.0, 1.0);
    let agitation = arousal_drive * (-mood_valence).clamp(0.0, 1.0);

    (base + (AGITATION_ROUGHNESS_GAIN * agitation * (1.0 - base))).clamp(0.0, 1.0)
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

/// Computes the `VOICE_V7` event-based gain on the upper-mid sparkle layer.
///
/// At `brightness_pressure == 0` (neutral curves: the empty chirp and
/// hand-built events) the gain is exactly `1.0`, preserving the `VOICE_V6`
/// always-on sparkle. For brightness-driven engine syllables the sparkle
/// becomes an event: a shaped attack/decay scaled so flourishes reserve more
/// brightness than ordinary chatter.
pub fn sparkle_event_gain(
    brightness_pressure: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    let brightness = if brightness_pressure.is_finite() {
        brightness_pressure.clamp(0.0, 1.0)
    } else {
        0.0
    };

    if brightness <= 0.0 {
        return 1.0;
    }

    // VOICE_V8: a sharper envelope (sin^2) with a lower floor and a steeper
    // brightness slope turns the upper-mid sparkle from a constant bed into an
    // event. Ordinary low-brightness syllables stay dim; bright accents burst.
    let envelope = sin(PI * syllable_progress(elapsed_seconds, duration_seconds));
    let reserve = 0.08 + (1.5 * brightness);

    (envelope * envelope * reserve).clamp(0.0, 1.8)
}

/// Computes a bounded deterministic per-gesture tape-speed detune in cents.
///
/// At `tension == 0` (neutral curves) the detune is exactly `0.0`. For
/// expressive engine syllables it adds a small authored imperfection that
/// varies by syllable (via the warble phase offset), bounded to `±6` cents.
pub fn imperfection_detune_cents(tension: f64, warble_phase_offset: f64) -> f64 {
    let tension = if tension.is_finite() {
        tension.clamp(0.0, 1.0)
    } else {
        0.0
    };

    if tension <= 0.0 {
        return 0.0;
    }

    (6.0 * tension * sin(2.0 * PI * wrap_phase(warble_phase_offset))).clamp(-6.0, 6.0)
}

/// Computes the deterministic upper-mid sparkle layer sample.
pub fn upper_mid_sparkle_sample(phase: f64, elapsed_seconds: f64, warble_depth: f64) -> f64 {
    let warble_amount = (warble_depth.clamp(-1.0, 1.0) + 1.0) * 0.5;
    let amount = 0.35 + (0.65 * warble_amount);
    let gesture = 0.75 + (0.25 * sin(2.0 * PI * 19.0 * elapsed_seconds));
    let sparkle = UPPER_MID_SPARKLE_MIX * amount * gesture * sin(2.0 * PI * wrap_phase(phase));

    sparkle.clamp(-UPPER_MID_SPARKLE_MIX, UPPER_MID_SPARKLE_MIX)
}

/// Gives the maximum `VOICE_V7` noise/breath excitation blend mix.
pub const NOISE_BREATH_MAX_MIX: f64 = 0.5;

/// Gives the value-noise stride that bandlimits the breath excitation.
const NOISE_BREATH_STRIDE: u32 = 7;

/// Computes one deterministic value-noise breath sample, scaled by roughness.
///
/// The source is authored, not random: a fixed integer hash of the sample index
/// produces a reproducible value-noise curve in `[-1, 1]`. At `roughness_amount
/// == 0` the result is exactly `0.0`, so ordinary syllables stay cleanly
/// periodic; otherwise the magnitude is bounded by the (clamped) amount.
pub fn noise_breath_sample(sample_index: u32, roughness_amount: f64) -> f64 {
    let amount = if roughness_amount.is_finite() {
        roughness_amount.clamp(0.0, 1.0)
    } else {
        0.0
    };

    if amount <= 0.0 {
        return 0.0;
    }

    (noise_breath_raw(sample_index) * amount).clamp(-1.0, 1.0)
}

/// Blends a deterministic noise/breath source under a tonal sample.
///
/// `roughness_amount == 0` returns `tonal` unchanged (clean periodicity);
/// higher amounts cross-fade toward the breath source up to
/// `NOISE_BREATH_MAX_MIX`, so a gesture's harmonicity can swing clean→rough
/// without any runtime randomness.
pub fn blend_noise_excitation(tonal: f64, sample_index: u32, roughness_amount: f64) -> f64 {
    let amount = if roughness_amount.is_finite() {
        roughness_amount.clamp(0.0, 1.0)
    } else {
        0.0
    };

    if amount <= 0.0 {
        return tonal;
    }

    let mix = amount * NOISE_BREATH_MAX_MIX;

    (tonal * (1.0 - mix)) + (noise_breath_raw(sample_index) * mix)
}

fn noise_breath_raw(sample_index: u32) -> f64 {
    let base = sample_index / NOISE_BREATH_STRIDE;
    let step = sample_index % NOISE_BREATH_STRIDE;
    let fraction = f64::from(step) / f64::from(NOISE_BREATH_STRIDE);
    let low = hashed_unit(base);
    let high = hashed_unit(base.wrapping_add(1));
    let smooth = fraction * fraction * (3.0 - (2.0 * fraction));

    low + ((high - low) * smooth)
}

fn hashed_unit(index: u32) -> f64 {
    let hashed = mix_hash(index);

    ((f64::from(hashed) / f64::from(u32::MAX)) * 2.0) - 1.0
}

fn mix_hash(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;

    value
}

/// Gives the number of broad resonances in the `VOICE_V7` mouth stage.
pub const MOUTH_RESONANCE_COUNT: usize = 3;

/// Gives the maximum wet mix of the `VOICE_V7` code-talkbox mouth stage.
pub const MOUTH_STAGE_MAX_MIX: f64 = 0.45;

const MOUTH_RESONANCE_Q: f64 = 2.2;
const MOUTH_BREATH_MIX: f64 = 0.40;
const MOUTH_SATURATION_DRIVE: f64 = 1.6;

/// Gives the per-gesture drive of the `VOICE_V7` mouth stage.
///
/// `closed` leaves the stage off so it passes the formant-core signal through
/// untouched; the planner opens it on inquisitive holds, moans, and flourishes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouthDrive {
    openness: f64,
    front_back: f64,
    breath_amount: f64,
}

impl MouthDrive {
    /// Builds a mouth drive from openness, tongue front/back, and breath.
    pub fn new(openness: f64, front_back: f64, breath_amount: f64) -> Self {
        Self {
            openness,
            front_back,
            breath_amount,
        }
    }

    /// Builds the closed (off) mouth drive.
    pub fn closed() -> Self {
        Self {
            openness: 0.0,
            front_back: 0.0,
            breath_amount: 0.0,
        }
    }

    fn sanitized(self) -> Option<(f64, f64, f64)> {
        let openness = if self.openness.is_finite() {
            self.openness.clamp(0.0, 1.0)
        } else {
            0.0
        };

        if openness <= 0.0 {
            return None;
        }

        let front_back = if self.front_back.is_finite() {
            self.front_back.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let breath_amount = if self.breath_amount.is_finite() {
            self.breath_amount.clamp(0.0, 1.0)
        } else {
            0.0
        };

        Some((openness, front_back, breath_amount))
    }
}

/// A bounded broad mouth filter applied after the formant bank.
#[derive(Debug, Clone, Default)]
pub struct MouthStage {
    filters: [BandpassFilter; MOUTH_RESONANCE_COUNT],
}

impl MouthStage {
    /// Builds a silent (closed) mouth stage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Processes one sample through the gated mouth stage.
    ///
    /// A closed drive returns the input unchanged; an open drive blends in
    /// broad moving resonances, optional breath, and mild bounded
    /// saturation. The output is always finite and bounded.
    pub fn process_sample(&mut self, input: f64, sample_index: u32, drive: MouthDrive) -> f64 {
        let Some((openness, front_back, breath_amount)) = drive.sanitized() else {
            return input;
        };
        let centers = mouth_resonance_hz(openness, front_back);
        let mut wet = 0.0;

        for (filter, center_hz) in self.filters.iter_mut().zip(centers) {
            wet += filter.process_sample(input, center_hz, MOUTH_RESONANCE_Q);
        }

        wet /= f64_from_usize(MOUTH_RESONANCE_COUNT);
        wet += noise_breath_sample(sample_index, breath_amount) * MOUTH_BREATH_MIX;

        let driven = tanh(wet * MOUTH_SATURATION_DRIVE) / MOUTH_SATURATION_DRIVE;
        let mix = openness * MOUTH_STAGE_MAX_MIX;

        (input * (1.0 - mix)) + (driven * mix)
    }

    /// Clears the mouth filter state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Computes the deterministic per-gesture mouth open/close envelope.
pub fn mouth_open_envelope(elapsed_seconds: f64, duration_seconds: f64) -> f64 {
    let progress = syllable_progress(elapsed_seconds, duration_seconds);

    sin(PI * progress).clamp(0.0, 1.0)
}

/// Computes the broad moving mouth resonance centers in hertz.
pub fn mouth_resonance_hz(openness: f64, front_back: f64) -> [f64; MOUTH_RESONANCE_COUNT] {
    let openness = if openness.is_finite() {
        openness.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let front_back = if front_back.is_finite() {
        front_back.clamp(-1.0, 1.0)
    } else {
        0.0
    };

    [
        (420.0 + (380.0 * openness)).clamp(200.0, 3_600.0),
        (1_100.0 + (700.0 * front_back)).clamp(200.0, 3_600.0),
        (2_500.0 + (300.0 * front_back)).clamp(200.0, 3_600.0),
    ]
}

fn f64_from_usize(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
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
            start_connection: SyllableConnection::Detached,
            end_connection: SyllableConnection::Detached,
            role: PhraseRole::ChattyReply,
            curves: PerformanceCurves::neutral(),
            tail_shape: TailShape::Sustained,
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
            start_connection: SyllableConnection::Detached,
            end_connection: SyllableConnection::Detached,
            role: PhraseRole::ChattyReply,
            curves: PerformanceCurves::neutral(),
            tail_shape: TailShape::Sustained,
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

    pub(crate) fn with_connections(
        mut self,
        start_connection: SyllableConnection,
        end_connection: SyllableConnection,
    ) -> Self {
        self.start_connection = start_connection;
        self.end_connection = end_connection;

        self
    }

    pub(crate) fn with_curves(mut self, role: PhraseRole, curves: PerformanceCurves) -> Self {
        self.role = role;
        self.curves = curves;

        self
    }

    pub(crate) fn with_tail_shape(mut self, tail_shape: TailShape) -> Self {
        self.tail_shape = tail_shape;

        self
    }

    fn whistle_amount(self) -> f64 {
        whistle_sweep_amount(
            self.role,
            self.curves.archetype_tension(),
            self.curves.pitch_velocity(),
        )
    }

    fn roughness_amount(self) -> f64 {
        syllable_roughness_amount(
            self.role,
            self.curves.archetype_tension(),
            self.curves.brightness_pressure(),
            self.mood_valence,
            self.mood_arousal,
        )
    }
}

impl SyllableConnection {
    fn is_connected(self) -> bool {
        !matches!(self, Self::Detached)
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
        let curves = performance.curves;
        let brightness_gain = (1.0
            + (performance.mood_arousal * 0.45)
            + (performance.mood_valence * 0.12)
            + (curves.brightness_pressure() * CURVE_BRIGHTNESS_GAIN))
            .clamp(0.78, 1.85);
        let subgesture_density =
            1.0 + (performance.mood_arousal * 0.80) + (performance.complexity * 1.20);
        let subgesture_count = complexity_subgesture_count(performance.complexity);
        let whistle_amount = performance.whistle_amount();
        let roughness_amount = performance.roughness_amount();
        let mouth_openness = curves.mouth_openness();
        let mouth_front_back = curves.formant_target();
        let pitch_span = gesture_pitch_span_semitones(performance.role, whistle_amount);
        let pitch_bias_semitones = curves.pitch_center_bias() * CURVE_PITCH_BIAS_SEMITONES;
        let target_pitch_hz = pitch_center_hz_with_span(knobs.pitch_center(), pitch_span)
            * semitone_multiplier(performance.pitch_offset_with_emphasis() + pitch_bias_semitones);
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
            whistle_amount,
            roughness_amount,
            mouth_openness,
            mouth_front_back,
            tail_shape: performance.tail_shape,
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
            mouth: MouthStage::new(),
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
        samples.push(render_performance_sample(sample_index, &controls, state));
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
    let warble_depth = (knobs.warble_depth() * 0.28).clamp(-1.0, 1.0);

    for sample_index in 0..duration_samples {
        let elapsed_seconds = f64::from(sample_index) / f64::from(SYNTH_SAMPLE_RATE_HZ);
        let progress = bridge_progress(sample_index, duration_samples);
        let pitch_hz = start_pitch_hz + ((target_pitch_hz - start_pitch_hz) * progress);
        let pitch_hz = apply_warble_hz_with_phase(pitch_hz, warble_depth, elapsed_seconds, 0.25);
        let source = source_oscillator_sample(state.phase, pitch_hz);
        let voiced = state.formants.process_sample(source, vowel_position);
        let layer = (0.72 * voiced)
            + (0.12 * source)
            + (0.24 * body_layer_sample(state.body_phase, vowel_position));
        let bridge_envelope = TRANSITION_BRIDGE_ENVELOPE_FLOOR
            + (TRANSITION_BRIDGE_ENVELOPE_SPAN * sin(PI * progress));
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
    controls: &SyllableRenderControls,
    state: &mut SyllableRenderState,
) -> f64 {
    let elapsed_seconds = f64::from(sample_index) / f64::from(SYNTH_SAMPLE_RATE_HZ);
    let parameters = performance_sample_parameters(controls, state, elapsed_seconds);
    let source = source_oscillator_sample(state.phase, parameters.pitch_hz);
    let source = blend_noise_excitation(source, sample_index, controls.roughness_amount);
    let voiced = state
        .formants
        .process_sample(source, parameters.vowel_position);
    let attack_transient = if controls.performance.start_connection.is_connected() {
        0.0
    } else {
        attack_transient_sample(elapsed_seconds, parameters.contour)
    };
    let word_onset_texture_gain = word_onset_texture_gain(
        elapsed_seconds,
        controls.duration_seconds,
        controls.performance.start_connection,
    );
    let layered = voiced
        + body_layer_sample(state.body_phase, parameters.vowel_position)
        + attack_transient
        + (controls.brightness_gain
            * word_onset_texture_gain
            * sparkle_event_gain(
                controls.performance.curves.brightness_pressure(),
                elapsed_seconds,
                controls.duration_seconds,
            )
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
        ) * word_onset_texture_gain;
    let layered =
        layered * archetype_amplitude_gain(controls.performance.archetype, elapsed_seconds);
    let layered = if controls.performance.emphasized {
        layered * PHRASE_EMPHASIS_GAIN
    } else {
        layered
    };
    let electronic = ring_modulate(layered, elapsed_seconds);
    let mouthed = apply_mouth_stage(electronic, sample_index, elapsed_seconds, controls, state);

    state.advance(
        parameters.pitch_hz,
        parameters.vowel_position,
        controls.warble_depth,
        parameters.contour,
    );

    let enveloped = apply_connected_amplitude_envelope(
        mouthed,
        elapsed_seconds,
        controls.duration_seconds,
        controls.performance.start_connection,
        controls.performance.end_connection,
    );

    enveloped
        * tail_shape_gain(
            elapsed_seconds,
            controls.duration_seconds,
            controls.tail_shape,
        )
}

/// Fraction of a syllable that plays at full level before a clipped tail
/// begins.
const CLIP_TAIL_START_FRACTION: f64 = 0.68;
/// Fraction of a syllable that plays at full level before a decayed tail
/// begins.
const DECAY_TAIL_START_FRACTION: f64 = 0.45;
/// Exponential rate of the trailing-off (ellipsis) tail decay.
const DECAY_TAIL_RATE: f64 = 3.2;

/// Returns the `VOICE_V9` trailing-edge amplitude gain for one sample.
///
/// A clipped (dash) tail holds full level, then gates to near silence with a
/// steep quartic so the syllable cuts off abruptly; a decayed (ellipsis) tail
/// fades exponentially so the syllable trails off. The default sustained shape
/// is a transparent unity gain, so every non-hesitation syllable is unchanged.
fn tail_shape_gain(elapsed_seconds: f64, duration_seconds: f64, tail_shape: TailShape) -> f64 {
    if duration_seconds <= 0.0 || !elapsed_seconds.is_finite() {
        return 1.0;
    }

    let (start_fraction, gain): (f64, fn(f64) -> f64) = match tail_shape {
        TailShape::Sustained => return 1.0,
        TailShape::Clipped => (CLIP_TAIL_START_FRACTION, |t| {
            let remaining = 1.0 - t;

            remaining * remaining * remaining * remaining
        }),
        TailShape::Decayed => (DECAY_TAIL_START_FRACTION, |t| exp(-DECAY_TAIL_RATE * t)),
    };
    let tail_start = duration_seconds * start_fraction;

    if elapsed_seconds <= tail_start {
        return 1.0;
    }

    let progress =
        ((elapsed_seconds - tail_start) / (duration_seconds - tail_start)).clamp(0.0, 1.0);

    gain(progress)
}

fn apply_mouth_stage(
    sample: f64,
    sample_index: u32,
    elapsed_seconds: f64,
    controls: &SyllableRenderControls,
    state: &mut SyllableRenderState,
) -> f64 {
    if controls.mouth_openness <= 0.0 {
        return sample;
    }

    let openness =
        controls.mouth_openness * mouth_open_envelope(elapsed_seconds, controls.duration_seconds);

    state.mouth.process_sample(
        sample,
        sample_index,
        MouthDrive::new(
            openness,
            controls.mouth_front_back,
            controls.roughness_amount,
        ),
    )
}

fn performance_sample_parameters(
    controls: &SyllableRenderControls,
    state: &SyllableRenderState,
    elapsed_seconds: f64,
) -> PerformanceSampleParameters {
    let articulation = complexity_articulation_offset(
        controls.performance.complexity,
        controls.subgesture_count,
        elapsed_seconds,
        controls.duration_seconds,
    ) * connected_word_motion_gain(
        elapsed_seconds,
        controls.duration_seconds,
        controls.performance.start_connection,
    );
    let contour = (controls.contour + articulation).clamp(-1.0, 1.0);
    let connected_motion_gain = connected_word_motion_gain(
        elapsed_seconds,
        controls.duration_seconds,
        controls.performance.start_connection,
    );
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
    let internal_pitch_hz = apply_internal_pitch_swoop_hz_for_duration_with_gain(
        phrase_final_pitch_hz,
        contour,
        elapsed_seconds,
        controls.duration_seconds,
        connected_motion_gain,
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
        connected_motion_gain,
    );
    let pitch_hz = apply_whistle_sweep_hz(
        pitch_hz,
        controls.whistle_amount,
        elapsed_seconds,
        controls.duration_seconds,
    );
    let pitch_hz = apply_imperfection_detune_hz(
        pitch_hz,
        controls.performance.curves.archetype_tension(),
        controls.warble_phase_offset,
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
            controls.performance.start_connection,
        ),
        vowel_position: connected_vowel_position(
            vowel_position,
            state.last_vowel_position,
            elapsed_seconds,
            controls.performance.start_connection,
        ),
        contour,
    }
}

fn connected_parameter_value(
    current: f64,
    previous: Option<f64>,
    elapsed_seconds: f64,
    start_connection: SyllableConnection,
) -> f64 {
    let Some(previous) = previous else {
        return current;
    };

    if !start_connection.is_connected() || !elapsed_seconds.is_finite() {
        return current;
    }

    let blend_seconds = match start_connection {
        SyllableConnection::Detached => return current,
        SyllableConnection::Subword => CONNECTED_PARAMETER_BLEND_SECONDS,
        SyllableConnection::Word => WORD_CONNECTED_PARAMETER_BLEND_SECONDS,
    };

    if elapsed_seconds >= blend_seconds {
        return current;
    }

    let linear = (elapsed_seconds / blend_seconds).clamp(0.0, 1.0);
    let progress = linear * linear * (3.0 - (2.0 * linear));

    previous + ((current - previous) * progress)
}

fn connected_vowel_position(
    current: f64,
    previous: Option<f64>,
    elapsed_seconds: f64,
    start_connection: SyllableConnection,
) -> f64 {
    if start_connection == SyllableConnection::Word {
        return word_onset_vowel_position(current, elapsed_seconds);
    }

    connected_parameter_value(current, previous, elapsed_seconds, start_connection)
}

fn word_onset_vowel_position(current: f64, elapsed_seconds: f64) -> f64 {
    if !elapsed_seconds.is_finite() || elapsed_seconds >= WORD_ONSET_VOWEL_OPEN_SECONDS {
        return current;
    }

    let rounded_start = (current + WORD_ONSET_VOWEL_ROUNDING).clamp(-1.0, 1.0);
    let progress = smoothstep(elapsed_seconds / WORD_ONSET_VOWEL_OPEN_SECONDS);

    rounded_start + ((current - rounded_start) * progress)
}

fn word_onset_texture_gain(
    elapsed_seconds: f64,
    duration_seconds: f64,
    start_connection: SyllableConnection,
) -> f64 {
    if start_connection != SyllableConnection::Word
        || !elapsed_seconds.is_finite()
        || !duration_seconds.is_finite()
    {
        return 1.0;
    }

    if elapsed_seconds < WORD_ONSET_VOWEL_OPEN_SECONDS {
        let progress = smoothstep(elapsed_seconds / WORD_ONSET_VOWEL_OPEN_SECONDS);

        return WORD_ONSET_TEXTURE_START_GAIN
            + ((WORD_CONNECTED_TEXTURE_END_GAIN - WORD_ONSET_TEXTURE_START_GAIN) * progress);
    }

    WORD_CONNECTED_TEXTURE_END_GAIN
        + ((1.0 - WORD_CONNECTED_TEXTURE_END_GAIN) * smoothstep(elapsed_seconds / duration_seconds))
}

fn connected_word_motion_gain(
    elapsed_seconds: f64,
    duration_seconds: f64,
    start_connection: SyllableConnection,
) -> f64 {
    if start_connection != SyllableConnection::Word
        || !elapsed_seconds.is_finite()
        || !duration_seconds.is_finite()
    {
        return 1.0;
    }

    WORD_CONNECTED_MOTION_START_GAIN
        + ((WORD_CONNECTED_MOTION_END_GAIN - WORD_CONNECTED_MOTION_START_GAIN)
            * smoothstep(elapsed_seconds / duration_seconds))
}

fn apply_connected_amplitude_envelope(
    sample: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
    start_connection: SyllableConnection,
    end_connection: SyllableConnection,
) -> f64 {
    sample
        * connected_amplitude_envelope(
            elapsed_seconds,
            duration_seconds,
            start_connection,
            end_connection,
        )
}

fn connected_amplitude_envelope(
    elapsed_seconds: f64,
    duration_seconds: f64,
    start_connection: SyllableConnection,
    end_connection: SyllableConnection,
) -> f64 {
    let base = amplitude_envelope(elapsed_seconds, duration_seconds);
    let attack_connection_end = ENVELOPE_ATTACK_SECONDS + 0.018;
    let release_start = (duration_seconds - ENVELOPE_RELEASE_SECONDS)
        .max(ENVELOPE_ATTACK_SECONDS + ENVELOPE_DECAY_SECONDS);

    match start_connection {
        SyllableConnection::Subword if elapsed_seconds <= attack_connection_end => {
            let connection_floor = subword_connection_floor();
            let target = amplitude_envelope(attack_connection_end, duration_seconds);
            let progress = smoothstep(elapsed_seconds / attack_connection_end);

            return connection_floor + ((target - connection_floor) * progress);
        }
        SyllableConnection::Word if elapsed_seconds <= WORD_CONNECTION_OPEN_SECONDS => {
            let target = amplitude_envelope(WORD_CONNECTION_OPEN_SECONDS, duration_seconds);
            let progress = smoothstep(elapsed_seconds / WORD_CONNECTION_OPEN_SECONDS);

            return WORD_CONNECTION_FLOOR + ((target - WORD_CONNECTION_FLOOR) * progress);
        }
        SyllableConnection::Detached | SyllableConnection::Subword | SyllableConnection::Word => {}
    }

    if matches!(
        (start_connection, end_connection),
        (SyllableConnection::Word, _) | (_, SyllableConnection::Word),
    ) && elapsed_seconds < release_start
    {
        return WORD_CONNECTED_ENVELOPE_LEVEL
            + ((base - WORD_CONNECTED_ENVELOPE_LEVEL) * WORD_CONNECTED_ENVELOPE_CONTRAST);
    }

    if elapsed_seconds >= release_start {
        return match end_connection {
            SyllableConnection::Detached => base,
            SyllableConnection::Subword => base.max(subword_connection_floor()),
            SyllableConnection::Word => base.max(WORD_CONNECTION_FLOOR),
        };
    }

    base
}

fn subword_connection_floor() -> f64 {
    ENVELOPE_SUSTAIN_LEVEL * 0.95
}

fn apply_imperfection_detune_hz(pitch_hz: f64, tension: f64, warble_phase_offset: f64) -> f64 {
    let cents = imperfection_detune_cents(tension, warble_phase_offset);

    if cents == 0.0 {
        return pitch_hz;
    }

    pitch_hz * exp(LN_2 * (cents / 1_200.0))
}

/// Applies the swept whistle to a syllable's pitch over its progress.
///
/// The sweep stays dormant until `CURVE_WHISTLE_START_FRACTION` of the syllable
/// has elapsed, then runs [`whistle_sweep_pitch_hz`] over the remainder. A zero
/// amount is a no-op; the sign of the amount sets the sweep direction.
pub fn apply_whistle_sweep_hz(
    pitch_hz: f64,
    whistle_amount: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
) -> f64 {
    if whistle_amount == 0.0 {
        return pitch_hz;
    }

    let progress = syllable_progress(elapsed_seconds, duration_seconds);

    if progress < CURVE_WHISTLE_START_FRACTION {
        return pitch_hz;
    }

    let local = ((progress - CURVE_WHISTLE_START_FRACTION) / (1.0 - CURVE_WHISTLE_START_FRACTION))
        .clamp(0.0, 1.0);

    whistle_sweep_pitch_hz(pitch_hz, whistle_amount, local)
}

fn apply_archetype_pitch_hz(
    pitch_hz: f64,
    archetype: GestureArchetype,
    elapsed_seconds: f64,
    duration_seconds: f64,
    motion_gain: f64,
) -> f64 {
    let progress = syllable_progress(elapsed_seconds, duration_seconds);
    let semitones = match archetype {
        GestureArchetype::Chatter => 0.0,
        GestureArchetype::Yelp => 0.65 * progress,
        GestureArchetype::Moan => -0.55 * (1.0 - (0.35 * progress)),
        GestureArchetype::StutterBurst => 0.28 * stutter_pulse(progress),
        GestureArchetype::Tremble => 0.18 * sin(2.0 * PI * 29.0 * elapsed_seconds),
    };

    pitch_hz * semitone_multiplier(semitones * motion_gain.clamp(0.0, 1.0))
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

fn apply_internal_pitch_swoop_hz_for_duration_with_gain(
    pitch_hz: f64,
    contour: f64,
    elapsed_seconds: f64,
    duration_seconds: f64,
    motion_gain: f64,
) -> f64 {
    let cents =
        internal_pitch_offset_cents_for_duration(contour, elapsed_seconds, duration_seconds)
            * motion_gain.clamp(0.0, 1.0);

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
        SyllableFinalGlide::Rising => QUESTION_RISE_SEMITONES,
        SyllableFinalGlide::Falling => -PUNCTUATION_GLIDE_SEMITONES,
        SyllableFinalGlide::Continuation => CONTINUATION_GLIDE_SEMITONES,
    };
    let final_glide_start_seconds = duration_seconds - PORTAMENTO_SECONDS;

    if elapsed_seconds < final_glide_start_seconds {
        return pitch_hz;
    }

    let progress = portamento_progress(elapsed_seconds - final_glide_start_seconds, semitones);
    let final_target_hz = target_pitch_hz * exp((LN_2 * semitones) / 12.0);
    let risen_hz = pitch_hz + ((final_target_hz - pitch_hz) * progress);

    // VOICE_V9 (R5): a question gathers into a small pre-final dip (L*) before
    // its wide rise (H-H%). The dip is zero at both ends of the glide and peaks
    // mid-glide, so the syllable still lands on the full rise target.
    if matches!(final_glide, SyllableFinalGlide::Rising) {
        let dip_semitones = QUESTION_PREFINAL_DIP_SEMITONES * sin(PI * progress);

        return risen_hz * exp((LN_2 * -dip_semitones) / 12.0);
    }

    risen_hz
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
