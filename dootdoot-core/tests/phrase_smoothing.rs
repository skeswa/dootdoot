//! Phrase-continuity regression tests.

use dootdoot_core::{SYNTH_SAMPLE_RATE_HZ, render_text_canonical_buffer};

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
