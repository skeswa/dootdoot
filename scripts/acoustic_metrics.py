# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "numpy>=2,<3",
#   "scipy>=1.13,<2",
# ]
# [tool.uv]
# exclude-newer = "2026-06-01T00:00:00Z"
# ///
"""Directional BB-8 acoustic metrics for VOICE_V* tuning.

This is a *tuning aid*, not part of the deterministic voice contract. It reports
the directional measurements used by the gap-analysis documents
(``docs/research/bb8-*-gap-analysis.md``): active fraction, active islands, the
largest internal silence, dominant-peak range, the fraction of active frames
above the old ~1.1 kHz pitch ceiling, harmonicity median/IQR, spectral centroid,
and the 2-5 kHz upper-mid share.

Run via the locked uv environment so numpy/scipy are reproducible:

    uv run scripts/acoustic_metrics.py reference=ref.wav dootdoot=render.wav

Each argument is ``LABEL=path-to-wav`` (mono 16-bit PCM WAV, any sample rate).
"""

from __future__ import annotations

import sys
import wave
from dataclasses import dataclass

import numpy as np
from scipy.fft import rfft, rfftfreq

FRAME = 2048
HOP = 512
GATE_FRACTION = 0.08
PITCH_CEILING_HZ = 1135.0
PEAK_LO_HZ = 50.0
PEAK_HI_HZ = 8000.0
UPPER_MID_LO_HZ = 2000.0
UPPER_MID_HI_HZ = 5000.0


@dataclass
class Metrics:
    duration_s: float
    active_fraction: float
    active_islands: int
    median_island_ms: float
    max_internal_gap_ms: float
    dominant_peak_range_hz: float
    fraction_above_ceiling: float
    harmonicity_median: float
    harmonicity_iqr: float
    spectral_centroid_hz: float
    upper_mid_share_median: float
    upper_mid_share_max: float


def read_wav_mono(path: str) -> tuple[np.ndarray, int]:
    with wave.open(path, "rb") as handle:
        channels = handle.getnchannels()
        width = handle.getsampwidth()
        rate = handle.getframerate()
        frames = handle.readframes(handle.getnframes())

    if width != 2:
        raise ValueError(f"{path}: expected 16-bit PCM, got {width * 8}-bit")

    samples = np.frombuffer(frames, dtype="<i2").astype(np.float64) / 32768.0
    if channels > 1:
        samples = samples.reshape(-1, channels).mean(axis=1)

    return samples, rate


def harmonicity(frame: np.ndarray, rate: int) -> float:
    """Normalized autocorrelation peak over a plausible pitch-lag range."""
    centered = frame - frame.mean()
    energy = float(np.dot(centered, centered))
    if energy <= 0.0:
        return 0.0

    correlation = np.correlate(centered, centered, mode="full")[len(centered) - 1 :]
    lo = max(1, int(rate / 1000.0))
    hi = min(len(correlation) - 1, int(rate / 60.0))
    if hi <= lo:
        return 0.0

    return float(correlation[lo:hi].max() / energy)


