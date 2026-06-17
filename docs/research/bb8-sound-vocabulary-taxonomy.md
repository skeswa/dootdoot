# BB-8 Sound-Vocabulary Taxonomy (frame-by-frame) vs. dootdoot

> Status: **research / directional**. This note segments every clip in the
> `bb8-clips/` contextual corpus into discrete sound _events_ (frame by frame),
> classifies each event into a droid sound _type_, and compares the resulting
> vocabulary against what dootdoot's synth actually emits when the same analyzer
> is pointed at dootdoot renders.
>
> It does not change the voice contract. Recommendations are directional and
> would require a new `VOICE_V*` (sample-affecting) + regenerated golden hashes.
>
> Where [`bb8-corpus-timbre-texture-analysis.md`](./bb8-corpus-timbre-texture-analysis.md)
> reports one aggregate vector per clip, this note works at the **gesture** level:
> it answers "what _kinds_ of sounds does BB-8 use, and which ones can dootdoot
> make?" rather than "what is the average timbre?"

## Method

`scripts/sound_taxonomy.py` (locked `uv` env, numpy/scipy, same as
`acoustic_metrics.py`) decodes a clip, computes per-frame features
(FRAME 2048 / HOP 256 ≈ 5.8 ms: dominant peak pitch, autocorrelation
harmonicity, spectral centroid, 2–5 kHz share, spectral flux), then:

1. **Gates** to active regions (RMS ≥ 0.10·max).
2. **Sub-segments** each region into single gestures at boundaries — onset
   spikes (flux local maxima), pitch discontinuities (≥ 5 st between settled
   frames), and deep RMS valleys — because BB-8 chains gestures _legato_
   (continuous energy, abrupt pitch jumps), so RMS gating alone fuses a whole
   phrase into one island.
3. **Classifies** each gesture from its contour (rise / fall / arch / dip /
   flat / wide-sweep), tonality (tonal / mixed / noisy via median harmonicity),
   brightness, tremolo, and internal articulation count into a droid sound type.

```bash
# decode the 7 contextual clips, then:
uv run scripts/sound_taxonomy.py target/sound-taxonomy/ctx-wav/*.wav
uv run scripts/sound_taxonomy.py --trace clip.wav    # per-event detail
uv run scripts/sound_taxonomy.py --frames clip.wav   # raw per-frame dump
```

**Caveats.** Simple argmax peak-pitch tracking on MP3-decoded, formant-heavy,
occasionally polyphonic film audio is noisy; the pitch track is 5-tap
median-filtered to reject octave errors, but absolute pitch is directional, not
exact. Three clips are quiet/low and partly **ambient-dominated**:
`lost-friends-sad` (centroid ~140 Hz — a soft low moan under film bass) and
`explosion-surprise` (a loud ~65 Hz explosion _boom_ with BB-8's reaction layered
on it) sit below BB-8's normal register, so their "events" are not all droid
gestures. Read the large, consistent cross-group differences, not the third
significant figure.

## The BB-8 gesture vocabulary (90 events across 7 clips)

Tally of sound types found frame-by-frame in `bb8-clips/`:

| Type              |   n | What it is                                                                                                  |
| ----------------- | --: | ----------------------------------------------------------------------------------------------------------- |
| `blip`            |  30 | Short (<150 ms) discrete tonal pip, near-flat. The connective tissue — BB-8 fires loose _strings_ of these. |
| `chirp-up`        |  10 | Short rising pip.                                                                                           |
| `rising-swoop`    |   9 | Sustained wide upward glide (mid register).                                                                 |
| `rising-whistle`  |   8 | Wide upward glide into the **bright/high** register (>1.2 kHz, up to ~4.3 kHz).                             |
| `falling-swoop`   |   8 | Sustained wide downward glide.                                                                              |
| `falling-whistle` |   7 | Wide **downward** glide from a high peak (often a full octave-plus drop).                                   |
| `chirp-down`      |   7 | Short falling pip.                                                                                          |
| `trill/chatter`   |   4 | Densely re-articulated run (many internal onsets).                                                          |
| `steady-tone`     |   4 | Flat sustained note — incl. the high alarm _siren_ tones (4.8 kHz, 6.8 kHz).                                |
| `noise-burst`     |   2 | Noisy / inharmonic burst (rough squawk).                                                                    |
| `wide-warble`     |   1 | Flat-ish but spanning a wide band with strong AM.                                                           |

