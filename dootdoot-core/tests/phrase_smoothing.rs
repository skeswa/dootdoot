//! Phrase-continuity regression tests.

use dootdoot_core::{
    BASE_SYLLABLE_SAMPLES, KnobSet, LEADING_SILENCE_SAMPLES, STAGED_REPLY_REST_MAX_SAMPLES,
    SYNTH_SAMPLE_RATE_HZ, SequenceEvent, SquashedVector, TRAILING_SILENCE_SAMPLES,
    WORD_PAUSE_SAMPLES, assemble_knobs, estimate_utterance_sample_count, render_canonical_buffer,
    render_text_canonical_buffer, sequence_events_for_text, staged_reply_rest_samples,
};

/// Gives the `VOICE_V8` neutral word-rest amount used by `deploy_one_timing`.
const NEUTRAL_WORD_REST_AMOUNT: f64 = 0.2;

#[test]
fn excited_phrase_holes_are_bounded_word_rests() {
    // VOICE_V8: a trailing "!" flourishes only the terminal syllable, so the
    // exclamation's chatty lead-in de-bridges into short word rests (the same
    // staging the plain statement uses). The voiced body may therefore contain
    // intentional silences, but none should exceed a staged word rest — there
    // must be no glitchy hard hole beyond a deliberate inter-word rest.
    let buffer =
        render_text_canonical_buffer("I am so excited wooohooo!").expect("phrase should render");
    let body = voiced_body(&buffer);
    let maximum_zero_run = longest_zero_run(body);
    let rest_ceiling =
        usize::try_from(STAGED_REPLY_REST_MAX_SAMPLES).expect("rest ceiling fits usize");

    assert!(
        maximum_zero_run <= rest_ceiling,
        "phrase body has a {maximum_zero_run}-sample hole, larger than a staged word rest",
    );
}

#[test]
fn excited_phrase_renders_as_phrase_level_active_islands() {
    let buffer =
        render_text_canonical_buffer("I am so excited wooohooo!").expect("phrase should render");
    let islands = active_island_count(voiced_body(&buffer));

    assert!(
        islands <= 6,
        "excited phrase rendered as {islands} short active islands",
    );
}

#[test]
fn repeated_connected_subwords_do_not_fire_hard_onsets() {
    let text = "hahahahahahahahahahah";
    let buffer = render_text_canonical_buffer(text).expect("phrase should render");
    let events = sequence_events_for_text(text).expect("phrase should sequence");
    let boundaries = connected_syllable_starts(&events);

    assert!(
        boundaries.len() >= 8,
        "repeated phrase should contain many connected starts",
    );

    let duration_samples = continuous_syllable_duration_samples(&events);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let onset_window =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 12) / 1_000).expect("onset window fits usize");
    let body_offset =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 50) / 1_000).expect("body offset fits usize");
    let mut roughness_ratios = Vec::new();

    for syllable_index in boundaries {
        let start = leading + (syllable_index * duration_samples);
        let onset = &buffer[start..start + onset_window];
        let body_start = start + body_offset;
        let body = &buffer[body_start..body_start + onset_window];
        let onset_roughness = derivative_rms(onset);
        let body_roughness = derivative_rms(body);

        roughness_ratios.push(onset_roughness / body_roughness.max(0.000_001));
    }

    roughness_ratios.sort_by(f64::total_cmp);

    let median_ratio = roughness_ratios[roughness_ratios.len() / 2];

    assert!(
        median_ratio <= 2.40,
        "connected starts are too click-like; median roughness ratio was {median_ratio:.2}",
    );
}