def analyze(path: str) -> Metrics:
    samples, rate = read_wav_mono(path)
    window = np.hanning(FRAME)
    frequencies = rfftfreq(FRAME, 1.0 / rate)
    peak_band = (frequencies >= PEAK_LO_HZ) & (frequencies <= PEAK_HI_HZ)
    upper_band = (frequencies >= UPPER_MID_LO_HZ) & (frequencies <= UPPER_MID_HI_HZ)
    peak_indices = np.flatnonzero(peak_band)

    rms = []
    dominant = []
    centroid = []
    upper_share = []
    harmonic = []

    for start in range(0, len(samples) - FRAME + 1, HOP):
        segment = samples[start : start + FRAME] * window
        rms.append(float(np.sqrt(np.mean(segment**2))))
        spectrum = np.abs(rfft(segment))
        power = spectrum**2
        total = float(power.sum()) + 1e-12
        dominant.append(float(frequencies[peak_indices[np.argmax(spectrum[peak_band])]]))
        centroid.append(float((frequencies * power).sum() / total))
        upper_share.append(float(power[upper_band].sum() / total))
        harmonic.append(harmonicity(segment, rate))

    rms = np.array(rms)
    if rms.size == 0:
        raise ValueError(f"{path}: too short to analyze")

    gate = GATE_FRACTION * rms.max()
    active_mask = rms >= gate
    active = np.flatnonzero(active_mask)
    active_fraction = float(active.size / rms.size)

    islands = 0
    island_lengths = []
    run = 0
    for flag in active_mask:
        if flag:
            run += 1
        elif run:
            islands += 1
            island_lengths.append(run)
            run = 0
    if run:
        islands += 1
        island_lengths.append(run)
    median_island_ms = (
        float(np.median(island_lengths)) * HOP / rate * 1000.0 if island_lengths else 0.0
    )

    max_gap = 0
    for previous, current in zip(active[:-1], active[1:]):
        max_gap = max(max_gap, int(current - previous - 1))
    max_gap_ms = max_gap * HOP / rate * 1000.0

    dominant = np.array(dominant)
    centroid = np.array(centroid)
    upper_share = np.array(upper_share)
    harmonic = np.array(harmonic)

    active_dominant = dominant[active] if active.size else dominant[:0]
    peak_range = float(active_dominant.max() - active_dominant.min()) if active.size else 0.0
    above = float(np.mean(active_dominant > PITCH_CEILING_HZ)) if active.size else 0.0
    active_harmonic = harmonic[active] if active.size else harmonic[:0]
    harmonic_median = float(np.median(active_harmonic)) if active.size else 0.0
    harmonic_iqr = (
        float(np.subtract(*np.percentile(active_harmonic, [75, 25]))) if active.size else 0.0
    )
    centroid_median = float(np.median(centroid[active])) if active.size else 0.0
    share_active = upper_share[active] if active.size else upper_share[:0]
    share_median = float(np.median(share_active)) if active.size else 0.0
    share_max = float(share_active.max()) if active.size else 0.0

    return Metrics(
        duration_s=len(samples) / rate,
        active_fraction=active_fraction,
        active_islands=islands,
        median_island_ms=median_island_ms,
        max_internal_gap_ms=max_gap_ms,
        dominant_peak_range_hz=peak_range,
        fraction_above_ceiling=above,
        harmonicity_median=harmonic_median,
        harmonicity_iqr=harmonic_iqr,
        spectral_centroid_hz=centroid_median,
        upper_mid_share_median=share_median,
        upper_mid_share_max=share_max,
    )


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__)
        return 2

    rows = []
    for argument in argv:
        if "=" not in argument:
            print(f"error: expected LABEL=path, got {argument!r}", file=sys.stderr)
            return 2
        label, path = argument.split("=", 1)
        rows.append((label, analyze(path)))

    fields = [
        ("duration_s", "duration(s)", "{:.2f}"),
        ("active_fraction", "active_frac", "{:.2f}"),
        ("active_islands", "islands", "{:d}"),
        ("median_island_ms", "median_island(ms)", "{:.0f}"),
        ("max_internal_gap_ms", "max_gap(ms)", "{:.0f}"),
        ("dominant_peak_range_hz", "peak_range(Hz)", "{:.0f}"),
        ("fraction_above_ceiling", "frac>1.1kHz", "{:.2f}"),
        ("harmonicity_median", "harmonicity_med", "{:.3f}"),
        ("harmonicity_iqr", "harmonicity_iqr", "{:.3f}"),
        ("spectral_centroid_hz", "centroid(Hz)", "{:.0f}"),
        ("upper_mid_share_median", "2-5kHz_med", "{:.3f}"),
        ("upper_mid_share_max", "2-5kHz_max", "{:.3f}"),
    ]

    label_width = max(len(label) for label, _ in rows)
    for attribute, title, fmt in fields:
        cells = "  ".join(
            f"{label:>{label_width}}={fmt.format(getattr(metric, attribute))}"
            for label, metric in rows
        )
        print(f"{title:>18}: {cells}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