**Aggregate profile** (90 events): contour **rise 31% / fall 28% / flat 41%**;
tonality **tonal 82% / mixed 16% / noisy 2%**; **27%** bright; **41%** are
wide-sweeps (≥12 st); gesture duration **87 ms median / 157 ms p90**; per-gesture
pitch range **7.3 st median / 44.9 st p90**; dominant-peak pitch **689 Hz median
/ 1600 Hz p90**; peak reach **818 Hz median / 2756 Hz p90**.

### Per-clip frame-by-frame character

- **enemy-approaching-alarm** (19 events): a scatter of mid blips and short
  swoops, resolving into **two very high steady tones (4.8 kHz, 6.8 kHz)** — the
  literal alarm siren. Wide range, bright tail.
- **excited-explanation** (22 events, the richest): rapid alternation of **wide
  rising swoops** and **dramatic falling whistles** (peaks crashing from highs
  down to 65–194 Hz, single-gesture ranges of 45–81 st), capped by a
  **noise-burst**. This is the "greatest variety" clip — even more than
  inquisitive.
- **found-fixed-excitement** (19 events): **two sustained high held tones**
  (1.3 kHz) — a happy "taa-daa" — surrounded by rising whistles/swoops and a
  closing chatter run.
- **inquisitive-then-chatty** (11 events): opens with **falling swoops**
  (downward "inquisitive" inflection), then climbs into **rising whistles to
  ~4.3 kHz** as it gets chatty.
- **left-behind-anxious** (7 events): wide swoops with **`mixed`/`noisy`
  tonality** (audible roughness) plus a **noise-burst** — the anxious texture.
- **lost-friends-sad** (4 events): **low (65–258 Hz), slow, descending** — a
  mournful downglide (ambient-influenced; see caveats).
- **explosion-surprise** (8 events): dominated by the **low explosion boom**
  (~65 Hz throughout); BB-8's reaction is not cleanly separable.

The signature is: **mostly short discrete tonal blips strung loosely, punctuated
by wide bidirectional glides — rising AND falling whistles that ride the dominant
spectral peak up into the 1.5–7 kHz region — with occasional rough/noisy bursts
for agitation.**

## dootdoot through the same lens (63 events across 10 renders)

Renders mirror the seven contexts plus neutral/staged phrases (e.g.
`enemy approaching, get ready!`, `what was that?!`, `hello. what? wait... no!`).

| Type            |   n |     | Type             |   n |
| --------------- | --: | --- | ---------------- | --: |
| `blip`          |  15 |     | `rising-whistle` |   4 |
| `trill/chatter` |  13 |     | `rising-swoop`   |   3 |
| `chirp-up`      |   9 |     | `warble-tone`    |   2 |
| `chirp-down`    |   5 |     | `falling-tone`   |   1 |
| `rising-tone`   |   5 |     | `wide-warble`    |   1 |
| `steady-tone`   |   4 |     | `falling-swoop`  |   1 |

**Aggregate profile** (63 events): contour **rise 40% / fall 13% / flat 48%**;
tonality **tonal 86% / mixed 14% / noisy 0%**; **22%** bright; **21%** wide-sweep;
gesture duration **134 ms median / 180 ms p90**; per-gesture pitch range **4.8 st
median / 13.6 st p90**; dominant-peak pitch **452 Hz median / 689 Hz p90**; peak
reach **581 Hz median / 711 Hz p90**.

## Side-by-side vocabulary profile

| Axis                               | BB-8 (90 ev)      | dootdoot (63 ev)  | Read                                                      |
| ---------------------------------- | ----------------- | ----------------- | --------------------------------------------------------- |
| contour rise / fall / flat         | 31 / **28** / 41% | 40 / **13** / 48% | dootdoot is **rising-biased**; ~half the falling gestures |
| tonality tonal / mixed / **noisy** | 82 / 16 / **2**%  | 86 / 14 / **0**%  | dootdoot **never crosses into noisy**                     |
| bright (≥1.1 kHz or upmid)         | 27%               | 22%               | **level matches** (confirms prior corpus note)            |
| wide-sweep (≥12 st)                | **41%**           | **21%**           | dootdoot makes half as many wide pitch excursions         |
| gesture duration med / p90         | **87** / 157 ms   | **134** / 180 ms  | dootdoot gestures ~50% longer — over-connected            |
| per-gesture range med / **p90**    | 7.3 / **44.9** st | 4.8 / **13.6** st | BB-8 swoops span ~4 octaves; dootdoot caps near 1         |
| peak reach med / **p90**           | 818 / **2756** Hz | 581 / **711** Hz  | dootdoot's dominant peak **almost never rides high**      |