#[test]
fn word_boundary_connections_open_as_smooth_vowel_blooms() {
    let events = [
        SequenceEvent::syllable(test_knobs([0.05, -0.20, -0.10, 0.10]), false),
        SequenceEvent::syllable(test_knobs([-0.30, 0.10, 0.15, 0.65]), false),
        SequenceEvent::syllable(test_knobs([0.15, -0.15, -0.45, 0.20]), false),
        SequenceEvent::syllable(test_knobs([-0.60, 0.05, 0.05, -0.05]), false),
    ];
    let buffer = render_canonical_buffer(&events);
    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let syllable_stride = usize::try_from(BASE_SYLLABLE_SAMPLES + WORD_PAUSE_SAMPLES)
        .expect("word syllable stride fits usize");
    let onset_window =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 18) / 1_000).expect("onset window fits usize");
    let body_offset =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 45) / 1_000).expect("body offset fits usize");
    let body_window =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 40) / 1_000).expect("body window fits usize");
    let mut onset_body_ratios = Vec::new();
    let mut onset_roughness_ratios = Vec::new();

    for syllable_index in 1..events.len() {
        let start = leading + (syllable_index * syllable_stride);
        let onset = &buffer[start..start + onset_window];
        let body_start = start + body_offset;
        let body = &buffer[body_start..body_start + body_window];

        onset_body_ratios.push(rms(onset) / rms(body).max(0.000_001));
        onset_roughness_ratios.push(derivative_rms(onset) / derivative_rms(body).max(0.000_001));
    }

    onset_body_ratios.sort_by(f64::total_cmp);
    onset_roughness_ratios.sort_by(f64::total_cmp);

    let median_level_ratio = onset_body_ratios[onset_body_ratios.len() / 2];
    let median_roughness_ratio = onset_roughness_ratios[onset_roughness_ratios.len() / 2];

    assert!(
        median_level_ratio <= 1.65,
        "word starts are too blocky; median onset/body level ratio was {median_level_ratio:.2}",
    );
    assert!(
        median_roughness_ratio <= 1.90,
        "word starts are too sharp; median onset/body roughness ratio was {median_roughness_ratio:.2}",
    );
}

#[test]
fn repeated_neutral_phrase_de_bridges_into_silent_word_rests() {
    // VOICE_V8: a plain, unpunctuated repeated phrase no longer bridges every
    // word boundary with foreground tone. Instead each word boundary becomes a
    // short, near-silent rest, so the phrase reads as rest-separated islands and
    // there is no two-pulse bridge tremolo between words.
    let text = "I am so excited I am so excited I am so excited I am so excited";
    let buffer = render_text_canonical_buffer(text).expect("phrase should render");
    let events = sequence_events_for_text(text).expect("phrase should sequence");
    let syllable_count = syllable_count(&events);
    let word_boundary_count = word_boundary_count(&events);

    assert_eq!(
        syllable_count, 16,
        "fixture should keep the reported repeated phrase shape",
    );
    assert_eq!(
        word_boundary_count, 15,
        "fixture should exercise neutral word boundaries only",
    );

    let leading = usize::try_from(LEADING_SILENCE_SAMPLES).expect("leading silence fits usize");
    let trailing = usize::try_from(TRAILING_SILENCE_SAMPLES).expect("trailing silence fits usize");
    let word_rest = usize::try_from(staged_reply_rest_samples(NEUTRAL_WORD_REST_AMOUNT))
        .expect("word rest fits usize");
    let voiced_samples = buffer
        .len()
        .checked_sub(leading + trailing + (word_boundary_count * word_rest))
        .expect("fixture render should contain voiced samples");

    assert_eq!(
        voiced_samples % syllable_count,
        0,
        "fixture syllables should have uniform duration",
    );

    let syllable_samples = voiced_samples / syllable_count;
    let mut rest_level_ratios = Vec::new();
    let mut position = leading;

    for boundary_index in 0..word_boundary_count {
        let syllable = &buffer[position..position + syllable_samples];
        let rest_start = position + syllable_samples;
        let rest = &buffer[rest_start..rest_start + word_rest];

        rest_level_ratios.push(rms(rest) / rms(syllable).max(0.000_001));
        position = rest_start + word_rest;

        assert!(
            boundary_index + 1 < syllable_count,
            "fixture should not count a boundary after the final syllable",
        );
    }

    rest_level_ratios.sort_by(f64::total_cmp);

    let median_rest_ratio = rest_level_ratios[rest_level_ratios.len() / 2];
    let cycle_seconds =
        usize_to_f64(syllable_samples + word_rest) / f64::from(SYNTH_SAMPLE_RATE_HZ);
    let word_cycle_hz = 1.0 / cycle_seconds;
    let double_cycle_hz = 2.0 * word_cycle_hz;
    let body = &buffer[leading..buffer.len() - trailing];
    let word_cycle_energy = envelope_modulation_strength(body, word_cycle_hz);
    let double_cycle_energy = envelope_modulation_strength(body, double_cycle_hz);

    assert!(
        median_rest_ratio <= 0.20,
        "neutral word boundaries should be near-silent rests, not bridges; median rest/syllable RMS ratio was {median_rest_ratio:.2}",
    );
    assert!(
        double_cycle_energy <= word_cycle_energy * 0.85,
        "two-pulse word-cycle tremolo is too strong; {double_cycle_hz:.2} Hz energy was {double_cycle_energy:.2} vs {word_cycle_hz:.2} Hz energy {word_cycle_energy:.2}",
    );
}

