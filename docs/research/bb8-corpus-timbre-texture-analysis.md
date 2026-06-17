# BB-8 Corpus Timbre & Texture Analysis (VOICE_V7)

> Status: **research / directional**. This note compares the **whole** local BB-8
> reference corpus of source recordings against the current
> `VOICE_V7` output, with a focus on **timbre and texture** rather than the single-clip
> macro-shape that earlier gap analyses studied.
>
> It does not change the voice contract. Every recommendation here is sample-affecting and
> would require a new voice version (`V8`) plus regenerated golden WAV hashes.
>
> It supersedes the framing — not the measurements — of
> [`bb8-inquisitive-chatty-gap-analysis.md`](./bb8-inquisitive-chatty-gap-analysis.md),
> which measured `VOICE_V6` against one clip. Many of that document's recommendations
> (raised pause ceiling, dash/ellipsis prosody, whistle sweep, noise/breath blend, mouth
> stage, performance planner) **shipped in `VOICE_V7`**. This note measures what that
> bought, corpus-wide, and where the residual gap actually is.

## Summary

`VOICE_V7` is much closer to BB-8 than the `V6` analysis would predict, and the headline
correction is this: **the gestures the earlier analysis asked for now exist and engage —
but engagement is gated by punctuation and affect, not inferred from neutral text.** A
plain declarative phrase renders flat, over-connected, and uniformly pure-toned; the same
content with sentence punctuation and exclamation/question marks engages V7's whistle,
roughness, brightness-burst, pause, and staging gestures and lands inside the BB-8 corpus
distribution on most timbre/texture axes.

Three things separate dootdoot from the corpus today:

1. **Neutral input stays flat (the dominant practical gap).** Without internal punctuation,
   dootdoot fuses a phrase into one long voiced island (active fraction ~0.92, median
   island ~900 ms, ~0 ms internal silence) with harmonicity pinned at ~0.95. BB-8's
   isolated library clips sit at active fraction ~0.45 with ~230 ms events. The expressive
   range is real but **locked behind punctuation/affect**, so ordinary text input does not
   sound BB-8-like.
2. **High-register reach is still modest, and brightness is constant rather than bursty.**
   Even on the most expressive renders, only 6–13% of active frames clear the old ~1.1 kHz
   ceiling, versus 22–34% for bright BB-8 clips. dootdoot keeps a steady 2–5 kHz layer
   present (median share ~0.058 across all renders) where BB-8 keeps that band near silent
   and then bursts it (max share up to ~0.81–0.97). The brightness _level_ now matches; the
   brightness _behavior_ (rare, extreme spark over a dark body) does not.
3. **Frame-to-frame animation is lower on neutral input.** Median spectral flux is 0.24 vs
   the corpus's 0.32–0.38 — the spectrum moves less. This collapses on expressive input
   (flux 0.46–0.60, equal to or above BB-8), so it is the same engagement-gating problem as
   #1 seen on a different axis.

And three things `VOICE_V7` genuinely matches:

- **Warble.** Tremolo rate (~3.8 Hz) and depth (~0.26) land inside the BB-8 pocket
  (rate ~2–4 Hz, depth ~0.1–0.32). The multi-LFO warble is doing its job.
- **Brightness level.** Spectral centroid (~650 Hz) and 85% rolloff (~3.7 kHz) sit between
  the two BB-8 sub-corpora. The `V6` "too bright" reading was a single-clip artifact, not a
  corpus-level truth.
- **Staging, when punctuation is present.** The raised pause ceiling and dash/ellipsis
  prosody work: `"hello. what? wait... no!"` renders as 4 islands, active fraction 0.26, and
  a **964 ms** internal pause — squarely BB-8-like, and impossible under `V6`'s ~240 ms
  ceiling.

The next step is therefore not "add more primitives" (as the V6 note concluded) but
**"engage the primitives V7 already has from neutral semantics, push the high register
harder, and make the upper-mid layer bursty instead of constant."**

## Method

The corpus splits into two acoustically distinct sub-corpora, analyzed separately:

- **`bb8-lib` (32 clips):** the numbered isolated sound effects (`bb8-01.mp3` …
  `bb8-32.mp3`, plus `bb2-02.mp3`). Mostly single short gestures.
- **`bb8-ctx` (7 clips):** the `bb8-clips/` performance excerpts cut from the supercut
  (alarm, anxious, surprise, sad, excited, found/fixed, inquisitive-then-chatty). Continuous
  performed exchanges.

