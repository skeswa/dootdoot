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

It additionally reports a **timbre/texture** block used by the corpus comparison
(``docs/research/bb8-corpus-timbre-texture-analysis.md``): spectral flatness
(tonal vs noisy), spectral rolloff and bandwidth, zero-crossing rate, spectral
flux (frame-to-frame animation), onset rate, amplitude-tremolo rate and depth,
and crest factor. These describe *how the sound is colored and how it moves*,
which is what separates a performed droid clip from a steady synth patch.

Run via the locked uv environment so numpy/scipy are reproducible:

    # Side-by-side comparison of individual clips:
    uv run scripts/acoustic_metrics.py reference=ref.wav dootdoot=render.wav

    # Corpus aggregate (pool many clips per group; reports median [p10-p90]):
    uv run scripts/acoustic_metrics.py --aggregate \
        bb8=ref1.wav bb8=ref2.wav dootdoot=render1.wav dootdoot=render2.wav

Each argument is ``LABEL=path-to-wav`` (mono 16-bit PCM WAV, any sample rate).
In ``--aggregate`` mode, every clip sharing a LABEL is pooled into one group.
"""

from __future__ import annotations

import sys
import wave
from dataclasses import dataclass, fields

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
ROLLOFF_FRACTION = 0.85
# Amplitude-tremolo search band over the frame-RMS envelope (sampled at rate/HOP).
TREMOLO_LO_HZ = 2.0
TREMOLO_HI_HZ = 20.0


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
    # Timbre / texture block.
    spectral_flatness_median: float
    spectral_rolloff_hz: float
    spectral_bandwidth_hz: float
    zero_crossing_rate: float
    spectral_flux_median: float
    onset_rate_hz: float
    tremolo_rate_hz: float
    tremolo_depth: float
    crest_factor_db: float


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


def envelope_tremolo(rms_active: np.ndarray, rate: int) -> tuple[float, float]:
    """Dominant amplitude-modulation rate (Hz) and depth from the RMS envelope.

    The envelope is sampled at ``rate / HOP``. We look for a dominant spectral
    peak in the 2-20 Hz band (BB-8's audible warble lives here). Depth is the
    modulation index of that peak relative to the envelope mean.
    """
    if rms_active.size < 8:
        return 0.0, 0.0
    envelope_rate = rate / HOP
    detrended = rms_active - rms_active.mean()
    if not np.any(detrended):
        return 0.0, 0.0
    window = np.hanning(detrended.size)
    spectrum = np.abs(rfft(detrended * window))
    freqs = rfftfreq(detrended.size, 1.0 / envelope_rate)
    band = (freqs >= TREMOLO_LO_HZ) & (freqs <= TREMOLO_HI_HZ)
    if not np.any(band):
        return 0.0, 0.0
    band_spectrum = spectrum[band]
    band_freqs = freqs[band]
    peak = int(np.argmax(band_spectrum))
    rate_hz = float(band_freqs[peak])
    # Modulation depth: amplitude of the dominant AM component / mean level.
    # rfft amplitude -> physical amplitude needs *2/N (window-corrected loosely).
    amplitude = float(band_spectrum[peak]) * 2.0 / detrended.size
    mean_level = float(rms_active.mean()) + 1e-12
    depth = amplitude / mean_level
    return rate_hz, depth


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
    flatness = []
    rolloff = []
    bandwidth = []
    zcr = []
    magnitudes = []

    for start in range(0, len(samples) - FRAME + 1, HOP):
        raw = samples[start : start + FRAME]
        segment = raw * window
        rms.append(float(np.sqrt(np.mean(segment**2))))
        spectrum = np.abs(rfft(segment))
        magnitudes.append(spectrum)
        power = spectrum**2
        total = float(power.sum()) + 1e-12
        dominant.append(float(frequencies[peak_indices[np.argmax(spectrum[peak_band])]]))
        centroid_hz = float((frequencies * power).sum() / total)
        centroid.append(centroid_hz)
        upper_share.append(float(power[upper_band].sum() / total))
        harmonic.append(harmonicity(segment, rate))
        # Spectral flatness (Wiener entropy): geo-mean / arith-mean of power.
        positive = power[1:] + 1e-12
        flatness.append(float(np.exp(np.mean(np.log(positive))) / np.mean(positive)))
        # 85% magnitude rolloff.
        cumulative = np.cumsum(spectrum)
        threshold = ROLLOFF_FRACTION * float(cumulative[-1])
        rolloff.append(float(frequencies[int(np.searchsorted(cumulative, threshold))]))
        # Power-weighted spectral spread around the centroid.
        spread = float(np.sqrt(((frequencies - centroid_hz) ** 2 * power).sum() / total))
        bandwidth.append(spread)
        # Zero-crossing rate of the un-windowed segment (crossings/sec).
        crossings = int(np.count_nonzero(np.diff(np.signbit(raw))))
        zcr.append(crossings * rate / FRAME)

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
    flatness = np.array(flatness)
    rolloff = np.array(rolloff)
    bandwidth = np.array(bandwidth)
    zcr = np.array(zcr)
    magnitudes = np.array(magnitudes)

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

    flatness_median = float(np.median(flatness[active])) if active.size else 0.0
    rolloff_median = float(np.median(rolloff[active])) if active.size else 0.0
    bandwidth_median = float(np.median(bandwidth[active])) if active.size else 0.0
    zcr_median = float(np.median(zcr[active])) if active.size else 0.0

    # Spectral flux: L2 distance between L2-normalized consecutive magnitude
    # spectra, over active transitions. A texture/animation proxy.
    flux = np.zeros(magnitudes.shape[0])
    if magnitudes.shape[0] >= 2:
        norms = np.linalg.norm(magnitudes, axis=1, keepdims=True) + 1e-12
        unit = magnitudes / norms
        flux[1:] = np.linalg.norm(np.diff(unit, axis=0), axis=1)
    active_flux = flux[active] if active.size else flux[:0]
    flux_median = float(np.median(active_flux)) if active_flux.size else 0.0

    # Onset rate: flux peaks above mean+std, per second of clip.
    onset_count = 0
    if flux.size >= 3:
        threshold = float(flux.mean() + flux.std())
        for index in range(1, flux.size - 1):
            if (
                flux[index] > threshold
                and flux[index] >= flux[index - 1]
                and flux[index] > flux[index + 1]
            ):
                onset_count += 1
    duration_s = len(samples) / rate
    onset_rate_hz = onset_count / duration_s if duration_s > 0 else 0.0

    tremolo_rate_hz, tremolo_depth = envelope_tremolo(
        rms[active] if active.size else rms[:0], rate
    )

    # Crest factor: peak sample vs RMS of active frames (dynamics proxy).
    rms_active_mean = float(rms[active].mean()) if active.size else 0.0
    peak_sample = float(np.max(np.abs(samples))) if samples.size else 0.0
    crest_factor_db = (
        20.0 * float(np.log10(peak_sample / rms_active_mean))
        if rms_active_mean > 0.0 and peak_sample > 0.0
        else 0.0
    )

    return Metrics(
        duration_s=duration_s,
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
        spectral_flatness_median=flatness_median,
        spectral_rolloff_hz=rolloff_median,
        spectral_bandwidth_hz=bandwidth_median,
        zero_crossing_rate=zcr_median,
        spectral_flux_median=flux_median,
        onset_rate_hz=onset_rate_hz,
        tremolo_rate_hz=tremolo_rate_hz,
        tremolo_depth=tremolo_depth,
        crest_factor_db=crest_factor_db,
    )


FIELDS = [
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
    ("spectral_flatness_median", "flatness_med", "{:.3f}"),
    ("spectral_rolloff_hz", "rolloff85(Hz)", "{:.0f}"),
    ("spectral_bandwidth_hz", "bandwidth(Hz)", "{:.0f}"),
    ("zero_crossing_rate", "zcr(Hz)", "{:.0f}"),
    ("spectral_flux_median", "flux_med", "{:.3f}"),
    ("onset_rate_hz", "onset_rate(Hz)", "{:.2f}"),
    ("tremolo_rate_hz", "tremolo(Hz)", "{:.1f}"),
    ("tremolo_depth", "tremolo_depth", "{:.3f}"),
    ("crest_factor_db", "crest(dB)", "{:.1f}"),
]


def print_comparison(rows: list[tuple[str, Metrics]]) -> None:
    label_width = max(len(label) for label, _ in rows)
    for attribute, title, fmt in FIELDS:
        cells = "  ".join(
            f"{label:>{label_width}}={fmt.format(getattr(metric, attribute))}"
            for label, metric in rows
        )
        print(f"{title:>18}: {cells}")


def print_aggregate(groups: dict[str, list[Metrics]]) -> None:
    numeric = [field.name for field in fields(Metrics)]
    labels = list(groups)
    column_width = max(28, *(len(label) for label in labels))
    header = "  ".join(f"{label:>{column_width}}" for label in labels)
    print(f"{'metric':>22}  {header}")
    counts = "  ".join(f"{f'n={len(groups[label])}':>{column_width}}" for label in labels)
    print(f"{'clip_count':>22}  {counts}")
    for attribute in numeric:
        cells = []
        for label in labels:
            values = np.array([getattr(metric, attribute) for metric in groups[label]])
            median = float(np.median(values))
            p10 = float(np.percentile(values, 10))
            p90 = float(np.percentile(values, 90))
            cells.append(f"{f'{median:.3g} [{p10:.3g}-{p90:.3g}]':>{column_width}}")
        print(f"{attribute:>22}  {'  '.join(cells)}")


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__)
        return 2

    aggregate = False
    if argv and argv[0] == "--aggregate":
        aggregate = True
        argv = argv[1:]

    parsed: list[tuple[str, str]] = []
    for argument in argv:
        if "=" not in argument:
            print(f"error: expected LABEL=path, got {argument!r}", file=sys.stderr)
            return 2
        label, path = argument.split("=", 1)
        parsed.append((label, path))

    if aggregate:
        groups: dict[str, list[Metrics]] = {}
        for label, path in parsed:
            groups.setdefault(label, []).append(analyze(path))
        print_aggregate(groups)
        return 0

    print_comparison([(label, analyze(path)) for label, path in parsed])
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
