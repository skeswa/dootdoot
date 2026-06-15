//! Directional BB-8 tuning metrics for Phase 7.

use std::{
    ffi::OsStr,
    fmt::Write as _,
    fs::{self, File},
    path::{Path, PathBuf},
};

use dootdoot_core::{SYNTH_SAMPLE_RATE_HZ, render_text_canonical_buffer, write_wav};
use num_traits::ToPrimitive;
use rustfft::{FftPlanner, num_complex::Complex};

use crate::{Result, SourceManifestError};

const FRAME_SIZE: usize = 2_048;
const HOP_SIZE: usize = 512;
const GATE_FRACTION: f64 = 0.08;
const ROLLOFF_FRACTION: f64 = 0.85;
const PHRASE_CORPUS: [&str; 6] = [
    "hello there",
    "where are you",
    "this is very exciting",
    "cat dog airplane",
    "playing",
    "?",
];

#[derive(Debug, Clone)]
struct CommandOptions {
    reference_wav_dir: PathBuf,
    output_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct AudioClip {
    samples: Vec<f64>,
}

#[derive(Debug, Clone)]
struct Frame {
    samples: Vec<f64>,
    rms: f64,
}

#[derive(Debug, Clone)]
struct ClipMetrics {
    active_fraction: f64,
    active_island_median_ms: f64,
    magnitude_centroid_hz: f64,
    magnitude_rolloff_85_hz: f64,
    dominant_peak_motion_hz: f64,
    harmonicity: f64,
    sub_500_hz_power: f64,
    mid_500_2000_hz_power: f64,
    upper_mid_2000_5000_hz_power: f64,
    over_6000_hz_power: f64,
}

#[derive(Debug, Clone)]
struct MetricSummary {
    clip_count: usize,
    active_fraction: Option<f64>,
    active_island_median_ms: Option<f64>,
    magnitude_centroid_hz: Option<f64>,
    magnitude_rolloff_85_hz: Option<f64>,
    dominant_peak_motion_hz: Option<f64>,
    harmonicity: Option<f64>,
    sub_500_hz_power: Option<f64>,
    mid_500_2000_hz_power: Option<f64>,
    upper_mid_2000_5000_hz_power: Option<f64>,
    over_6000_hz_power: Option<f64>,
}

/// Runs the BB-8 tuning metrics report.
pub(crate) fn run(args: &[String]) -> Result<String> {
    let options = CommandOptions::parse(args)?;
    let dootdoot_dir = options.output_dir.join("dootdoot-wav");

    fs::create_dir_all(&options.output_dir).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to create {}: {error}",
            options.output_dir.display(),
        ))
    })?;
    fs::create_dir_all(&dootdoot_dir).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to create {}: {error}",
            dootdoot_dir.display(),
        ))
    })?;
    render_phrase_corpus(&dootdoot_dir)?;

    let reference_metrics = metrics_for_wav_dir(&options.reference_wav_dir)?;
    let dootdoot_metrics = metrics_for_wav_dir(&dootdoot_dir)?;
    let report = format_report(
        &options.reference_wav_dir,
        &dootdoot_dir,
        &summarize_metrics(&reference_metrics),
        &summarize_metrics(&dootdoot_metrics),
    );
    let report_path = options.output_dir.join("bb8-metrics.md");

    fs::write(&report_path, &report).map_err(|error| {
        SourceManifestError::new(format!(
            "failed to write {}: {error}",
            report_path.display()
        ))
    })?;

    Ok(report)
}

impl CommandOptions {
    fn parse(args: &[String]) -> Result<Self> {
        let mut reference_wav_dir = None;
        let mut output_dir = None;
        let mut index = 0;

        while let Some(argument) = args.get(index) {
            match argument.as_str() {
                "--reference-wav-dir" => {
                    let value = args.get(index + 1).ok_or_else(|| {
                        SourceManifestError::new("--reference-wav-dir requires a path")
                    })?;
                    reference_wav_dir = Some(PathBuf::from(value));
                    index += 2;
                }
                "--output-dir" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| SourceManifestError::new("--output-dir requires a path"))?;
                    output_dir = Some(PathBuf::from(value));
                    index += 2;
                }
                unknown => {
                    return Err(SourceManifestError::new(format!(
                        "unknown bb8-metrics argument: {unknown}",
                    )));
                }
            }
        }

        Ok(Self {
            reference_wav_dir: reference_wav_dir
                .unwrap_or_else(|| PathBuf::from("target/bb8-metrics/reference-wav")),
            output_dir: output_dir.unwrap_or_else(|| PathBuf::from("target/bb8-metrics")),
        })
    }
}