Decode + render + the existing band/timing report are produced by `scripts/bb8-metrics`
(decodes every MP3 to mono 44.1 kHz 16-bit, renders a phrase corpus via `xtask bb8-metrics`,
writes `target/bb8-metrics/bb8-metrics.md`). The **timbre/texture** block below comes from
the expanded `scripts/acoustic_metrics.py` (driven by `scripts/acoustics`), run in
`--aggregate` mode to pool clips per group and report `median [p10–p90]`:

```bash
scripts/bb8-metrics <reference-recordings-dir>    # decode corpus + render baseline phrase set

# 15 emotionally-varied dootdoot renders mirroring the clip contexts:
#   target/bb8-metrics/dootdoot-expanded/*.wav
uv run scripts/acoustic_metrics.py --aggregate \
  bb8-lib=target/bb8-metrics/reference-wav/*.wav \
  bb8-ctx=target/bb8-metrics/contextual-wav/*.wav \
  dootdoot=target/bb8-metrics/dootdoot-expanded/*.wav
```

### What the expanded metrics measure

`acoustic_metrics.py` gained a timbre/texture block (additive; the existing directional
fields and their formulas are unchanged, so prior single-clip notes still reproduce):

| Metric                              | What it captures                                       | Why it matters for BB-8                                |
| ----------------------------------- | ------------------------------------------------------ | ------------------------------------------------------ |
| `spectral_flatness_median`          | Wiener entropy: tonal (→0) vs noisy (→1)               | Separates pitched tone from breath/noise texture       |
| `spectral_rolloff_hz`               | 85% magnitude rolloff                                  | Brightness edge, robust to a single bright bin         |
| `spectral_bandwidth_hz`             | Power-weighted spread around the centroid              | Timbral width — thin tone vs full-bodied buzz          |
| `zero_crossing_rate`                | Time-domain crossings/sec                              | HF/noise content cross-check on centroid               |
| `spectral_flux_median`              | L2 change between L2-normalized adjacent spectra       | **Animation / liveliness** — how much the timbre moves |
| `onset_rate_hz`                     | Flux-peak onsets per second                            | Articulation density (chatter vs sustained tone)       |
| `tremolo_rate_hz` / `tremolo_depth` | Dominant 2–20 Hz AM of the RMS envelope, and its depth | **Warble** — the signature BB-8 amplitude wobble       |
| `crest_factor_db`                   | Peak vs active-frame RMS                               | Dynamics — peaky transients vs flat sustain            |

These are directional tuning instruments, not acceptance constants. MP3-sourced references,
formant-heavy droid audio, and simple F0/peak tracking all add noise; the value is in the
large, consistent differences, not the third significant figure.

## Corpus measurements

`median [p10–p90]` across each group. dootdoot = 15 emotionally-varied renders.