## Mapping: which BB-8 sounds can dootdoot make?

| BB-8 gesture                   | dootdoot support    | Notes                                                                                                                                                                               |
| ------------------------------ | ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Short blip                     | ✅ yes              | But dootdoot fires fewer (15 vs 30) and binds them into `trill` runs.                                                                                                               |
| Rising chirp                   | ✅ yes              | `chirp-up` is dootdoot's most native gesture.                                                                                                                                       |
| Falling chirp                  | ⚠️ weak             | Present (5) but smaller; falls come only from punctuation glides (−3 st).                                                                                                           |
| Rising swoop                   | ✅ partial          | Exists; range capped (~13 st vs BB-8's 30–50 st).                                                                                                                                   |
| **Falling swoop**              | ⚠️ weak             | Only **1** observed vs BB-8's 8. dootdoot has no strong downward glide primitive.                                                                                                   |
| **Rising whistle**             | ⚠️ partial          | Whistle sweep exists (→3.4 kHz target) but **under-engaged**: dootdoot's dominant peak stays ≤711 Hz p90 because sparkle/whistle is a thin 4.5% layer, never the loudest component. |
| **Falling whistle**            | ❌ missing          | **0** observed. The whistle primitive only sweeps _up_; there is no high-peak-then-drop gesture.                                                                                    |
| Trill / chatter                | ✅ over-represented | dootdoot's legato over-connection produces _more_ of these than BB-8.                                                                                                               |
| Steady tone                    | ✅ yes              | But never the high siren tones (4.8–6.8 kHz).                                                                                                                                       |
| Warble tone                    | ✅ yes              | Tremolo rate/depth already in the BB-8 pocket (prior note).                                                                                                                         |
| **Noise-burst / rough squawk** | ❌ missing          | dootdoot is harmonic/formant-based; noise/breath is a subordinate blend, never a true noisy burst. 0% noisy events.                                                                 |

## Findings (ranked by leverage)

1. **No falling whistle, and falling glides generally are scarce.** BB-8 spends
   28% of gestures falling (incl. 7 falling-whistles, 8 falling-swoops);
   dootdoot spends 13% and produced **zero** falling-whistles. Architecturally
   the whistle sweep only climbs (toward `WHISTLE_TARGET_HZ` 3.4 kHz) and
   archetype/internal pitch gestures are rise-weighted; the only downward motion
   is the modest −3 st punctuation glide. **A downward-whistle/swoop gesture is
   the single clearest missing word in the vocabulary.**
2. **High-register reach is rare and brief, so the dominant peak stays low.**
   dootdoot's brightness _level_ matches (27% vs 22% bright), but its dominant
   spectral peak tops out at ~711 Hz (p90) while BB-8's rides to 1.6–2.7 kHz
   (p90) and beyond. dootdoot has two separate high-register mechanisms and
   neither carries the dominant peak in practice: the **whistle sweep**
   (`whistle_sweep_pitch_hz`) _does_ move the oscillator fundamental up to
   3.4–4.2 kHz — exactly the right mechanism — but it is gated to
   `TerminalFlourish` and rare body accents (`tension ≥ 0.75`) and only over the
   last 55% of one syllable (`CURVE_WHISTLE_START_FRACTION` 0.45), so few frames
   land high; the **upper-mid sparkle** is a separate additive layer capped at
   `UPPER_MID_SPARKLE_MIX` (0.045), subordinate to `voiced` by design and never
   the dominant peak (synth.rs:1336–1350). So the fix is _engaging the whistle
   sweep more often, earlier, and harder_, not boosting the sparkle. (Consistent
   with the corpus note's "constant not bursty" finding, now localized to
   gestures.)
3. **Per-gesture pitch excursions are too small.** BB-8 routinely swoops 30–50 st
   in a single gesture (p90 = 45 st); dootdoot caps near 14 st (the ±16 st
   `WIDE_GESTURE_PITCH_SPAN` is the hard ceiling and is rarely reached). BB-8
   makes dramatic octave-spanning sweeps; dootdoot makes gentle bends.
4. **dootdoot over-connects and over-articulates.** Gestures run ~50% longer
   (134 vs 87 ms median) and lean to `trill/chatter` (13) where BB-8 leans to
   discrete `blip` (30). BB-8's idiom is short pips with air between them.
5. **No rough/noisy register.** BB-8 uses `noisy`/`mixed` tonality for agitation
   (anxious, the tail of excited); dootdoot stays 0% noisy. The deterministic
   noise/breath blend exists but never reaches burst level.

## Recommendations (directional; a future `VOICE_V*`)

These reuse machinery dootdoot already has — the gap is _engagement, polarity,
range, and mix_, not missing instruments (matching the prior note's conclusion).
Each is sample-affecting and would require a version bump (`VOICE_V9`→`V10`) plus
regenerated golden WAV hashes. They are ordered by leverage; (1) and (2) are the
two that would most change how dootdoot _reads_.

### 1. Add a downward whistle/swoop (the clearest missing word)

**Gap.** BB-8: 7 falling-whistles + 8 falling-swoops across 7 clips (15 in the
library); dootdoot: **0** falling-whistles, 1 falling-swoop. dootdoot's whistle
only ever climbs.

**Why, in code.** `whistle_sweep_pitch_hz` (synth.rs:433) always sweeps from
`start_hz` _toward_ a target at or above `WHISTLE_TARGET_HZ` (3.4 kHz); there is
no path that sweeps below `start_hz`. Downward pitch motion comes only from the
−3 st `PUNCTUATION_GLIDE_SEMITONES` final glide and the modest `Moan` archetype
offset — both far too small to read as a falling whistle. The role curves that
_do_ carry negative `pitch_velocity` (`Hesitation` −0.15, `Aside` −0.10,
performance.rs:433/444) are exactly the roles where `whistle_amount` returns 0
(synth.rs:1090), so falling intent and whistle engagement never co-occur.

**Concrete change.** Generalize the sweep to a signed direction driven by the
sign of `pitch_velocity`: when velocity ≥ 0 sweep up to the existing
`WHISTLE_TARGET_HZ`; when velocity < 0 sweep _down_ to a new `WHISTLE_FLOOR_HZ`
(≈ 280–350 Hz, matching the falling-whistle landings of 65–194 Hz observed but
kept inside dootdoot's bounded register). Then give the statement/exclamation
`TerminalFlourish` a _negative_ terminal `pitch_velocity` while the question
`Probe`/flourish keeps positive — which is also semantically right (questions
rise, statements fall) and mirrors the existing ±-glide split. Keep the neutral
(`tension == 0`) path byte-identical so unchanged inputs don't move.

**Expected effect.** Adds a whole gesture class; brings the fall fraction from
13% toward BB-8's ~28%, and makes terminal statements land with the
characteristic BB-8 descending whistle instead of a small drop.

### 2. Engage the whistle sweep more often, earlier, and harder

**Gap.** dootdoot's dominant peak p90 is ~711 Hz vs BB-8's ~2.7 kHz; only the
rare flourish frame clears 1.1 kHz.

**Why, in code.** The fundamental-moving mechanism already exists and is correct
— it is just under-deployed. `whistle_amount` (synth.rs:1074) is non-zero only
for `TerminalFlourish`, or for `ChattyReply`/`Probe` once
`archetype_tension ≥ WHISTLE_ACCENT_TENSION_THRESHOLD` (0.75) — a high bar that
neutral and mildly-expressive text rarely clears. When it does fire, the sweep
only starts at `CURVE_WHISTLE_START_FRACTION` (0.45) of the syllable, so at most
~55% of one syllable's frames land high. The sparkle layer is _not_ the lever
here (see finding 2); do not raise `UPPER_MID_SPARKLE_MIX`.

**Concrete change.** (a) Lower `WHISTLE_ACCENT_TENSION_THRESHOLD` and/or let the
planner push body-syllable tension higher on semantic accents so accents in
ordinary text reach the whistle band — raising the share of frames clearing
1.1 kHz toward BB-8's 0.2–0.3. (b) Start the sweep earlier (drop
`CURVE_WHISTLE_START_FRACTION` toward ~0.25) on the most salient accent so more
frames ride high. (c) Keep `WHISTLE_PITCH_CEILING_HZ` (4.2 kHz) — reach is fine;
_frequency of use_ and _dwell time_ are the gap.

**Expected effect.** Moves the dominant-peak p90 up toward the 1.5–2.7 kHz BB-8
band without touching brightness _level_ — turns the whistle from a rare punctuation
ornament into a regular accent gesture.

### 3. Widen accent pitch excursions toward 2–4 octaves

**Gap.** BB-8's per-gesture pitch range p90 is ~45 st (single gestures spanning
3–4 octaves); dootdoot's p90 is ~14 st.

**Why, in code.** Even the "wide" gesture path is capped at
`WIDE_GESTURE_PITCH_SPAN_SEMITONES` (±16 st, synth.rs:141), selected only when
`whistle_amount > 0` (synth.rs:1149); ordinary syllables use
`PITCH_SEMITONE_SPAN` (±10 st). The internal swoop/arch
(`INTERNAL_PITCH_SWEEP_CENTS` 220, `INTERNAL_PITCH_ARCH_CENTS` 90) adds only ~±2 st
on top. So a single dootdoot gesture can't span what BB-8 routinely does.

**Concrete change.** On the single highest-salience accent per phrase (the
semantic-accent the planner already identifies for VOICE*V8), allow the gesture
span to reach ~24–36 st — either by raising the wide-gesture span on that one
gesture, or by letting the whistle sweep run across a \_lengthened* accent
syllable so the start→peak interval itself widens. Leave non-accent gestures at
the current spans so the voice doesn't become uniformly wild.

**Expected effect.** Lifts the per-gesture range p90 from ~14 st toward the
20–45 st BB-8 region on accents, giving the dramatic swoop the corpus is full of.

### 4. Shorten and separate neutral gestures

**Gap.** dootdoot gestures run ~50% longer (134 vs 87 ms median) and skew to
`trill/chatter` (13) where BB-8 skews to discrete `blip` (30 of 90). BB-8's idiom
is short pips with air between them.

**Why, in code.** This is the over-connection the prior corpus note flagged:
neutral multi-word input fuses into one long voiced island (active fraction
~0.92, ~0 ms internal gaps). Word boundaries use a tonal _bridge_ rather than
silence (sequence.rs), so gestures rarely separate.

**Concrete change.** On neutral (un-punctuated) multi-word input, allow short
(~30–80 ms) inter-word rests instead of always bridging, and bias neutral
syllable duration shorter, so the median gesture falls toward ~90 ms and active
fraction toward the library's ~0.45. This is recommendation 5 of the prior
corpus note, here confirmed at the gesture level.

**Expected effect.** Shifts the type mix from `trill/chatter` back toward
discrete `blip` strings — the dominant BB-8 idiom — without removing dootdoot's
ability to chatter when the text warrants it.

### 5. Allow an occasional rough/noisy burst on high-arousal agitation

**Gap.** BB-8 uses `noisy`/`mixed` tonality for agitation (anxious clip, the tail
of excited; 6% genuinely noisy across the library); dootdoot is **0% noisy**.

**Why, in code.** `roughness_amount` (synth.rs:1094) caps the noise/breath blend
well below a true burst: even the roughest role (`Hesitation`, 0.7) feeds
`mix = amount · NOISE_BREATH_MAX_MIX` = 0.35, so the voiced source still
dominates and harmonicity never drops into the noisy band (≤0.42). The blend can
roughen but cannot _break up_.

**Concrete change.** On a high-arousal agitated accent (negative valence + high
arousal — the planner's `Tremble`/agitation path), let `roughness_amount` spike
briefly toward ~1.0 (mix → `NOISE_BREATH_MAX_MIX`) for a short window so a single
gesture crosses into the noisy band, then recovers. Keep the steady-state floor
where it is; this is a transient burst, not a new baseline texture.

**Expected effect.** Adds the rare rough squawk BB-8 uses for surprise/agitation
(`noise-burst`), moving a small fraction of agitated gestures into `mixed`/`noisy`
without coloring the whole voice.

## Non-goals (unchanged from prior analysis)

- Don't lower overall brightness (level already matches) — fix its _shape_ and
  _polarity_.
- Don't touch warble (rate/depth already in range).
- Don't add nondeterministic randomness, a vocoder, or sample libraries; every
  recommendation is a deterministic re-shaping of existing primitives.
