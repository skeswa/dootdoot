# BB-8 Inquisitive-Then-Chatty Gap Analysis

> Status: **research / recommendation**. This note compares the local reference clip
> `/Users/skeswa/repos/anddav87/bb8-sounds/bb8-clips/inquisitive-then-chatty.mp3`
> against the current `VOICE_V6` output for
> `Hello - good morning Sandile. What are you doing today?!`.
>
> It does not change the voice contract. Every recommendation below is
> sample-affecting and would require a new voice version plus regenerated golden WAV
> hashes.
>
> Revision note: the original draft of this document framed the gap as primarily a
> performance-planning problem. A second pass independently reproduced every clip-level
> measurement (within rounding) and traced each cause to source. That pass changed the
> conclusion's weighting: two of the reference's three largest signatures are things the
> current synthesis instrument **physically cannot produce**, regardless of how a planner
> schedules them. The claims and the suggested implementation order below reflect that
> reweighting.

## Summary

`VOICE_V6` has solved many first-order problems from the earlier tuning passes: it is no
longer a single clean beep, word starts are smoother, and repeated phrase bridges no
longer dominate the syllable body. The remaining difference is higher level. The
reference sounds like a performed exchange: one inquisitive gesture, a long conversational
gap, then a compact chatty answer made of varied mini-gestures that climb into a
whistle. The dootdoot render sounds like a deterministic token sequence: every voiced
token goes through a similar high-arousal treatment, in a narrow low pitch band, with
short bridged word gaps and only punctuation creating true phrase resets.

The gap is **three co-primary problems**, not one. In rough order of how much each
separates the two clips:

1. **Tonal pitch range and glide (synthesis limit).** The reference's dominant spectral
   peak sweeps across ~4.5 octaves and ends on a ~4.3 kHz whistle. dootdoot's fundamental
   is mathematically confined to ~0.5–1.1 kHz and moves ~2.6 semitones within a syllable.
   dootdoot cannot whistle, ever. This is the **largest** ratio in the measurement table
   (8.3×) and is fixed by named constants, not by planning.
2. **Timing — two independent failures.** (a) The reference has a ~1.1 s question-to-answer
   pause; dootdoot's largest possible pause is a ~240 ms constant. (b) dootdoot fills 0.63
   of the file with sound vs the reference's 0.44, because it **bridges word gaps with
   tone** instead of leaving rests.
3. **Excitation roughness (synthesis limit).** The reference swings between cleanly
   pitched and rough/smeared frames (harmonicity IQR 0.23). dootdoot's excitation is
   always a strictly periodic oscillator (IQR 0.05); there is no aperiodic/noise source in
   the baseline path.

A fourth, enabling problem sits on top of these: affect and archetype are computed once
per utterance and applied globally, so the whole phrase saturates into one
high-arousal Yelp identity. This is real, but it is the layer that should _deploy_ the
gestures above — it is not where the missing identity comes from.

The most important correction to earlier intuition: **do not "make it brighter."** This
render already has a higher spectral centroid than the reference. The brightness it has is
the wrong _kind_ (see §"Cause Analysis 1"). The next step is to give the synthesis
instrument the dynamic range it lacks — pitch sweep, true silence, and roughness — and
then a deterministic performance planner to schedule those into discourse roles.

## Method

Reference decode:

```bash
ffmpeg -hide_banner -loglevel error -y \
  -i /Users/skeswa/repos/anddav87/bb8-sounds/bb8-clips/inquisitive-then-chatty.mp3 \
  -ac 1 \
  -ar 44100 \
  target/research/bb8-inquisitive-chatty/reference.wav
```

Dootdoot render:

```bash
cargo run -p dootdoot -- \
  "Hello - good morning Sandile. What are you doing today?!" \
  -o target/research/bb8-inquisitive-chatty/dootdoot.wav
```

The clip-level measurements below use:

- 44.1 kHz mono, 16-bit PCM WAVs.
- Hann-windowed 2048-sample frames with a 512-sample hop.
- Active frames gated at 8% of each clip's peak frame RMS.
- A separate `ffmpeg silencedetect` pass at `noise=-36dB:d=0.035` to confirm audible
  rests.

