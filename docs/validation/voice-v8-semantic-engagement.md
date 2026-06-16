# VOICE_V8 semantic engagement & bursty texture acceptance

Status: **Accepted for VOICE_V8**.

`VOICE_V8` engages the expressive primitives that `VOICE_V7` already shipped (whistle
sweep, noise/breath roughness, event-based sparkle, staged rests) from **semantics**
rather than only punctuation, and reshapes the upper-mid layer from a constant bed into
bursts. It keeps the four PCA semantic axes as the learnable core and reuses the V2–V7
performance channels.

## Scope

- `dootdoot --version` reports `dootdoot VOICE_V8`.
- The embedded semantic mapping remains the locked token-to-axis table in
  `assets/dootdoot_asset_v1.doot`; the PCA mapping is unchanged.
- The discourse-performance planner now derives bounded per-syllable **semantic
  engagement** from the four-axis salience and word-to-word movement, and promotes one
  **semantic accent** per chatty-reply/probe segment. Punctuation-assigned roles are
  unchanged; only the per-syllable curves widen.
- A body-syllable semantic accent can engage the **whistle** sweep and wider pitch span
  without terminal punctuation, gated by a named archetype-tension threshold.
- The default upper-mid brightness of ordinary body syllables is lowered and the
  event-based sparkle is **burstier** (sharper `sin^2` envelope, lower floor, higher accent
  peak), so the constant 2-5 kHz share stays modest while accent bursts rise.
- Engaged body syllables carry a small always-on **roughness floor**, so neutral text is
  not pinned to pure periodicity. Neutral-curve rendering (the empty chirp and hand-built
  events) keeps a zero roughness floor and stays cleanly periodic.
- Neutral multi-word input gets short (~40 ms) silent **word-boundary rests** independent
  of punctuation; structured/punctuated utterances keep their longer staged rests and tonal
  bridges.
- The committed golden WAV hashes were regenerated under `VOICE_V8`.

## Directional Check

Derived from
[`bb8-corpus-timbre-texture-analysis.md`](../research/bb8-corpus-timbre-texture-analysis.md).
The check compares the **whole** local BB-8 reference corpus against 15 emotionally-varied
dootdoot renders, before (`VOICE_V7`) and after (`VOICE_V8`). All clips decoded to mono
44.1 kHz; 2048-sample Hann frames, 512 hop, active frames gated at 8% of each clip's peak
frame RMS. Numbers are produced by the locked uv harness and reproduced with:

```bash
scripts/bb8-metrics
uv run scripts/acoustic_metrics.py --aggregate \
  bb8-lib=target/bb8-metrics/reference-wav/*.wav \
  bb8-ctx=target/bb8-metrics/contextual-wav/*.wav \
  dootdoot=target/bb8-metrics/dootdoot-expanded/*.wav
```

Group medians (`[p10-p90]` omitted for brevity; see the research note). `bb8-lib` = 32
isolated library SFX, `bb8-ctx` = 7 performance excerpts:

| Measurement                | bb8-lib | bb8-ctx | dootdoot VOICE_V7 | dootdoot VOICE_V8 | Read                                                         |
| -------------------------- | ------: | ------: | ----------------: | ----------------: | ------------------------------------------------------------ |
| Active frame fraction      |    0.45 |    0.99 |             0.915 |             0.812 | neutral word rests de-bridge the single long island.         |
| Active islands             |       2 |       4 |                 1 |                 5 | neutral input now stages into discrete events.               |
| Median active island       |  232 ms |  389 ms |            911 ms |            186 ms | events are now BB-8-sized, not one long smear.               |
| Max internal gap           |   64 ms |   12 ms |              0 ms |             35 ms | neutral input now has real internal silence.                 |
| Dominant peak range        | 1010 Hz | 2610 Hz |            431 Hz |           2240 Hz | semantic accents engage the whistle without punctuation.     |
| Fraction above ~1.1 kHz    |    0.04 |    0.18 |              0.00 |              0.02 | accents now reach the whistle band.                          |
| Harmonicity median         |    0.80 |    0.79 |             0.946 |             0.916 | roughness floor swings the body off pure periodicity.        |
| Harmonicity IQR            |    0.19 |    0.27 |             0.042 |             0.109 | clean→rough swing now present (2.6× wider).                  |
| Spectral centroid median   |  846 Hz |  619 Hz |            647 Hz |            617 Hz | brightness **level** unchanged and in range (a non-goal).    |
| 2-5 kHz share, median      |   0.033 |   0.003 |             0.058 |             0.066 | upper-mid bed stays modest (within library range).           |
| 2-5 kHz share, max (burst) |    0.31 |    0.81 |             0.157 |             0.877 | upper-mid now **bursts** on accents instead of staying flat. |
| Spectral flux median       |    0.32 |    0.38 |             0.242 |             0.263 | slightly more animated.                                      |
| Onset rate                 |  6.5 Hz |  9.0 Hz |            6.1 Hz |            8.3 Hz | more articulations per second.                               |
| Tremolo depth              |    0.32 |    0.12 |             0.257 |             0.264 | warble depth held in the BB-8 pocket (unchanged on purpose). |
| Crest factor               | 20.2 dB | 18.5 dB |           21.8 dB |           19.9 dB | dynamics now match the BB-8 corpus.                          |

Every targeted axis moved toward the BB-8 corpus; the two deliberate non-goals (brightness
level and warble) held steady. The one soft axis is the 2-5 kHz **median** share, which
stays slightly above the library median but well within its range — the headline change is
that the upper-mid is now bursty (max 0.16 → 0.88), not constant.

## Acceptance

`VOICE_V8` is accepted as the active contextual-performance contract. The directional
metrics above are a tuning aid, not the sample-level contract; the **golden WAV hashes**
remain the byte-exact contract and were regenerated under `VOICE_V8`. Determinism,
boundedness, and the no-runtime-randomness and no-PCA-change invariants are preserved.
