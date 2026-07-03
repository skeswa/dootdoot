# VOICE_V12 spike evaluation worksheet (T-118)

> Status: **complete — GO, recipe locked (2026-07-03).** Round 1 passed class
> identity and pacing but heard the markers as a separate percussive pre-beat and
> preferred the conservative ambiguity policy; the recipe was retuned (fused attack
> ramps, softer mixes, `FallBackToOther`). **Round 2 confirmed by ear: the mark now
> fuses into the word and the softened gain sounds right.** The locked recipe below
> is what the ship tasks (T-119…T-127) implement.

## Locked recipe (the T-118 output — target constants for the ship tasks)

- **Marking scope**: word-initial content nouns/verbs only; continuations and
  function words never marked.
- **Ambiguity policy**: conservative — noun/verb-ambiguous lemmas fall back to
  `Other` (by-ear A/B winner). Closed-class words are simply absent from the table.
- **Noun marker**: broadband click/pop splash + 620 Hz thud; partials
  {1670, 2390, 3110, 3970, 4830, 5660, 6420} Hz; window 30 ms; quadratic attack ramp
  8 ms (fuses with the body's 15 ms bloom); mix 0.10.
- **Verb marker**: dual-sine up-swept chirp, 1400→3600 Hz and 2050→5150 Hz; window
  50 ms; attack fraction 0.25; mix 0.09.
- **Compound silhouette**: uniform `stem → resolution` (2 syllables), target
  `max(subword_count, 2)` capped at 3; per-syllable compound duration scale 0.62.
- **Noun settle transform**: pitch −0.28; vowel `×0.35 + 0.65` (toward `oo`);
  contour ×0.15; warble ×0.5.
- **Verb push transform**: pitch +0.18; vowel `×0.35 − 0.65` (toward `ee`);
  contour `0.70 + 0.30×stem`; warble `0.6×stem + 0.25`.
- **POS source & storage**: sidecar baked class table (research §9.1), entries
  ranked by a pinned coding-domain corpus snapshot, keyed by **lemma** with an
  explicit inflection rule (the spike's surface-form-only matching is a known gap);
  source + corpus pinned in `assets/source_manifest.toml` (T-120).

## Round-1 outcomes (by ear, 2026-07-03)

| #   | Question        | Verdict                                                     | Action taken                                                                                                                           |
| --- | --------------- | ----------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Fusion          | **Fail** — read as a separate pre-beat                      | Markers now bloom with the tonal body: noun gets an 8 ms quadratic attack ramp + wider 30 ms window; verb attack fraction 0.12 → 0.25. |
| 2   | Class identity  | **Pass** — noun vs verb tellable                            | —                                                                                                                                      |
| 3   | Pace            | **Pass** — still breathes                                   | `COMPOUND_SYLLABLE_DURATION_SCALE` stays 0.62.                                                                                         |
| 4   | Ambiguity A/B   | **Conservative preferred**                                  | `SPIKE_AMBIGUITY_POLICY` locked to `FallBackToOther`; `build/fix/run/update/sync/…` stay unmarked blips.                               |
| 5   | Marker severity | **Too percussive/severe** (though marked = easier to parse) | Mixes dialed back: noun 0.16 → 0.10, verb 0.14 → 0.09 (still ≳2× the 0.04 soft transient).                                             |

Round-2 renders (`*-v2.wav` in `target/spike-v12/`): `a-bug-marked-v2`,
`a-run-marked-v2`, `b-fix-the-bug-marked-v2` (note: `fix` is now unmarked under the
conservative policy — only `bug` carries a mark), and `d-content-heavy-v2`
("verify the release and deploy the server", two unambiguous verbs + one unambiguous
noun). Taxonomy still separates the retuned pair: `bug` = flat tonal blip, `run` =
rising chirp (+9.2 st).

## How to reproduce the renders

The spike is behind the default-off `spike-noun-verb` cargo feature (a local
compile-time gate, not a user-facing voice). Evaluation WAVs land in
`target/spike-v12/`:

```bash
# Marked legs (gate on)
cargo run -p dootdoot --features spike-noun-verb -- "bug" -o target/spike-v12/a-bug-marked.wav
cargo run -p dootdoot --features spike-noun-verb -- "run" -o target/spike-v12/a-run-marked.wav
cargo run -p dootdoot --features spike-noun-verb -- "fix the bug" -o target/spike-v12/b-fix-the-bug-marked.wav
cargo run -p dootdoot --features spike-noun-verb -- "sync the build and run the update" -o target/spike-v12/c-ambiguous-dominant.wav

# VOICE_V11 baselines (gate off): same commands without --features.
# Conservative-ambiguity leg: flip SPIKE_AMBIGUITY_POLICY in
# dootdoot-core/src/pos_class.rs to FallBackToOther, rebuild, re-render leg c.
```

The research doc's canonical pairs (_dog_ vs _run_, _the cat sits_) are swapped for
coding-domain pairs (_bug_ vs _run_, _fix the bug_) because the spike lexicon is
domain-weighted per research §5.1; _dog_/_cat_/_sits_ are not in it. Note the spike
matches **surface forms only** (no lemmatization): _fails_, _bugs_, _running_ classify
`Other` unless WordPiece happens to split them onto a lexicon stem. The baked table
(T-120) should rank **lemmas** and decide inflection handling explicitly.

## Automated results (2026-07-03)

### Minimal pair separates on category, not just pitch

`scripts/sound_taxonomy.py` on the marked minimal pair:

| render       | label          | contour    | tonality | range   | onsets |
| ------------ | -------------- | ---------- | -------- | ------- | ------ |
| `bug` (noun) | blip           | flat       | tonal    | 1.2 st  | 0      |
| `run` (verb) | rising-whistle | wide-sweep | mixed    | 34.9 st | 2      |

The two classes land in **different gesture categories** on onset count, contour
class, and tonality — exactly the P6 cross-category separation the recipe targets.
In `fix the bug`, the verb-initial word reads as a 2-onset rising chirp and the
noun-final word as a `chirp-down → settled blip` pair (the stem→settle silhouette).

### Pacing guardrail held

`scripts/acoustic_metrics.py`, `fix the bug` marked vs `VOICE_V11` baseline: identical
island structure (3 islands, median 174 ms, max gap 35 ms), duration +7% (0.71 s →
0.76 s), onset rate 8.4 → 9.2 Hz, crest 19.0 → 23.7 dB. The compound scale (0.62)
keeps the phrase pace; the markers add attack energy rather than length. A marked
single word runs ~1.25× its `VOICE_V11` length (stem + resolution at 0.62 each).
Against the BB-8 reference clip the marked render stays in-family on harmonicity
(0.81 vs 0.85) and active fraction (0.77 vs 0.63).

### Ambiguity policy: the conservative rule unmarks the whole phrase

On `sync the build and run the update` — all four content words are ambiguous lemmas —
the **conservative leg renders byte-identical to the `VOICE_V11` baseline** (nothing
marked at all), while the dominant leg carries 11 gesture events vs the baseline's 7.
This confirms research §5.1 finding 3 at full strength: fall-back-to-`Other` silences
the marking system on exactly the register it was built for. Unless the by-ear session
finds dominant-class marking actively misleading, **dominant-class is the presumptive
policy**.

## By-ear checklist (the human half of T-118)

1. **Fusion** — in `a-*-marked.wav`, does the mark fuse into the word's attack
   (one object) or read as a separate pre-beat? (T-116 acceptance.)
2. **Class identity** — with eyes closed, can you tell `bug` from `run` by onset
   flavor (click/pop vs up-chirp) and silhouette (settle vs push)?
3. **Pace** — does `b-fix-the-bug-marked.wav` still breathe like `VOICE_V11`, or do
   compound words drag? (T-117 acceptance; tune `COMPOUND_SYLLABLE_DURATION_SCALE`.)
4. **Ambiguity A/B** — `c-ambiguous-dominant.wav` vs `c-ambiguous-conservative.wav`:
   is consistently-marked-but-sometimes-wrong more learnable than mostly-unmarked?
5. **Marker gain** — are `NOUN_MARKER_MIX` (0.16) / `VERB_MARKER_MIX` (0.14) bold
   enough to cue class without reading as percussion? (Research §9.3: start bold,
   dial back.)

## Decisions to lock before T-119 (research §9)

| #   | Decision                | Spike setting (starting point)                                       | Locked value                                                  |
| --- | ----------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------- |
| 1   | POS source & storage    | hard-coded lexicon; ship = sidecar baked table, coding-domain-ranked | _tbd at T-119/T-120_                                          |
| 2   | Marker aggressiveness   | word-initial content nouns/verbs only                                | **locked: word-initial content only** (round 1 #2 passed)     |
| 3   | Foley boldness          | mixes 0.16 / 0.14 (vs 0.04 soft transient)                           | **locked: 0.10 / 0.09 + fused attack ramps** (round 2 passed) |
| 4   | Syllable count & pacing | uniform 2 syllables, cap 3; compound scale 0.62                      | **locked: uniform 2, cap 3, scale 0.62** (round 1 #3 passed)  |
| 5   | Aspect marking          | out of scope (`VOICE_V13`)                                           | —                                                             |
| 6   | Ambiguity policy        | dominant-class (A/B'd; conservative leg = silent on coding text)     | **locked: conservative / fall-back-to-`Other`** (round 1 #4)  |

**Go/no-go:** **GO** — round 1: classes tellable, pace holds, marked renders easier to
parse; round 2: the mark fuses into the word and the softened gain sounds right.
