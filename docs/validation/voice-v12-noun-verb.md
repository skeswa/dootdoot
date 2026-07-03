# VOICE_V12 noun/verb recognizability — acceptance

**Accepted for VOICE_V12** on 2026-07-04, after a two-round by-ear evaluation
(2026-07-03, recorded in
[`voice-v12-spike-evaluation.md`](../research/voice-v12-spike-evaluation.md)) and the
directional instrument checks below. The surfaced version string is
`dootdoot VOICE_V12`.

## What changed

Content words carry a systematic two-pillar class signature (FR-114…FR-121):

- **Layered co-onset class markers** — noun = broadband click/pop splash (7 inharmonic
  partials + 620 Hz thud, 30 ms window, 8 ms attack ramp, mix 0.10), verb = up-swept
  dual-sine chirp (1400→3600 Hz and 2050→5150 Hz, 50 ms, 25% attack fraction, mix
  0.09). Both start together with the tonal body and bloom with its 15 ms attack so
  the mark fuses into the word rather than reading as a separate pre-beat (the
  round-1 by-ear finding; round 2 confirmed the fix and the softened gains).
- **Compound `stem → class-resolution` silhouette** — noun settles (vowel rounds
  toward `oo`, pitch steps down, contour flattens, warble calms), verb pushes
  (brighter toward `ee`, rising floor). Single-token content words gain one derived
  resolution syllable; multi-token words shape their last subword; every content-word
  syllable takes the 0.62 compound duration scale so words read heavy without
  dragging the `VOICE_V11` pace.
- **Classes from a baked, pinned table** under the conservative ambiguity policy the
  by-ear A/B locked: noun/verb-ambiguous coding lemmas (`build`, `fix`, `run`,
  `update`) stay unmarked, closed-class words are excluded, and unclassified words
  render exactly as `VOICE_V11` — the empty chirp and all hand-built paths are
  byte-identical.

## By-ear acceptance

Round 1: class identity tellable (`bug` vs `run` blind), pacing breathes, but the
markers read as a separate percussive pre-beat → fused attack ramps + softer mixes.
Round 2: "the mark does now fuse into the word and softening sounds great." The
conservative ambiguity leg won the A/B (the dominant leg's consistently-wrong
markings were not preferred).

## Directional instrument checks

`scripts/sound_taxonomy.py` on the frozen minimal pair — the classes separate on
**gesture category**, not just pitch:

| render | label    | contour | tonality | net     |
| ------ | -------- | ------- | -------- | ------- |
| `bug`  | blip     | flat    | tonal    | −1.2 st |
| `run`  | chirp-up | rise    | tonal    | +9.2 st |

`scripts/acoustics` (gap-analysis metrics) on "verify the bug in the changelog" vs
the BB-8 reference clip: harmonicity median 0.879 vs 0.853 (in-family), max internal
gap 35 ms vs 58 ms, 2–5 kHz share median 0.066 — the markers add bright-band attack
energy while the voice stays one droid. Against the `VOICE_V11` baseline the pacing
guardrail held during the spike evaluation: identical island structure and +7%
duration on the marked sentence.

## Contract

`ACTIVE_VOICE` is `VOICE_V12`; the golden WAV fixtures were regenerated
byte-for-byte (`DOOTDOOT_REGEN_GOLDEN=1`). Only text-path fixtures containing baked
content words moved (`cat`, `dash`, `ellipsis`, `long`); `empty.wav` and all
unclassified-word fixtures are unchanged, pinning the no-class path. The class table
is part of the voice contract: regenerating `assets/dootdoot_pos_v1.doot` with
different classifications changes rendered samples and requires a version bump.