fn render_phrase_corpus(output_dir: &Path) -> Result<()> {
    for phrase in PHRASE_CORPUS {
        let samples = render_text_canonical_buffer(phrase).map_err(|error| {
            SourceManifestError::new(format!("failed to render {phrase}: {error}"))
        })?;
        let file_name = phrase
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '-'
                }
            })
            .collect::<String>();
        let path = output_dir.join(format!("{file_name}.wav"));
        let file = File::create(&path).map_err(|error| {
            SourceManifestError::new(format!("failed to create {}: {error}", path.display()))
        })?;

        write_wav(file, &samples).map_err(|error| {
            SourceManifestError::new(format!("failed to write {}: {error}", path.display()))
        })?;
    }

    Ok(())
}

fn metrics_for_wav_dir(directory: &Path) -> Result<Vec<ClipMetrics>> {
    let mut metrics = Vec::new();

    if !directory.exists() {
        return Ok(metrics);
    }

    for path in sorted_wav_paths(directory)? {
        let clip = AudioClip::read_wav(&path)?;

        metrics.push(compute_clip_metrics(&clip)?);
    }

    Ok(metrics)
}

fn sorted_wav_paths(directory: &Path) -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir(directory).map_err(|error| {
        SourceManifestError::new(format!("failed to read {}: {error}", directory.display()))
    })?;
    let mut paths = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| {
            SourceManifestError::new(format!("failed to read directory entry: {error}"))
        })?;
        let path = entry.path();

        if path.extension() == Some(OsStr::new("wav")) {
            paths.push(path);
        }
    }

    paths.sort();

    Ok(paths)
}

impl AudioClip {
    fn read_wav(path: &Path) -> Result<Self> {
        let mut reader = hound::WavReader::open(path).map_err(|error| {
            SourceManifestError::new(format!("failed to open {}: {error}", path.display()))
        })?;
        let spec = reader.spec();

        if spec.channels != 1
            || spec.sample_rate != SYNTH_SAMPLE_RATE_HZ
            || spec.bits_per_sample != 16
            || spec.sample_format != hound::SampleFormat::Int
        {
            return Err(SourceManifestError::new(format!(
                "{} should be mono 44.1 kHz 16-bit PCM",
                path.display(),
            )));
        }

        let mut samples = Vec::new();

        for sample in reader.samples::<i16>() {
            let sample = sample.map_err(|error| {
                SourceManifestError::new(format!("failed to read {}: {error}", path.display()))
            })?;

            samples.push(f64::from(sample) / 32_768.0);
        }

        Ok(Self { samples })
    }
}

fn compute_clip_metrics(clip: &AudioClip) -> Result<ClipMetrics> {
    let frames = windowed_frames(&clip.samples)?;
    let peak_rms = frames.iter().map(|frame| frame.rms).fold(0.0, f64::max);
    let gate = peak_rms * GATE_FRACTION;
    let active_flags = frames
        .iter()
        .map(|frame| frame.rms > gate && frame.rms > 0.0)
        .collect::<Vec<_>>();
    let active_count = active_flags.iter().filter(|active| **active).count();
    let mut frame_metrics = Vec::new();
    let mut dominant_peaks = Vec::new();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FRAME_SIZE);

    for (frame, active) in frames.iter().zip(&active_flags) {
        if !active {
            continue;
        }

        let mut spectrum = frame
            .samples
            .iter()
            .copied()
            .map(|sample| Complex::new(sample, 0.0))
            .collect::<Vec<_>>();

        fft.process(&mut spectrum);

        let frame_metrics_result = compute_frame_metrics(&frame.samples, &spectrum)?;
        dominant_peaks.push(frame_metrics_result.dominant_peak_hz);
        frame_metrics.push(frame_metrics_result);
    }

    let frame_count = usize_to_f64(frames.len())?;
    let active_fraction = if frame_count == 0.0 {
        0.0
    } else {
        usize_to_f64(active_count)? / frame_count
    };

    Ok(ClipMetrics {
        active_fraction,
        active_island_median_ms: active_island_median_ms(&active_flags)?,
        magnitude_centroid_hz: median_option(frame_metrics.iter().map(|frame| frame.centroid_hz)),
        magnitude_rolloff_85_hz: median_option(
            frame_metrics.iter().map(|frame| frame.rolloff_85_hz),
        ),
        dominant_peak_motion_hz: range_or_zero(&dominant_peaks),
        harmonicity: median_option(frame_metrics.iter().map(|frame| frame.harmonicity)),
        sub_500_hz_power: median_option(frame_metrics.iter().map(|frame| frame.sub_500_hz_power)),
        mid_500_2000_hz_power: median_option(
            frame_metrics
                .iter()
                .map(|frame| frame.mid_500_2000_hz_power),
        ),
        upper_mid_2000_5000_hz_power: median_option(
            frame_metrics
                .iter()
                .map(|frame| frame.upper_mid_2000_5000_hz_power),
        ),
        over_6000_hz_power: median_option(
            frame_metrics.iter().map(|frame| frame.over_6000_hz_power),
        ),
    })
}

