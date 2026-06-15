//! Phrase-continuity regression tests.

use dootdoot_core::{
    BASE_SYLLABLE_SAMPLES, KnobSet, LEADING_SILENCE_SAMPLES, SYNTH_SAMPLE_RATE_HZ, SequenceEvent,
    SquashedVector, TRAILING_SILENCE_SAMPLES, WORD_PAUSE_SAMPLES, assemble_knobs,
    estimate_utterance_sample_count, render_canonical_buffer, render_text_canonical_buffer,
    sequence_events_for_text,
};

#[test]
fn excited_phrase_has_no_hard_zero_holes_inside_the_voiced_body() {
    let buffer =
        render_text_canonical_buffer("I am so excited wooohooo!").expect("phrase should render");
    let body = voiced_body(&buffer);
    let maximum_zero_run = longest_zero_run(body);
    let five_ms = usize::try_from((SYNTH_SAMPLE_RATE_HZ * 5) / 1_000)
        .expect("sample-rate threshold fits usize");

    assert!(
        maximum_zero_run < five_ms,
        "phrase body contains a hard zero hole of {maximum_zero_run} samples",
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