| metric                   | bb8-lib (n=32)       | bb8-ctx (n=7)       | dootdoot (n=15)         | Read                                                                 |
| ------------------------ | -------------------- | ------------------- | ----------------------- | -------------------------------------------------------------------- |
| duration (s)             | 1.52 [0.84–2.6]      | 3.0 [1.0–3.0]       | 1.18 [0.88–2.1]         | comparable spans                                                     |
| active fraction          | **0.45 [0.29–0.54]** | 0.99 [0.68–1.0]     | **0.92 [0.82–0.95]**    | dootdoot fills the file like a performance, not like an isolated SFX |
| active islands           | 2 [1–6]              | 4 [1–7]             | **1 [1–4]**             | dootdoot fuses into one island on neutral input                      |
| median island (ms)       | 232 [110–463]        | 389 [158–964]       | **911 [279–1350]**      | dootdoot events are far longer / over-connected                      |
| max internal gap (ms)    | 64 [0–447]           | 12 [0–511]          | **0 [0–91]**            | neutral dootdoot rarely goes truly silent mid-clip                   |
| dominant-peak range (Hz) | 1010 [345–5410]      | 2610 [332–7010]     | **431 [332–534]**       | neutral dootdoot peak barely moves                                   |
| frac > 1.1 kHz           | 0.04 [0–0.41]        | 0.18 [0–0.28]       | **0 [0–0]**             | neutral dootdoot never reaches whistle register                      |
| harmonicity median       | 0.80 [0.61–0.87]     | 0.79 [0.64–0.87]    | **0.95 [0.91–0.96]**    | dootdoot is much more purely periodic                                |
| harmonicity IQR          | 0.19 [0.12–0.32]     | 0.27 [0.19–0.32]    | **0.04 [0.02–0.09]**    | dootdoot does not swing clean↔rough on neutral input                 |
| centroid (Hz)            | 846 [473–1760]       | 619 [203–838]       | 647 [621–731]           | brightness level is well-matched                                     |
| 2–5 kHz share, median    | 0.033 [0.003–0.16]   | 0.003 [0.001–0.013] | **0.058 [0.036–0.079]** | dootdoot keeps upper-mid constantly present                          |
| 2–5 kHz share, max       | 0.31 [0.08–0.93]     | 0.81 [0.02–0.86]    | **0.16 [0.09–0.26]**    | BB-8 bursts upper-mid far harder than dootdoot                       |
| spectral flatness        | ~1e-4                | ~2e-5               | ~7e-5                   | all strongly tonal; not a discriminator (see caveat)                 |
| rolloff 85% (Hz)         | 4550 [2110–6490]     | 1670 [1290–2620]    | 3690 [3550–3940]        | dootdoot sits between the sub-corpora                                |
| bandwidth (Hz)           | 777 [427–1260]       | 307 [212–507]       | 741 [643–875]           | full-bodied like the library                                         |
| ZCR (Hz)                 | 2540 [1140–4740]     | 1550 [560–1940]     | 1460 [1300–1680]        | dootdoot is slightly lower-noise                                     |
| spectral flux            | 0.32 [0.23–0.45]     | 0.38 [0.28–0.42]    | **0.24 [0.21–0.30]**    | neutral dootdoot is less animated                                    |
| onset rate (Hz)          | 6.5 [4.1–8.4]        | 9.0 [7.8–10.7]      | 6.1 [5.3–6.8]           | fewer articulations than chatty BB-8                                 |
| tremolo rate (Hz)        | 3.7 [2.4–7.8]        | 2.1 [2.1–4.8]       | 3.8 [3.2–4.2]           | **warble rate matched**                                              |
| tremolo depth            | 0.32 [0.13–0.37]     | 0.12 [0.09–0.19]    | 0.26 [0.20–0.34]        | **warble depth in range**                                            |
| crest factor (dB)        | 20.2 [16.9–22.5]     | 18.5 [17.2–20.3]    | 21.8 [20.2–23.1]        | dynamics comparable (slightly peakier)                               |

The two BB-8 sub-corpora are themselves very different, and worth holding as two distinct
targets: the **library** is sparse, bright, wide-ranging single gestures (active 0.45,
centroid 846 Hz, rolloff 4.5 kHz, peak range >1 kHz); the **performance** clips are dense,
dark-bodied, with near-silent upper-mid that bursts hard (active 0.99, centroid 619 Hz,
2–5 kHz median 0.003 but max 0.81). dootdoot currently reads as a _third_ thing: a
continuously voiced, uniformly periodic, modestly-but-constantly bright tone.

## The reframe: engagement is gated by punctuation and affect

The flat corpus aggregate above is dominated by neutral, lightly-punctuated phrases. When
the input carries strong terminal punctuation, V7's expressive gestures fire and the
timbre/texture profile moves decisively toward BB-8. Per-clip comparison of two bright BB-8
references against two punctuated dootdoot renders:

| metric                   | bb8-09 (lib) | found-fixed (ctx) | dd `what was that?!` | dd `wheeee so excited!` |
| ------------------------ | -----------: | ----------------: | -------------------: | ----------------------: |
| dominant-peak range (Hz) |         3682 |              2606 |             **2433** |                **2885** |
| frac > 1.1 kHz           |         0.22 |              0.34 |             **0.06** |                **0.13** |
| harmonicity IQR          |        0.177 |             0.281 |            **0.270** |               **0.297** |
| 2–5 kHz share, max       |        0.970 |             0.860 |            **0.899** |               **0.941** |
| spectral flux            |        0.327 |             0.366 |            **0.456** |               **0.602** |
| onset rate (Hz)          |         9.11 |             10.33 |            **10.67** |                **8.28** |

On these inputs dootdoot's roughness swing (harmonicity IQR), upper-mid bursting, and
animation are **inside or above** the BB-8 range. The whistle/flourish even lifts the
dominant peak above the old ceiling (6–13% of frames, vs 0% for neutral input). The
machinery the V6 note asked for is present and working.

