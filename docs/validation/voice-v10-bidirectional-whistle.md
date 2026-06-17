# VOICE_V10 acceptance — bidirectional whistle & gesture vocabulary

> Status: **Accepted for VOICE_V10.** The `--version` string reports
> `dootdoot VOICE_V10`. This note records the directional gesture-vocabulary
> deltas behind the freeze; the binding contract is the regenerated golden WAV
> hashes (`dootdoot-core/tests/fixtures/golden_wav_hashes.tsv`, header
> `# VOICE_V10`), not the numbers here.

## Source

Phase 19 implements the recommendations of
[`bb8-sound-vocabulary-taxonomy.md`](../research/bb8-sound-vocabulary-taxonomy.md):
a frame-by-frame, gesture-level comparison found dootdoot had the right gesture
families but was **rising-biased and register-shy** — it never produced a falling
whistle, its accents barely left the register, single gestures spanned ~1 octave
where BB-8 spans 3–4, neutral gestures ran long, and it never crossed into a
rough/noisy burst.

The six slices (each its own revision):

- **T-102** — a signed whistle sweep: a negative amount now descends toward
  `WHISTLE_FLOOR_HZ` instead of only climbing.
- **T-103** — the exclamation terminal flourish descends (a falling whistle),
  while the question flourish keeps rising; direction follows `pitch_velocity`.
- **T-104** — the accent whistle engages harder (an engaged floor instead of a
  near-zero ramp) and earlier (`CURVE_WHISTLE_START_FRACTION` 0.45 → 0.30), with
  the gate isolating the one promoted accent from non-accent body syllables.
- **T-105** — the promoted semantic accent swoops on the wider
  `ACCENT_PITCH_SPAN_SEMITONES` (26 st) toward BB-8's multi-octave gestures.
- **T-106** — neutral (text-path) syllables pace shorter than the base so
  gestures read as quick blips; the hand-built / empty-chirp path is byte-identical.
- **T-107** — a body accent in an agitated utterance (high arousal **and**
  negative valence) bursts its noise/breath roughness toward the noisy band, then
  recovers; non-accent and calm syllables keep the steady-state texture.

## Method

`scripts/sound_taxonomy.py` segments renders into gesture events and classifies
each into a droid sound type (see the research note for the full method). The
profile below pools ten emotionally-varied renders mirroring the BB-8 clip
contexts (the same set the research note used), before (`VOICE_V9`) and after
(`VOICE_V10`):

```bash
uv run scripts/sound_taxonomy.py target/sound-taxonomy/ddv10-wav/*.wav
```

This is a directional tuning instrument, not the voice contract; by-ear review
remains the acceptance gate.

## Vocabulary deltas (10 renders)

| axis                                        | BB-8 target   | dootdoot V9       | dootdoot V10      | read                                                                           |
| ------------------------------------------- | ------------- | ----------------- | ----------------- | ------------------------------------------------------------------------------ |
| contour rise / fall / flat                  | 31 / 28 / 41% | 40 / **13** / 48% | 34 / **22** / 45% | the fall fraction nearly doubled toward BB-8                                   |
| falling gestures (down chirp/swoop/whistle) | many          | 2                 | **13**            | descending gestures now exist (incl. `chirp-down` 0 → 7, a falling whistle)    |
| gesture duration med / p90 (ms)             | 87 / 157      | **134** / 180     | **110** / 163     | neutral gestures shortened toward BB-8's blips                                 |
| per-gesture range p90 (st)                  | 44.9          | 13.6              | 13.0              | accents swoop wider in capability; p90 little moved on this sample             |
| dominant-peak reach p90 (Hz)                | 2756          | 711               | 754               | whistle reaches higher, but still rarely carries the peak                      |
| tonality mixed / noisy                      | 16 / 2%       | 14 / 0%           | 18 / 0%           | roughness lifted; no agitated-enough accent in this sample to cross into noisy |

## Read & residual gap

The headline win is **polarity**: dootdoot is no longer rising-only. The
exclamation flourish descends, downward chirps/swoops appear across the corpus,
and the fall fraction moves from 13% to 22% (BB-8 ~28%). Neutral gestures also
shortened (134 → 110 ms median) toward BB-8's quick blips.

Two gaps remain capability-present but under-realised on this sample, consistent
with the research note: the whistle now reaches higher and engages harder, but
still rarely carries the **dominant** spectral peak for enough frames to move the
p90, and the agitation burst (T-107) needs a genuinely high-arousal,
negative-valence accent to fire, which this phrase set only grazes. Both are
engagement-depth tuning, not missing machinery, and are left for a future pass.

## Determinism & contract

- All synthesis stays inside the fixed, deterministic, bounded droid parameter
  space (NFR-16); no unseeded randomness was added.
- The hand-built / empty-chirp / neutral-curve path is byte-identical to
  `VOICE_V9` (T-106 shortening and T-107 roughness are gated on the explicit
  text/planner path; the whistle sign/floor changes are no-ops at zero amount).
- `ACTIVE_VOICE` is `VOICE_V10`; the **golden WAV hashes** were regenerated and
  the double-run determinism test passes.
