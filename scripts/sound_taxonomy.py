# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "numpy>=2,<3",
#   "scipy>=1.13,<2",
# ]
# [tool.uv]
# exclude-newer = "2026-06-01T00:00:00Z"
# ///
"""Frame-by-frame BB-8 sound-event taxonomy (VOICE_V* tuning aid).

Where ``acoustic_metrics.py`` reports one aggregate vector per clip, this script
*segments* each clip into discrete sound events (gated active islands) and
classifies each event into a droid sound "type" from its frame-by-frame pitch
contour, tonality, brightness, and internal articulation. The goal is to
enumerate the *vocabulary* of distinct sounds a corpus uses, not to score a
single macro shape.

This is a directional tuning instrument, NOT part of the voice contract.

    uv run scripts/sound_taxonomy.py clip1.wav clip2.wav ...
    uv run scripts/sound_taxonomy.py --trace clip.wav      # per-event detail
    uv run scripts/sound_taxonomy.py --frames clip.wav     # raw per-frame dump

Each positional arg is a mono 16-bit PCM WAV (any sample rate).
"""

from __future__ import annotations

import sys
import wave
from dataclasses import dataclass

import numpy as np
from scipy.fft import rfft, rfftfreq
from scipy.signal import medfilt

FRAME = 2048
HOP = 256  # ~5.8 ms at 44.1 kHz: fine enough to see contour inside short events.
GATE_FRACTION = 0.10
PEAK_LO_HZ = 50.0
PEAK_HI_HZ = 8000.0
UPPER_MID_LO_HZ = 2000.0
UPPER_MID_HI_HZ = 5000.0
# Event post-processing.
MIN_EVENT_MS = 60.0  # discard / coalesce gestures shorter than this.
PITCH_JUMP_ST = 5.0  # semitone jump between settled frames marks a new gesture.
VALLEY_RATIO = 0.45  # RMS local min below this fraction of nearby peak = re-articulation.
# Classification thresholds (semitones / ratios). Directional, not contractual.
TONAL_HARMONICITY = 0.62
NOISY_HARMONICITY = 0.42
FLAT_RANGE_ST = 2.0
WIDE_SWEEP_ST = 12.0
CONTOUR_ST = 2.0
SHORT_EVENT_MS = 150.0
BRIGHT_CENTROID_HZ = 1100.0
BRIGHT_UPPER_SHARE = 0.20


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


@dataclass
class Frame:
    t_ms: float
    rms: float
    pitch_hz: float
    harmonicity: float
    centroid_hz: float
    upper_share: float
    flatness: float
    flux: float


def per_frame(samples: np.ndarray, rate: int) -> list[Frame]:
    window = np.hanning(FRAME)
    freqs = rfftfreq(FRAME, 1.0 / rate)
    peak_band = (freqs >= PEAK_LO_HZ) & (freqs <= PEAK_HI_HZ)
    upper_band = (freqs >= UPPER_MID_LO_HZ) & (freqs <= UPPER_MID_HI_HZ)
    peak_indices = np.flatnonzero(peak_band)

    mags: list[np.ndarray] = []
    rows: list[Frame] = []
    for start in range(0, len(samples) - FRAME + 1, HOP):
        raw = samples[start : start + FRAME]
        seg = raw * window
        spectrum = np.abs(rfft(seg))
        mags.append(spectrum)
        power = spectrum**2
        total = float(power.sum()) + 1e-12
        rms = float(np.sqrt(np.mean(seg**2)))
        pitch = float(freqs[peak_indices[np.argmax(spectrum[peak_band])]])
        centroid = float((freqs * power).sum() / total)
        upper = float(power[upper_band].sum() / total)
        positive = power[1:] + 1e-12
        flat = float(np.exp(np.mean(np.log(positive))) / np.mean(positive))
        rows.append(
            Frame(
                t_ms=start / rate * 1000.0,
                rms=rms,
                pitch_hz=pitch,
                harmonicity=harmonicity(seg, rate),
                centroid_hz=centroid,
                upper_share=upper,
                flatness=flat,
                flux=0.0,
            )
        )
    # Spectral flux between L2-normalized adjacent spectra.
    if len(mags) >= 2:
        stack = np.array(mags)
        unit = stack / (np.linalg.norm(stack, axis=1, keepdims=True) + 1e-12)
        flux = np.concatenate([[0.0], np.linalg.norm(np.diff(unit, axis=0), axis=1)])
        for row, value in zip(rows, flux):
            row.flux = float(value)
    # Median-filter the raw peak-pitch track to reject single-frame octave
    # errors from the simple argmax tracker (MP3 + formant-heavy audio is noisy).
    if len(rows) >= 5:
        smoothed = medfilt(np.array([r.pitch_hz for r in rows]), kernel_size=5)
        for row, value in zip(rows, smoothed):
            row.pitch_hz = float(value)
    return rows


