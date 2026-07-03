# VOICE_V12 spike evaluation worksheet (T-118)

> Status: **awaiting the by-ear session.** The spike (T-115…T-117) is built and the
> automated half of the T-118 evaluation has run; this note records those results, the
> listening checklist, and the open decisions the by-ear session must lock. The go/no-go
> and the locked recipe get recorded here, then T-119 turns them into contract FRs.

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

| #   | Decision                | Spike setting (starting point)                                       | Locked value |
| --- | ----------------------- | -------------------------------------------------------------------- | ------------ |
| 1   | POS source & storage    | hard-coded lexicon; ship = sidecar baked table, coding-domain-ranked | _tbd_        |
| 2   | Marker aggressiveness   | word-initial content nouns/verbs only                                | _tbd_        |
| 3   | Foley boldness          | mixes 0.16 / 0.14 (vs 0.04 soft transient)                           | _tbd_        |
| 4   | Syllable count & pacing | uniform 2 syllables, cap 3; compound scale 0.62                      | _tbd_        |
| 5   | Aspect marking          | out of scope (`VOICE_V13`)                                           | —            |
| 6   | Ambiguity policy        | dominant-class (A/B'd; conservative leg = silent on coding text)     | _tbd_        |

**Go/no-go:** _tbd by ear._