fn voiced_body(buffer: &[i16]) -> &[i16] {
    let start = buffer
        .iter()
        .position(|sample| *sample != 0)
        .expect("rendered phrase should contain non-zero audio");
    let end = buffer
        .iter()
        .rposition(|sample| *sample != 0)
        .expect("rendered phrase should contain non-zero audio");

    &buffer[start..=end]
}

fn test_knobs(axes: [f64; 4]) -> KnobSet {
    assemble_knobs(
        SquashedVector::new([0.0, 0.0, 0.0, 0.0]),
        SquashedVector::new(axes),
    )
}

fn active_island_count(samples: &[i16]) -> usize {
    let frame_samples =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 10) / 1_000).expect("frame length fits usize");
    let hop_samples =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 2) / 1_000).expect("hop length fits usize");
    let frames = rms_frames(samples, frame_samples, hop_samples);
    let peak = frames.iter().copied().fold(0.0_f64, f64::max);
    let threshold = (peak * 0.08).max(0.002);
    let mut islands = Vec::new();
    let mut current_start = None;
    let mut current_end = 0_usize;

    for (index, rms) in frames.iter().copied().enumerate() {
        if rms >= threshold {
            if current_start.is_none() {
                current_start = Some(index);
            }

            current_end = index;
        } else if let Some(start) = current_start.take() {
            islands.push((start, current_end));
        }
    }

    if let Some(start) = current_start {
        islands.push((start, current_end));
    }

    merge_close_islands(islands, frame_samples, hop_samples).len()
}

fn rms_frames(samples: &[i16], frame_samples: usize, hop_samples: usize) -> Vec<f64> {
    if samples.len() < frame_samples {
        return vec![rms(samples)];
    }

    let mut frames = Vec::new();
    let mut start = 0_usize;

    while start + frame_samples <= samples.len() {
        frames.push(rms(&samples[start..start + frame_samples]));
        start = start.saturating_add(hop_samples);
    }

    frames
}

fn rms(samples: &[i16]) -> f64 {
    let sum = samples
        .iter()
        .map(|sample| {
            let normalized = f64::from(*sample) / 32_768.0;

            normalized * normalized
        })
        .sum::<f64>();
    let count = u32::try_from(samples.len()).expect("sample window length fits u32");

    (sum / f64::from(count)).sqrt()
}

fn merge_close_islands(
    islands: Vec<(usize, usize)>,
    frame_samples: usize,
    hop_samples: usize,
) -> Vec<(usize, usize)> {
    let merge_gap_samples =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 20) / 1_000).expect("merge gap fits usize");
    let mut merged = Vec::new();

    for island in islands {
        if let Some((_, previous_end)) = merged.last_mut() {
            let gap_frames = island.0.saturating_sub(*previous_end);
            let gap_samples = gap_frames.saturating_mul(hop_samples) + frame_samples;

            if gap_samples <= merge_gap_samples {
                *previous_end = island.1;
                continue;
            }
        }

        merged.push(island);
    }

    merged
}