@dataclass
class Event:
    start_ms: float
    end_ms: float
    frames: list[Frame]

    @property
    def duration_ms(self) -> float:
        return self.end_ms - self.start_ms


def segment(rows: list[Frame]) -> list[Event]:
    """Split into single-gesture events.

    BB-8 chains gestures legato (continuous energy, abrupt pitch jumps), so RMS
    gating alone fuses a whole phrase into one island. We therefore gate to find
    active regions, then sub-split each region at gesture boundaries:
      * onset spikes (spectral-flux local maxima above mean+std), and
      * pitch discontinuities (>= PITCH_JUMP_ST between settled adjacent frames),
      * deep RMS valleys (a local minimum well below its neighboring peaks).
    Boundaries closer than MIN_EVENT_MS are coalesced.
    """
    if not rows:
        return []
    rms = np.array([f.rms for f in rows])
    pitch = np.array([f.pitch_hz for f in rows])
    flux = np.array([f.flux for f in rows])
    harm = np.array([f.harmonicity for f in rows])
    gate = GATE_FRACTION * rms.max()
    active = rms >= gate

    # Active runs (silence-separated islands).
    runs: list[tuple[int, int]] = []
    run_start = None
    for i, flag in enumerate(active):
        if flag and run_start is None:
            run_start = i
        elif not flag and run_start is not None:
            runs.append((run_start, i - 1))
            run_start = None
    if run_start is not None:
        runs.append((run_start, len(active) - 1))
    if not runs:
        return []

    min_gap = max(1, int(MIN_EVENT_MS / 1000.0 * rate_global / HOP))
    events: list[Event] = []
    for s, e in runs:
        if e - s + 1 < 3:
            continue
        cuts = {s, e + 1}
        seg_flux = flux[s : e + 1]
        thr = float(seg_flux.mean() + seg_flux.std())
        for i in range(s + 1, e):
            # Onset spike.
            if flux[i] > thr and flux[i] >= flux[i - 1] and flux[i] > flux[i + 1]:
                cuts.add(i)
            # Pitch discontinuity between settled (tonal) frames.
            elif (
                harm[i] > 0.4
                and harm[i - 1] > 0.4
                and abs(hz_to_st(pitch[i - 1], pitch[i])) >= PITCH_JUMP_ST
            ):
                cuts.add(i)
            # Deep RMS valley (re-articulation without going silent).
            elif (
                rms[i] < rms[i - 1]
                and rms[i] <= rms[i + 1]
                and rms[i] < VALLEY_RATIO * max(rms[max(s, i - 4) : i + 5].max(), 1e-9)
            ):
                cuts.add(i)
        ordered = sorted(cuts)
        # Coalesce boundaries that are too close together.
        kept = [ordered[0]]
        for c in ordered[1:]:
            if c - kept[-1] >= min_gap:
                kept.append(c)
        if kept[-1] != e + 1:
            kept[-1] = e + 1
        for a, b in zip(kept[:-1], kept[1:]):
            frames = rows[a:b]
            if len(frames) < 2:
                continue
            ev = Event(start_ms=frames[0].t_ms, end_ms=frames[-1].t_ms, frames=frames)
            if ev.duration_ms >= MIN_EVENT_MS:
                events.append(ev)
    return events


