# VOICE_V7 contextual performance acceptance

Status: **Accepted for VOICE_V7**.

`VOICE_V7` adds contextual performance, expanded synthesis dynamic range, and mouth
articulation on top of the `VOICE_V6` connected-phrase voice. It keeps the four PCA
semantic axes as the learnable core and the V2–V6 performance channels.

## Scope

- `dootdoot --version` reports `dootdoot VOICE_V7`.
- The embedded semantic mapping remains the locked token-to-axis table in
  `assets/dootdoot_asset_v1.doot`.
- New deterministic, bounded primitives: a swept-oscillator whistle gesture and wider
  per-gesture pitch span, a noise/breath excitation blend, role-gated long pauses with
  suppressible word-boundary bridging, dash/ellipsis hesitation markers, and a gated
  code-talkbox mouth stage.
- A deterministic discourse-performance planner assigns local phrase roles (`probe`,
  `chatty_reply`, `hesitation`, `terminal_flourish`, `aside`) and continuous performance
  curves, deploying the primitives by role. Affect and archetype are localized so a
  high-arousal utterance no longer collapses into one global Yelp; whistle/yelp are
  reserved for the opener and terminal accent.
- The standalone dash is **control-only**: it no longer appears with four-axis values in
  `--explain`, and `--explain` surfaces a `role:` annotation per voiced token.
- Neutral-curve rendering (the empty chirp and hand-built events) is byte-identical to
  `VOICE_V6`; only the planner-driven engine path changes samples.
- The committed golden WAV hashes were regenerated under `VOICE_V7`.

## Directional Check

Reference clip: `inquisitive-then-chatty.mp3`. Dootdoot phrase:
`Hello - good morning Sandile. What are you doing today?!`. Both decoded to mono 44.1 kHz;
2048-sample Hann frames, 512 hop, active frames gated at 8% of each clip's peak frame RMS;
dominant spectral peak and an autocorrelation harmonicity proxy tracked per frame. The
numbers are produced by the locked uv harness and reproduced with:

```bash
scripts/acoustics \
  "Hello - good morning Sandile. What are you doing today?!" \
  /path/to/bb8-clips/inquisitive-then-chatty.mp3
```

The `VOICE_V6` column reproduces the figures from
[`bb8-inquisitive-chatty-gap-analysis.md`](../research/bb8-inquisitive-chatty-gap-analysis.md).

| Measurement                | Reference | VOICE_V6 | VOICE_V7 | Read                                                             |
| -------------------------- | --------: | -------: | -------: | ---------------------------------------------------------------- |
| Active frame fraction      |      0.44 |     0.63 |     0.57 | V7 leaves more rests; closer to staged ref                       |
| Max internal gap           |   1138 ms |   232 ms |   929 ms | V7 opens a real turn gap (was capped)                            |
| Dominant peak range        |   4264 Hz |   517 Hz |  2649 Hz | V7 can now sweep, not stuck low                                  |
| Fraction of frames >1.1kHz |      0.18 |     0.00 |     0.03 | V7 reaches whistle range on accents                              |
| Harmonicity IQR            |     0.226 |    0.050 |    0.159 | V7 noise/breath lets harmonicity swing                           |
| Spectral centroid, median  |    618 Hz |  2214 Hz |   879 Hz | V7 dropped the constant brightness floor                         |
| 2–5 kHz share, max         |     0.834 |    0.366 |    0.920 | V7 has rare, extreme upper-mid bursts                            |
| 2–5 kHz share, median      |     0.000 |    0.104 |    0.113 | residual floor; brightness is event-shaped but not lower overall |

The three co-primary gaps move in the right direction: the question-to-answer **timing**
gap widens from 232 ms to 929 ms, the **tonal pitch range** widens roughly 5× and now
crosses the old ~1.1 kHz ceiling on terminal-flourish accents, and the **excitation** gains
an authored noise/breath path so the harmonicity IQR widens from 0.050 toward the reference's
0.226. The global spectral centroid also falls sharply (2214 Hz → 879 Hz), so V7 does not
brighten as a level — exactly the non-goal. The standalone dash is control-only.

Residual: the 2–5 kHz median is still slightly above the reference. The event-based sparkle
gives brightness shape and reserves the loudest bursts for accents (max share 0.920), but
the constant floor is not yet below the reference. This is left as future tuning rather than
a blocking gap, consistent with the non-goal of raising global brightness as a level.

This is a directional listening aid. By-ear acceptance for reliable BB-8-family identity and
the "opener, wait, answer" staging remains the final gate. The sample-level contract remains
the committed golden WAV hash fixture.