These are directional diagnostics, not acceptance constants. The reference clip is a
sound-effect clip, not a lab-isolated phonetic sample, and simple F0 tracking can be
fooled by formant-heavy droid audio. The measurements are still useful because the
largest differences are structural and large. They were also independently reproduced
with a separate numpy/scipy pipeline (same frame/hop/gate), which agreed with the table
below within rounding and to the millisecond on the island sequences.

## Observations

The current `--explain` output for the dootdoot phrase starts with:

```text
token │ pitch │ vowel │ contour │ warble
mood │ valence:+0.475 │ arousal:+1.000 │ - │ -
```

That matters. The input's punctuation, length, and positive terms saturate arousal to
`1.000`, so the current high-energy path is applied broadly rather than staged. The
standalone `-` also appears as a voiced token with its own four-axis values, not as a
control marker.

Direct comparison:

| Measurement                 | Reference clip | dootdoot render | Read                                                                 |
| --------------------------- | -------------: | --------------: | -------------------------------------------------------------------- |
| Duration                    |         3.00 s |          3.32 s | Similar overall span.                                                |
| Active frame fraction       |           0.44 |            0.63 | dootdoot fills more of the file with active sound.                   |
| Active islands              |              6 |               9 | reference has fewer, more staged events.                             |
| Median active island        |         197 ms |          209 ms | median event size is similar; event ordering is the bigger gap.      |
| Max internal gap            |        1103 ms |          232 ms | reference has a true question-to-answer pause; dootdoot does not.    |
| Fixed-threshold max silence |        1151 ms |          269 ms | same conclusion under a separate `-36 dB` silence detector.          |
| Dominant peak range         |        4264 Hz |          517 Hz | **largest ratio in the table**: reference sweeps, dootdoot is stuck. |
| Autocorr pitch proxy IQR    |         199 Hz |           79 Hz | dootdoot's periodic pitch region is much steadier.                   |
| Harmonicity median          |          0.904 |           0.937 | dootdoot is more cleanly periodic.                                   |
| Harmonicity IQR             |          0.207 |           0.050 | reference varies between cleaner and rougher frames more strongly.   |
| Spectral centroid median    |         984 Hz |         2214 Hz | this dootdoot render is brighter overall, not darker.                |
| 2-5 kHz power share, median |         ~0.000 |           0.104 | dootdoot keeps upper-mid energy present more constantly.             |
| 2-5 kHz power share, max    |          0.834 |           0.366 | reference has rarer but more extreme upper-mid bursts.               |

Two shapes carry the "inquisitive then chatty" identity, and both are visible in the data.

**Macro-timing.** The reference starts with a ~557 ms opening event, leaves ~1.1 s of
space, then answers with shorter bursts around 244, 151, 151, 104, and 313 ms. The
dootdoot render produces nine more evenly connected islands — about 209, 197, 569, 360,
209, 197, 186, 244, and 209 ms — with no internal gap over ~232 ms.

**Pitch trajectory.** Tracking the median dominant spectral peak in ~250 ms windows
exposes the second shape directly:

```text
REFERENCE: 732 754 409 409 ──── ──── ──── 366 775 689 1593 452 4328   (Hz)
DOOTDOOT : 560 409 345 624 452 388 398 431 409 388  301  323  323     (Hz)
```

The reference opens around 740 Hz (the inquisitive tone), drops, goes silent for ~750 ms,
then answers and **ends on a 4.3 kHz whistle**. 18% of its active frames sit above
dootdoot's hard pitch ceiling of ~1135 Hz. The dootdoot render wanders inside a 280–797 Hz
band for the entire clip, never exceeds 1135 Hz (0% of frames), and never goes silent
mid-clip.

So the reference's identity is not just timbre, and not just timing. It is a macro-shape
(opener, wait, answer) **carried by a tonal voice that can dive dark and leap to a
whistle**. dootdoot's current planner cannot infer the turn structure, its synthesis
cannot produce the whistle, and the standalone `-` is voiced as a normal token rather than
acting as a prosodic dash.

## Cause Analysis

### 1. Brightness is the wrong _kind_, and the synth tonal range is capped