def hz_to_st(a: float, b: float) -> float:
    """Interval in semitones from a to b (signed)."""
    if a <= 0 or b <= 0:
        return 0.0
    return 12.0 * float(np.log2(b / a))


@dataclass
class EventFeatures:
    duration_ms: float
    pitch_med_hz: float
    pitch_lo_hz: float
    pitch_hi_hz: float
    range_st: float
    contour: str  # rise / fall / arch / dip / flat / wide-sweep / wander
    net_st: float  # signed start->end
    harmonicity_med: float
    tonality: str  # tonal / mixed / noisy
    centroid_hz: float
    upper_share_max: float
    bright: bool
    tremolo_hz: float
    tremolo_depth: float
    onsets: int  # internal flux-peak articulations
    label: str


def event_tremolo(rms: np.ndarray) -> tuple[float, float]:
    if rms.size < 8:
        return 0.0, 0.0
    env_rate = rate_global / HOP
    detr = rms - rms.mean()
    if not np.any(detr):
        return 0.0, 0.0
    win = np.hanning(detr.size)
    spec = np.abs(rfft(detr * win))
    freqs = rfftfreq(detr.size, 1.0 / env_rate)
    band = (freqs >= 2.0) & (freqs <= 30.0)
    if not np.any(band):
        return 0.0, 0.0
    bf, bs = freqs[band], spec[band]
    peak = int(np.argmax(bs))
    rate_hz = float(bf[peak])
    amp = float(bs[peak]) * 2.0 / detr.size
    depth = amp / (float(rms.mean()) + 1e-12)
    return rate_hz, depth