fn windowed_frames(samples: &[f64]) -> Result<Vec<Frame>> {
    let starts = frame_starts(samples.len());
    let window = hann_window()?;
    let mut frames = Vec::new();

    for start in starts {
        let mut frame_samples = Vec::with_capacity(FRAME_SIZE);

        for (offset, window_value) in window.iter().copied().enumerate() {
            let sample = samples.get(start + offset).copied().unwrap_or(0.0) * window_value;

            frame_samples.push(sample);
        }

        let rms = root_mean_square(&frame_samples)?;

        frames.push(Frame {
            samples: frame_samples,
            rms,
        });
    }

    Ok(frames)
}

fn frame_starts(sample_len: usize) -> Vec<usize> {
    if sample_len <= FRAME_SIZE {
        return vec![0];
    }

    (0..=(sample_len - FRAME_SIZE)).step_by(HOP_SIZE).collect()
}

fn hann_window() -> Result<Vec<f64>> {
    let denominator = usize_to_f64(FRAME_SIZE - 1)?;
    let mut window = Vec::with_capacity(FRAME_SIZE);

    for index in 0..FRAME_SIZE {
        let phase = (2.0 * core::f64::consts::PI * usize_to_f64(index)?) / denominator;

        window.push(0.5 - (0.5 * phase.cos()));
    }

    Ok(window)
}

fn root_mean_square(samples: &[f64]) -> Result<f64> {
    if samples.is_empty() {
        return Ok(0.0);
    }

    let energy = samples.iter().map(|sample| sample * sample).sum::<f64>();

    Ok((energy / usize_to_f64(samples.len())?).sqrt())
}

#[derive(Debug, Clone, Copy)]
struct FrameMetrics {
    centroid_hz: f64,
    rolloff_85_hz: f64,
    dominant_peak_hz: f64,
    harmonicity: f64,
    sub_500_hz_power: f64,
    mid_500_2000_hz_power: f64,
    upper_mid_2000_5000_hz_power: f64,
    over_6000_hz_power: f64,
}