This is the correction that reframes everything else. The measurement table looks
contradictory: dootdoot has a **narrower** dominant peak range (517 vs 4264 Hz) yet a
**higher** spectral centroid (2214 vs 984 Hz). Both are true, and the reconciliation is
the key insight for this clip.

In dootdoot, the tonal peak and the centroid are effectively the same object — median
~409 Hz. The high centroid comes entirely from an **always-present diffuse layer**: 48
saw/pulse harmonics plus a fixed upper-mid sparkle (`UPPER_MID_SPARKLE_MIX = 0.045`,
clamped to 2–5 kHz) mixed under every active syllable. So dootdoot's brightness is _fixed
hash sprinkled over a stuck-low tone_.

In the reference, the tonal body is usually dark (median tonal peak ~538 Hz) but the
**dominant peak itself leaps into whistle range in bursts** (up to 4457 Hz), while median
2–5 kHz share is ~0.000. So the reference's brightness is _the fundamental gliding up_ — a
gesture, not a layer.

This is why a global brightness increase moves in the wrong direction, and why the fix is
not "more sparkle." dootdoot's fundamental is confined by two constants:

- `PITCH_REGISTER_BIAS_HZ = 760.0` and `PITCH_SEMITONE_SPAN = 10.0` bound the fundamental
  to roughly **506–1135 Hz across all possible inputs** (~2 octaves total).
- `INTERNAL_PITCH_SWEEP_CENTS = 220.0` plus `INTERNAL_PITCH_ARCH_CENTS = 90.0` cap
  within-syllable motion at **~2.6 semitones**. There is no rising-chirp gesture that
  sweeps the oscillator upward.

The reference spans ~194–4457 Hz (~4.5 octaves). dootdoot cannot reach whistle register,
so it cannot produce the clip's most salient accents. This is a synthesis-capability
limit, fixed by named constants — not a planning gap.

### 2. Timing: a hard pause ceiling _and_ gap-filling bridges

Two independent timing problems combine to flatten the macro-shape.

First, **the pause ceiling**. `VOICE_V6` timing is boundary-driven, and its largest pause
constant is `LONG_PUNCTUATION_PAUSE_SAMPLES = 10_584` (~240 ms). Multiple punctuation
marks take a `.max()` rather than summing, so there is no path to the reference's ~1.1 s
question-to-answer gap. The measured dootdoot max gap of 232 ms is exactly this ceiling
minus envelope tails.

Second, **gap-filling**. dootdoot's active fraction is 0.63 vs the reference's 0.44, and it
produces nine connected islands vs six staged ones. The cause is the word-boundary
behavior: dootdoot can choose a harmonic _transition bridge_ — tone connecting one word's
pitch to the next — instead of an inter-word rest. That fills silence that the BB-8
aesthetic wants left empty. Even if a planner inserts one large pause, the reply phrase
would still read as over-connected unless bridging can be suppressed.

Also, the ASCII hyphen in the prompt is a voiced token (it appears with four-axis values
in `--explain`), so it consumes time and timbre as a semantic syllable instead of acting
as a hesitation or phrase separator. `.`, `?`, and `!` are already control-only.

### 3. Excitation is always periodic; there is no roughness source

The reference has lower median harmonicity and much wider harmonicity variation (IQR 0.23
vs 0.05). That matches the listening impression of something partly vocal and performed:
some frames lock into a pitched tone, some smear, roughen, or shift formant emphasis.

dootdoot's excitation is a strictly periodic saw/pulse source (48 harmonics) modulated by
deterministic multi-LFO warble (±45 cents vibrato across fixed 3.1/8.5/15.7 Hz rates).
There is **no aperiodic/noise excitation in the baseline path** — only optional servo and
noise-tail textures gated by archetype, which the default Chatter path does not trigger.
With nothing to break periodicity, harmonicity cannot swing the way the reference's does.
This is deterministic by design, and good for it, but it can sound like a synth patch
rather than something almost mammalian. The fix is authored irregularity — a noise/breath
excitation blend and per-gesture roughness — not runtime randomness.

### 4. Affect and archetype are global, so the utterance saturates into one mode

This is the enabling problem that makes the limits above more obvious. Affect is computed
once for the whole utterance: `pooled_arousal` sums eight contributions and clamps to
`[0, 1]`, which for this input pins arousal at `1.000`. That single mood drives every
syllable.