fn longest_zero_run(samples: &[i16]) -> usize {
    let mut current = 0_usize;
    let mut longest = 0_usize;

    for sample in samples {
        if *sample == 0 {
            current = current.saturating_add(1);
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }

    longest
}

fn connected_syllable_starts(events: &[SequenceEvent]) -> Vec<usize> {
    events
        .iter()
        .filter_map(|event| match event {
            SequenceEvent::Syllable(syllable) => Some(syllable.is_continuation()),
            SequenceEvent::Mood(_)
            | SequenceEvent::Complexity(_)
            | SequenceEvent::Archetype(_)
            | SequenceEvent::Punctuation(_) => None,
        })
        .enumerate()
        .filter_map(|(index, is_continuation)| is_continuation.then_some(index))
        .collect()
}

fn syllable_count(events: &[SequenceEvent]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, SequenceEvent::Syllable(_)))
        .count()
}

fn word_boundary_count(events: &[SequenceEvent]) -> usize {
    let continuations = syllable_continuations(events);

    continuations.windows(2).filter(|window| !window[1]).count()
}

fn syllable_continuations(events: &[SequenceEvent]) -> Vec<bool> {
    events
        .iter()
        .filter_map(|event| match event {
            SequenceEvent::Syllable(syllable) => Some(syllable.is_continuation()),
            SequenceEvent::Mood(_)
            | SequenceEvent::Complexity(_)
            | SequenceEvent::Archetype(_)
            | SequenceEvent::Punctuation(_) => None,
        })
        .collect()
}

fn continuous_syllable_duration_samples(events: &[SequenceEvent]) -> usize {
    let syllable_count = events
        .iter()
        .filter(|event| matches!(event, SequenceEvent::Syllable(_)))
        .count();
    let leading = u64::from(LEADING_SILENCE_SAMPLES);
    let trailing = u64::from(TRAILING_SILENCE_SAMPLES);
    let estimated = estimate_utterance_sample_count(events);
    let voiced_samples = estimated
        .checked_sub(leading + trailing)
        .expect("continuous phrase estimate includes fixed padding");
    let syllable_count_u64 = u64::try_from(syllable_count).expect("syllable count fits u64");

    assert_eq!(
        voiced_samples % syllable_count_u64,
        0,
        "repeated test phrase should have uniform connected syllable durations",
    );

    usize::try_from(voiced_samples / syllable_count_u64).expect("syllable duration fits usize")
}

fn derivative_rms(samples: &[i16]) -> f64 {
    let sum = samples
        .windows(2)
        .map(|window| {
            let previous = f64::from(window[0]) / 32_768.0;
            let next = f64::from(window[1]) / 32_768.0;
            let delta = next - previous;

            delta * delta
        })
        .sum::<f64>();
    let count = u32::try_from(samples.len().saturating_sub(1)).expect("window length fits u32");

    (sum / f64::from(count)).sqrt()
}

fn envelope_modulation_strength(samples: &[i16], frequency_hz: f64) -> f64 {
    let frame_samples =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 20) / 1_000).expect("frame length fits usize");
    let hop_samples =
        usize::try_from((SYNTH_SAMPLE_RATE_HZ * 5) / 1_000).expect("hop length fits usize");
    let frames = rms_frames(samples, frame_samples, hop_samples);
    let peak = frames.iter().copied().fold(0.0_f64, f64::max);
    let active_threshold = peak * 0.08;
    let mut active = Vec::new();

    for (index, value) in frames.iter().copied().enumerate() {
        if value >= active_threshold {
            active.push((
                usize_to_f64(index * hop_samples) / f64::from(SYNTH_SAMPLE_RATE_HZ),
                value,
            ));
        }
    }

    if active.is_empty() {
        return 0.0;
    }

    let mean = active
        .iter()
        .map(|(_, value)| (value + 0.000_000_001).ln())
        .sum::<f64>()
        / usize_to_f64(active.len());
    let mut real = 0.0;
    let mut imaginary = 0.0;
    let mut norm = 0.0;

    for (time_seconds, value) in active {
        let centered = (value + 0.000_000_001).ln() - mean;
        let angle = 2.0 * core::f64::consts::PI * frequency_hz * time_seconds;

        real += centered * angle.cos();
        imaginary -= centered * angle.sin();
        norm += centered * centered;
    }

    ((real * real) + (imaginary * imaginary)).sqrt() / norm.sqrt().max(0.000_001)
}

fn usize_to_f64(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