def classify(ev: Event) -> EventFeatures:
    frames = ev.frames
    pitch = np.array([f.pitch_hz for f in frames])
    harm = np.array([f.harmonicity for f in frames])
    rms = np.array([f.rms for f in frames])
    centroid = np.array([f.centroid_hz for f in frames])
    upper = np.array([f.upper_share for f in frames])
    flux = np.array([f.flux for f in frames])

    # Robust pitch endpoints over the strong-tonal portion of the event.
    tonal_mask = harm >= 0.45
    track = pitch[tonal_mask] if tonal_mask.sum() >= 3 else pitch
    n = len(frames)
    head = max(1, n // 5)
    p_start = float(np.median(pitch[:head]))
    p_end = float(np.median(pitch[-head:]))
    p_mid = float(np.median(pitch[head:-head])) if n > 2 * head else float(np.median(pitch))
    p_lo, p_hi = float(track.min()), float(track.max())
    p_med = float(np.median(track))
    range_st = hz_to_st(p_lo, p_hi)
    net_st = hz_to_st(p_start, p_end)

    # Contour classification.
    rise_mid = hz_to_st(p_start, p_mid)
    fall_mid = hz_to_st(p_mid, p_end)
    if range_st >= WIDE_SWEEP_ST:
        contour = "wide-sweep"
    elif abs(net_st) < FLAT_RANGE_ST and range_st < FLAT_RANGE_ST:
        contour = "flat"
    elif net_st >= CONTOUR_ST and rise_mid >= 0 and fall_mid >= -1.0:
        contour = "rise"
    elif net_st <= -CONTOUR_ST and rise_mid <= 1.0 and fall_mid <= 0:
        contour = "fall"
    elif rise_mid >= CONTOUR_ST and fall_mid <= -CONTOUR_ST:
        contour = "arch"
    elif rise_mid <= -CONTOUR_ST and fall_mid >= CONTOUR_ST:
        contour = "dip"
    elif net_st >= CONTOUR_ST:
        contour = "rise"
    elif net_st <= -CONTOUR_ST:
        contour = "fall"
    else:
        contour = "wander"

    harm_med = float(np.median(harm))
    if harm_med >= TONAL_HARMONICITY:
        tonality = "tonal"
    elif harm_med <= NOISY_HARMONICITY:
        tonality = "noisy"
    else:
        tonality = "mixed"

    cen_med = float(np.median(centroid))
    upper_max = float(upper.max())
    bright = cen_med >= BRIGHT_CENTROID_HZ or upper_max >= BRIGHT_UPPER_SHARE

    trem_hz, trem_depth = event_tremolo(rms)

    # Internal articulations: flux peaks above mean+std inside the event.
    onsets = 0
    if flux.size >= 3:
        thr = float(flux.mean() + flux.std())
        for i in range(1, flux.size - 1):
            if flux[i] > thr and flux[i] >= flux[i - 1] and flux[i] > flux[i + 1]:
                onsets += 1

    dur = ev.duration_ms
    onset_density = onsets / (dur / 1000.0) if dur > 0 else 0.0

    label = _label(
        dur, contour, tonality, bright, range_st, net_st, p_med, trem_depth, onset_density
    )

    return EventFeatures(
        duration_ms=dur,
        pitch_med_hz=p_med,
        pitch_lo_hz=p_lo,
        pitch_hi_hz=p_hi,
        range_st=range_st,
        contour=contour,
        net_st=net_st,
        harmonicity_med=harm_med,
        tonality=tonality,
        centroid_hz=cen_med,
        upper_share_max=upper_max,
        bright=bright,
        tremolo_hz=trem_hz,
        tremolo_depth=trem_depth,
        onsets=onsets,
        label=label,
    )


def _label(
    dur: float,
    contour: str,
    tonality: str,
    bright: bool,
    range_st: float,
    net_st: float,
    p_med: float,
    trem_depth: float,
    onset_density: float,
) -> str:
    """Heuristic droid sound-type label from the extracted features."""
    # Noise-dominant events first.
    if tonality == "noisy":
        if dur < SHORT_EVENT_MS:
            return "noise-burst"
        return "buzz/growl" if p_med < 600 else "rough-squawk"
    # Strongly articulated runs read as trills / chatter regardless of contour.
    if onset_density >= 7.0 and dur >= SHORT_EVENT_MS:
        return "trill/chatter"
    # Wide glides.
    if contour == "wide-sweep":
        if net_st > 0:
            return "rising-whistle" if (bright or p_med > 1200) else "rising-swoop"
        if net_st < 0:
            return "falling-whistle" if (bright or p_med > 1200) else "falling-swoop"
        return "wide-warble"
    # Short events = blips/chirps.
    if dur < SHORT_EVENT_MS:
        if contour == "rise":
            return "chirp-up"
        if contour == "fall":
            return "chirp-down"
        return "blip"
    # Sustained events.
    if contour == "rise":
        return "rising-tone"
    if contour == "fall":
        return "falling-tone"
    if contour == "arch":
        return "arch-coo"
    if contour == "dip":
        return "dip-tone"
    # Flat / wander sustained: warble vs steady.
    if trem_depth >= 0.25 or tonality == "mixed":
        return "warble-tone"
    return "steady-tone"


# Mutable module globals set per file (keeps signatures terse in this aid script).
rate_global = 44100


def analyze_file(path: str) -> tuple[list[Event], list[EventFeatures], list[Frame]]:
    global rate_global
    samples, rate = read_wav_mono(path)
    rate_global = rate
    rows = per_frame(samples, rate)
    events = segment(rows)
    feats = [classify(ev) for ev in events]
    return events, feats, rows


def short_name(path: str) -> str:
    base = path.rsplit("/", 1)[-1]
    return base[:-4] if base.endswith(".wav") else base


def print_summary(path: str, events: list[Event], feats: list[EventFeatures]) -> None:
    name = short_name(path)
    total = sum(f.duration_ms for f in feats)
    print(f"\n=== {name}  ({len(feats)} events, {total / 1000:.2f}s active) ===")
    header = (
        f"{'#':>2} {'start':>6} {'dur':>5} {'pitch':>6} {'range':>6} "
        f"{'net':>5} {'contour':>11} {'tonality':>7} {'br':>2} "
        f"{'trem':>5} {'ons':>3}  label"
    )
    print(header)
    for i, (ev, f) in enumerate(zip(events, feats)):
        print(
            f"{i:>2} {ev.start_ms:>6.0f} {f.duration_ms:>5.0f} "
            f"{f.pitch_med_hz:>6.0f} {f.range_st:>5.1f}st {f.net_st:>+5.1f} "
            f"{f.contour:>11} {f.tonality:>7} {'Y' if f.bright else '.':>2} "
            f"{f.tremolo_depth:>5.2f} {f.onsets:>3}  {f.label}"
        )


def print_frames(path: str, rows: list[Frame]) -> None:
    print(f"\n=== {short_name(path)} per-frame ===")
    print(f"{'t_ms':>7} {'rms':>7} {'pitch':>6} {'harm':>5} {'cen':>6} {'upmid':>6} {'flux':>5}")
    peak = max((r.rms for r in rows), default=1.0) or 1.0
    for r in rows:
        if r.rms < 0.04 * peak:
            continue
        print(
            f"{r.t_ms:>7.0f} {r.rms:>7.4f} {r.pitch_hz:>6.0f} {r.harmonicity:>5.2f} "
            f"{r.centroid_hz:>6.0f} {r.upper_share:>6.3f} {r.flux:>5.2f}"
        )


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__)
        return 2
    mode = "summary"
    if argv[0] in ("--trace", "--frames"):
        mode = argv[0][2:]
        argv = argv[1:]
    if not argv:
        print("error: no WAV paths given", file=sys.stderr)
        return 2

    corpus_counts: dict[str, int] = {}
    all_feats: list[EventFeatures] = []
    for path in argv:
        events, feats, rows = analyze_file(path)
        if mode == "frames":
            print_frames(path, rows)
        print_summary(path, events, feats)
        all_feats.extend(feats)
        for f in feats:
            corpus_counts[f.label] = corpus_counts.get(f.label, 0) + 1

    print("\n=== corpus sound-type tally ===")
    for label, count in sorted(corpus_counts.items(), key=lambda kv: -kv[1]):
        print(f"{count:>4}  {label}")

    print_profile(all_feats)
    return 0