Archetype selection compounds it. In `archetype.rs`, the Yelp branch
(`valence > 0.30 && arousal >= 0.45`) is tested _before_ the StutterBurst branch
(`complexity >= 0.58`). With `valence:+0.475 / arousal:+1.000`, **every voiced syllable
resolves to Yelp**, and because the archetype reads the single global mood event, there is
no per-phrase variety by construction. BB-8's clip is a palette performance: a longer
questioning tone, then smaller chatter elements, then little spectral and amplitude
accents. dootdoot collapses the whole utterance into one color.

Note the dependency: making affect and archetype local will diversify _which_ gestures
fire, but the gestures it would rotate through (whistle, rough chatter, true rests) do not
yet exist. That is why this is the deploying layer, not the root.

### 5. Semantic mapping is not the bottleneck

The semantic knobs move across tokens, and `VOICE_V6` already carries phrase, mood,
complexity, archetype, and continuity channels. The gap is not that text fails to affect
sound. The gap is that the synthesis instrument lacks the dynamic range to express the
reference's gestures, and the performance channels that would stage them are still global.

The semantic PCA layer should remain the learnable core. The next work should sit above
and around it: new synthesis primitives, and a performance layer that schedules them.

## Recommendations

### 1. Expand the synthesis instrument's dynamic range first

A planner can only schedule gestures that exist. Before adding discourse roles, give the
synth the range the reference uses:

- a dedicated rising-chirp/whistle gesture that sweeps the **oscillator** (not just the
  sparkle layer) toward the 2–4 kHz region, so the tonal peak itself can climb;
- a wider per-gesture pitch span so selected events can leave the current ~0.5–1.1 kHz
  band;
- a noise/breath excitation blend mixed under the tonal source for selected gestures, so
  harmonicity can swing clean→rough within a gesture rather than staying pinned near 0.94.

These are bounded, deterministic additions. They are the most direct path to both the
"whistle accent" and the "almost mammalian" qualities.

### 2. Fix timing in two places, not one