fn compute_frame_metrics(samples: &[f64], spectrum: &[Complex<f64>]) -> Result<FrameMetrics> {
    let half_len = (FRAME_SIZE / 2) + 1;
    let sample_rate_hz = f64::from(SYNTH_SAMPLE_RATE_HZ);
    let frame_size_hz = usize_to_f64(FRAME_SIZE)?;
    let mut total_magnitude = 0.0;
    let mut weighted_magnitude = 0.0;
    let mut total_power = 0.0;
    let mut sub_500 = 0.0;
    let mut mid_500_2000 = 0.0;
    let mut upper_mid_2000_5000 = 0.0;
    let mut over_6000 = 0.0;
    let mut magnitudes = Vec::with_capacity(half_len);

    for (bin, value) in spectrum.iter().take(half_len).enumerate() {
        let frequency_hz = (usize_to_f64(bin)? * sample_rate_hz) / frame_size_hz;
        let magnitude = value.norm();
        let power = magnitude * magnitude;

        total_magnitude += magnitude;
        weighted_magnitude += frequency_hz * magnitude;
        total_power += power;

        if frequency_hz < 500.0 {
            sub_500 += power;
        } else if frequency_hz < 2_000.0 {
            mid_500_2000 += power;
        } else if frequency_hz < 5_000.0 {
            upper_mid_2000_5000 += power;
        } else if frequency_hz > 6_000.0 {
            over_6000 += power;
        }

        magnitudes.push((frequency_hz, magnitude));
    }

    let centroid_hz = if total_magnitude > 0.0 {
        weighted_magnitude / total_magnitude
    } else {
        0.0
    };
    let rolloff_85_hz = rolloff_frequency(&magnitudes, total_magnitude * ROLLOFF_FRACTION);
    let dominant_peak_hz = dominant_peak_frequency(&magnitudes);
    let band_denominator = if total_power > 0.0 { total_power } else { 1.0 };

    Ok(FrameMetrics {
        centroid_hz,
        rolloff_85_hz,
        dominant_peak_hz,
        harmonicity: harmonicity(samples),
        sub_500_hz_power: sub_500 / band_denominator,
        mid_500_2000_hz_power: mid_500_2000 / band_denominator,
        upper_mid_2000_5000_hz_power: upper_mid_2000_5000 / band_denominator,
        over_6000_hz_power: over_6000 / band_denominator,
    })
}

fn rolloff_frequency(magnitudes: &[(f64, f64)], threshold: f64) -> f64 {
    let mut cumulative = 0.0;

    for (frequency_hz, magnitude) in magnitudes {
        cumulative += magnitude;

        if cumulative >= threshold {
            return *frequency_hz;
        }
    }

    magnitudes
        .last()
        .map_or(0.0, |(frequency_hz, _)| *frequency_hz)
}

fn dominant_peak_frequency(magnitudes: &[(f64, f64)]) -> f64 {
    let mut best_frequency_hz = 0.0;
    let mut best_magnitude = f64::NEG_INFINITY;

    for (frequency_hz, magnitude) in magnitudes.iter().skip(1) {
        if *magnitude > best_magnitude {
            best_magnitude = *magnitude;
            best_frequency_hz = *frequency_hz;
        }
    }

    best_frequency_hz
}

fn harmonicity(samples: &[f64]) -> f64 {
    let denominator = samples.iter().map(|sample| sample * sample).sum::<f64>();

    if denominator <= 0.0 {
        return 0.0;
    }

    let min_lag = usize::try_from(SYNTH_SAMPLE_RATE_HZ / 1_500).unwrap_or(1);
    let max_lag = usize::try_from(SYNTH_SAMPLE_RATE_HZ / 80).unwrap_or(samples.len() / 2);
    let bounded_max_lag = max_lag.min(samples.len().saturating_sub(1));
    let mut best = 0.0;

    for lag in min_lag..=bounded_max_lag {
        let mut correlation = 0.0;

        for index in 0..(samples.len() - lag) {
            correlation += samples[index] * samples[index + lag];
        }

        let normalized = (correlation / denominator).max(0.0);

        if normalized > best {
            best = normalized;
        }
    }

    best
}

fn active_island_median_ms(active_flags: &[bool]) -> Result<f64> {
    let mut islands = Vec::new();
    let mut current_frames = 0;

    for active in active_flags {
        if *active {
            current_frames += 1;
        } else if current_frames > 0 {
            islands.push(active_frames_to_ms(current_frames)?);
            current_frames = 0;
        }
    }

    if current_frames > 0 {
        islands.push(active_frames_to_ms(current_frames)?);
    }

    Ok(median_option(islands))
}

fn active_frames_to_ms(frame_count: usize) -> Result<f64> {
    let samples = (frame_count.saturating_sub(1) * HOP_SIZE) + FRAME_SIZE;

    Ok((usize_to_f64(samples)? * 1_000.0) / f64::from(SYNTH_SAMPLE_RATE_HZ))
}

fn range_or_zero(values: &[f64]) -> f64 {
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;

    for value in values {
        minimum = minimum.min(*value);
        maximum = maximum.max(*value);
    }

    if values.is_empty() {
        0.0
    } else {
        maximum - minimum
    }
}