def print_profile(feats: list[EventFeatures]) -> None:
    """Aggregate vocabulary profile across every event analyzed this run."""
    if not feats:
        return
    n = len(feats)
    rising = sum(1 for f in feats if f.net_st >= CONTOUR_ST)
    falling = sum(1 for f in feats if f.net_st <= -CONTOUR_ST)
    flatish = n - rising - falling
    tonal = sum(1 for f in feats if f.tonality == "tonal")
    mixed = sum(1 for f in feats if f.tonality == "mixed")
    noisy = sum(1 for f in feats if f.tonality == "noisy")
    bright = sum(1 for f in feats if f.bright)
    wide = sum(1 for f in feats if f.range_st >= WIDE_SWEEP_ST)
    durs = np.array([f.duration_ms for f in feats])
    ranges = np.array([f.range_st for f in feats])
    hi = np.array([f.pitch_hi_hz for f in feats])
    med = np.array([f.pitch_med_hz for f in feats])

    def pct(x: int) -> str:
        return f"{100.0 * x / n:4.0f}%"

    print("\n=== vocabulary profile ===")
    print(f"  events                : {n}")
    print(f"  contour   rise/fall/flat: {pct(rising)} / {pct(falling)} / {pct(flatish)}")
    print(f"  tonality tonal/mix/noisy: {pct(tonal)} / {pct(mixed)} / {pct(noisy)}")
    print(f"  bright (>=1.1kHz or upmid): {pct(bright)}")
    print(f"  wide-sweep (>=12 st)    : {pct(wide)}")
    print(f"  gesture dur  ms  med/p90 : {np.median(durs):.0f} / {np.percentile(durs, 90):.0f}")
    print(f"  pitch range st  med/p90  : {np.median(ranges):.1f} / {np.percentile(ranges, 90):.1f}")
    print(f"  pitch med  Hz   med/p90  : {np.median(med):.0f} / {np.percentile(med, 90):.0f}")
    print(f"  pitch peak Hz   med/p90  : {np.median(hi):.0f} / {np.percentile(hi, 90):.0f}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