- Raise the pause ceiling: allow strong hesitation/turn gaps in the 600–1200 ms range for
  explicit dash/ellipsis or selected question-to-answer arcs. This is a constant change,
  but it must be gated by role (see #4) so simple sentences do not become sluggish.
- Make word-boundary bridging suppressible, so the reply phrase can use 30–80 ms internal
  rests and the opener-gap-answer shape can actually open up. The active fraction should be
  able to fall toward the reference's ~0.44 for staged inputs.
- Allow phrase-final lengthening and amplitude tails that occupy space without counting as
  another voiced syllable.

### 3. Make upper-mid texture event-based, and stop treating brightness as a level

The constant sparkle layer is the wrong kind of brightness. Revise it into a gesture
resource:

- lower the default upper-mid mix for ordinary word-connected syllables;
- reserve short, high-contrast brightness for chirps, terminal flourishes, and selected
  chatter notes, ideally carried by the swept oscillator from #1 rather than added hash;
- give each bright gesture an attack and decay so brightness has shape;
- keep >6 kHz modest, consistent with previous research and this clip.

Success should sound less like "constant gleam" and more like "small bright droid
articulations, including the occasional whistle, inside a rounder voice."

### 4. Add a deterministic discourse-performance planner to deploy the new primitives

Once the primitives exist, add a planner that runs after tokenization and before
synthesis, assigning local phrase roles as a pure function of the event stream,
punctuation, word count, and simple control tokens. Surface its decisions in `--explain`.

Initial roles worth supporting:

| Role                | Trigger candidates                                  | Rendering direction                                    |
| ------------------- | --------------------------------------------------- | ------------------------------------------------------ |
| `probe`             | question mark, leading short phrase, "what/why/how" | longer rising gesture, less chatter density            |
| `chatty_reply`      | phrase after a strong pause or sentence reset       | shorter events, denser burst, alternating archetypes   |
| `hesitation`        | standalone dash, ellipsis, repeated punctuation     | real pause or quiet rounded connector, no voiced token |
| `terminal_flourish` | final `?!`, `!`, or `?`                             | one accented whistle/yelp, not all syllables yelped    |
| `aside`             | comma/colon-delimited short segment                 | lower volume, rounder/darker body, shorter pitch span  |

The planner should also localize affect and archetype: keep the utterance-level mood row,
but compute per-phrase and per-syllable curves (arousal attack/hold/release, final-marker
accent isolated near `?!`, local valence from nearby tokens). The current
`valence:+0.475 / arousal:+1.000` should not make every syllable share the same Yelp
identity — reserve the whistle/yelp for the opener and ending accent and let the middle
rotate chatter/stutter/tremble variants.

### 5. Treat the standalone dash as a control marker

For this exact phrase, the standalone `-` should become a hesitation control marker
(`--`, em dash, and `...` likewise) with a deterministic pause, instead of a voiced
semantic token. That alone makes the prompt's intended pacing more legible and is a small,
self-contained change.

### 6. Add a second vocal-formant stage for mammalian warmth

The design already identifies BB-8's production chain as formant synth plus talkbox-like
vocal shaping. A future voice could add a lightweight second stage after the existing
formant bank:

- a broad, moving mouth filter with a deterministic open-close envelope per gesture;
- optional breath/noise excitation into that stage for moans and inquisitive holds
  (overlaps with #1's roughness work);
- mild saturation or soft clipping before the final envelope to reduce pure periodicity.

This should be bounded and subtle so the output stays droid-like rather than becoming
TTS-like.

### 7. Track contextual-clip acceptance separately from golden determinism

The golden WAV hashes should remain the sample-level contract. Separately, add a
directional acceptance note for this clip and phrase, similar to the V5/V6 forensic notes.
Useful directional metrics:

- max internal gap and active-island sequence shape;
- **dominant peak range and the fraction of active frames above ~1.1 kHz** (today 0%;
  the reference is 18%);
- active frame fraction (today 0.63; reference 0.44);
- harmonicity median and IQR;
- 2-5 kHz burstiness, especially p90/max compared with median;
- whether the standalone dash is control-only;
- by-ear check for "opener, wait, answer" staging.

These metrics should not become hard CI thresholds. They are tuning instruments that keep
the work pointed at the desired perception.

## Suggested Implementation Order

The ordering is deliberately primitives-before-orchestration: the planner has nothing
dramatic to schedule until the synth can whistle, go silent, and roughen.

1. **VOICE_V7 scope note:** document that the next voice version targets contextual
   performance _and_ expanded synthesis dynamic range, not another cleanup of
   word-boundary smoothing.
2. **Synthesis dynamic range:** add a swept-oscillator chirp/whistle gesture, a wider
   per-gesture pitch span, and a noise/breath excitation blend so harmonicity can swing.
3. **Timing primitives:** raise the pause ceiling for selected arcs and make word-boundary
   bridging suppressible.
4. **Dash/ellipsis prosody:** treat standalone `-`, `--`, em dash, and `...` as
   control-only hesitation markers with deterministic pauses.
5. **Local performance planner:** add phrase roles and local arousal/archetype curves,
   with `--explain` rows for role decisions, deploying the gestures from steps 2–3.
6. **Archetype contrast + event-based sparkle:** prevent high positive arousal from
   selecting `Yelp` for an entire utterance; reserve whistle/yelp for accents; reduce
   constant sparkle and carry brightness with the swept oscillator.
7. **Second vocal stage:** prototype a subtle mouth-filter/noise/saturation layer for
   held inquisitive and moan-like gestures.
8. **Contextual acceptance doc:** regenerate the exact comparison and write a V7
   acceptance note before freezing hashes.

## Non-Recommendations

- Do not raise global brightness. This render is already brighter than the reference in
  median centroid and constant 2-5 kHz energy. The reference's brightness is a swept tonal
  peak, not a higher noise floor.
- Do not add nondeterministic randomness. Authored deterministic variation is enough and
  preserves the core promise.
- Do not change the semantic PCA mapping for this problem. The missing quality is in
  synthesis dynamic range and performance planning.
- Do not make all punctuation pauses much longer. The reference needs contrastive
  staging, not uniformly slower speech.
- Do not treat the performance planner as the whole fix. It is necessary, but it can only
  schedule gestures the synthesis instrument is able to produce; two of this clip's three
  defining signatures need new primitives first.