fn summarize_metrics(metrics: &[ClipMetrics]) -> MetricSummary {
    MetricSummary {
        clip_count: metrics.len(),
        active_fraction: metric_median(metrics, |metric| metric.active_fraction),
        active_island_median_ms: metric_median(metrics, |metric| metric.active_island_median_ms),
        magnitude_centroid_hz: metric_median(metrics, |metric| metric.magnitude_centroid_hz),
        magnitude_rolloff_85_hz: metric_median(metrics, |metric| metric.magnitude_rolloff_85_hz),
        dominant_peak_motion_hz: metric_median(metrics, |metric| metric.dominant_peak_motion_hz),
        harmonicity: metric_median(metrics, |metric| metric.harmonicity),
        sub_500_hz_power: metric_median(metrics, |metric| metric.sub_500_hz_power),
        mid_500_2000_hz_power: metric_median(metrics, |metric| metric.mid_500_2000_hz_power),
        upper_mid_2000_5000_hz_power: metric_median(metrics, |metric| {
            metric.upper_mid_2000_5000_hz_power
        }),
        over_6000_hz_power: metric_median(metrics, |metric| metric.over_6000_hz_power),
    }
}

fn metric_median<F>(metrics: &[ClipMetrics], metric: F) -> Option<f64>
where
    F: Fn(&ClipMetrics) -> f64,
{
    if metrics.is_empty() {
        return None;
    }

    Some(median_option(metrics.iter().map(metric)))
}

fn median_option<I>(values: I) -> f64
where
    I: IntoIterator<Item = f64>,
{
    let mut values = values.into_iter().collect::<Vec<_>>();

    values.sort_by(f64::total_cmp);

    if values.is_empty() {
        return 0.0;
    }

    values[values.len() / 2]
}

fn format_report(
    reference_dir: &Path,
    dootdoot_dir: &Path,
    reference: &MetricSummary,
    dootdoot: &MetricSummary,
) -> String {
    let mut report = String::new();

    writeln!(report, "# BB-8 Tuning Metrics\n").expect("writing to String cannot fail");
    writeln!(report, "- reference WAV dir: {}", reference_dir.display())
        .expect("writing to String cannot fail");
    writeln!(report, "- dootdoot WAV dir: {}", dootdoot_dir.display())
        .expect("writing to String cannot fail");
    writeln!(report, "- corpus: {}", PHRASE_CORPUS.join(", "),)
        .expect("writing to String cannot fail");
    writeln!(
        report,
        "- convention: Hann 2048 / hop 512; active-island metrics are gate-dependent; centroid and 85% rolloff use magnitude spectrum; BB-8 brightness target is 2-5 kHz upper-mid energy, not above 6 kHz.\n",
    )
    .expect("writing to String cannot fail");
    writeln!(
        report,
        "| set | clips | active fraction | active island median ms | magnitude centroid hz | magnitude rolloff 85 hz | dominant peak motion hz | harmonicity | sub-500 hz power | 500-2000 hz power | 2000-5000 hz power | over-6000 hz power |",
    )
    .expect("writing to String cannot fail");
    writeln!(
        report,
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    )
    .expect("writing to String cannot fail");
    write_summary_row(&mut report, "BB-8 reference", reference);
    write_summary_row(&mut report, "dootdoot corpus", dootdoot);

    report
}

fn write_summary_row(report: &mut String, name: &str, summary: &MetricSummary) {
    writeln!(
        report,
        "| {name} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        summary.clip_count,
        format_optional(summary.active_fraction),
        format_optional(summary.active_island_median_ms),
        format_optional(summary.magnitude_centroid_hz),
        format_optional(summary.magnitude_rolloff_85_hz),
        format_optional(summary.dominant_peak_motion_hz),
        format_optional(summary.harmonicity),
        format_optional(summary.sub_500_hz_power),
        format_optional(summary.mid_500_2000_hz_power),
        format_optional(summary.upper_mid_2000_5000_hz_power),
        format_optional(summary.over_6000_hz_power),
    )
    .expect("writing to String cannot fail");
}

fn format_optional(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".to_owned(), |value| format!("{value:.3}"))
}

fn usize_to_f64(value: usize) -> Result<f64> {
    value
        .to_f64()
        .ok_or_else(|| SourceManifestError::new(format!("value does not fit f64: {value}")))
}