Staging responds the same way. `"hello. what? wait... no!"` renders as:

```text
active fraction 0.26   islands 4   median island 157 ms   max internal gap 964 ms
```

That is the BB-8 library's sparsity and a ~1 s conversational pause — both impossible under
`V6`. The raised pause ceiling, dash/ellipsis prosody, and per-clause islanding all work.

## Where the residual gap actually is

1. **Neutral semantics do not engage the gestures.** This is the dominant practical
   difference. Plain text ("hello there", "playing music and dancing", "I do not want to be
   left behind") produces one long over-connected island with no roughness swing, no
   high-register motion, and constant modest brightness. The performance planner localizes
   affect when punctuation/markers exist, but flat declarative text saturates into a single
   color. Most real input is flat declarative text, so most output does not sound BB-8-like.
2. **High-register reach is modest and not bursty.** Even the best expressive renders clear
   1.1 kHz on only 6–13% of frames (BB-8: 22–34%), and the 2–5 kHz layer is _always_
   present at ~0.058 rather than near-zero with occasional ~0.8–0.97 bursts. The brightness
   level is right; the brightness _behavior_ — rare, extreme spark over an otherwise dark
   body — is not. This matches the V6 "wrong kind of brightness" finding, now narrowed: it
   is a constant-vs-bursty problem plus an under-reaching whistle, not an overall level
   problem.
3. **Roughness floor is too clean.** Neutral harmonicity sits at ~0.95 (IQR ~0.04). The
   noise/breath blend exists but only swings on engaged gestures. Even a small constant
   roughening of the neutral body would close distance to BB-8's ~0.80 median.

## Recommendations (directional; a future `VOICE_V8`)

1. **Engage expressive gestures from neutral semantics, not just punctuation.** Let the
   performance planner derive local arousal/role contrast from the semantic PCA axes and
   word-to-word movement so that an unpunctuated phrase still gets a probe/reply/flourish
   shape, internal rests, and a roughness/whistle accent. This is the single highest-leverage
   change and needs no new primitives — only wider, semantics-driven engagement of the
   curves V7 already drives.
2. **Make the upper-mid layer bursty, not constant.** Lower the default 2–5 kHz mix toward
   the BB-8 median-near-zero, and reallocate that energy into short, high-contrast bursts on
   accents (target max share ~0.6–0.9, as the corpus shows). Brightness should have an
   envelope, not a floor.
3. **Push the whistle harder and more often on accents.** Raise the proportion of accent
   frames that clear ~1.1 kHz toward the BB-8 0.2–0.3 range, and let selected flourishes
   sweep the dominant peak further into the 2–4 kHz region. The capability exists; it is
   under-deployed in frequency and frequency-of-use.
4. **Add a small constant roughness to the neutral body.** A subtle always-on noise/breath
   or filter-mismatch term would move neutral harmonicity off ~0.95 toward BB-8's ~0.80
   without needing a gesture to fire.
5. **Reduce neutral over-connection.** Allow short (30–80 ms) inter-word rests on neutral
   multi-word input so active fraction can fall from ~0.92 toward the library's ~0.45,
   independent of punctuation.

## Non-recommendations

- **Do not lower overall brightness.** Centroid and rolloff already match; the problem is
  the _shape_ of the upper-mid energy, not its average level.
- **Do not touch the warble.** Tremolo rate and depth are already in the BB-8 pocket.
- **Do not chase spectral flatness.** Every group is strongly tonal (flatness ~1e-4 to
  ~2e-5); it does not discriminate here. Harmonicity median/IQR is the better roughness
  instrument.
- **Do not add nondeterministic randomness, a speech vocoder, or sample libraries.** The
  residual gap is engagement and shaping of existing deterministic primitives, not a missing
  instrument.

## Caveats

- References are MP3-decoded sound-effect clips, not lab-isolated tones; absolute values
  carry MP3 and reverb coloration. Conclusions rest on large, consistent cross-group
  differences.
- `tremolo_*` reads the dominant 2–20 Hz AM of the frame-RMS envelope; on multi-event clips
  it reports the strongest single rate, not a full modulation spectrum.
- `onset_rate` and `spectral_flux` are gate- and normalization-dependent and should be read
  relatively between groups computed the same way, never as absolute thresholds.
- The dootdoot phrase set is a 15-phrase hand-built sample chosen to mirror the clip
  contexts; it is directional, not exhaustive.
