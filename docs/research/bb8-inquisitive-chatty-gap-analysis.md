# BB-8 Inquisitive-Then-Chatty Gap Analysis

> Status: **research / recommendation**. This note compares the local reference clip
> `/Users/skeswa/repos/anddav87/bb8-sounds/bb8-clips/inquisitive-then-chatty.mp3`
> against the current `VOICE_V6` output for
> `Hello - good morning Sandile. What are you doing today?!`.
>
> It does not change the voice contract. Every recommendation below is
> sample-affecting and would require a new voice version plus regenerated golden WAV
> hashes.

## Summary

`VOICE_V6` has solved many first-order problems from the earlier tuning passes: it is no
longer a single clean beep, word starts are smoother, and repeated phrase bridges no
longer dominate the syllable body. The remaining difference is higher level. The
reference sounds like a performed exchange: one inquisitive gesture, a long conversational
gap, then a compact chatty answer made of varied mini-gestures. The dootdoot render still
sounds like a deterministic token sequence: every voiced token goes through a similar
high-arousal treatment, with short bridged word gaps and only punctuation creating true
phrase resets.

The main causes are:

1. Pacing is planned per token and punctuation boundary, not as a phrase-level
   discourse arc.
2. Affect and archetype selection are mostly utterance-global for this input, so the
   phrase saturates into one high-energy mode.
3. Timbre variation exists, but the selected texture path is shallow and too uniform
   across adjacent syllables.
4. The "organic" quality in the reference comes from performed, irregular vocal-formant
   motion and changing periodicity; dootdoot is still a very stable oscillator/formant
   instrument with deterministic LFO motion.

The highest-leverage next step is not "make it brighter" or "add randomness." For this
clip, dootdoot is already brighter and more constant in the upper-mid band than the
reference. The next step should be a deterministic performance planner that varies phrase
roles, local arousal, gesture archetypes, duration, amplitude, and texture over time.

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
largest differences are structural and large.

## Observations

The current `--explain` output for the dootdoot phrase starts with:

```text
token │ pitch │ vowel │ contour │ warble
mood │ valence:+0.475 │ arousal:+1.000 │ - │ -
```

That matters. The input's punctuation, length, and positive terms saturate arousal to
`1.000`, so the current high-energy path is applied broadly rather than staged.

Direct comparison:

| Measurement                 | Reference clip | dootdoot render | Read                                                               |
| --------------------------- | -------------: | --------------: | ------------------------------------------------------------------ |
| Duration                    |         3.00 s |          3.32 s | Similar overall span.                                              |
| Active frame fraction       |           0.44 |            0.63 | dootdoot fills more of the file with active sound.                 |
| Active islands              |              6 |               9 | reference has fewer, more staged events.                           |
| Median active island        |         197 ms |          209 ms | median event size is similar; event ordering is the bigger gap.    |
| Max internal gap            |        1103 ms |          232 ms | reference has a true question-to-answer pause; dootdoot does not.  |
| Fixed-threshold max silence |        1151 ms |          269 ms | same conclusion under a separate `-36 dB` silence detector.        |
| Dominant peak range         |        4264 Hz |          517 Hz | reference has much larger spectral/pitch-region movement.          |
| Autocorr pitch proxy IQR    |         199 Hz |           79 Hz | dootdoot's periodic pitch region is much steadier.                 |
| Harmonicity median          |          0.904 |           0.937 | dootdoot is more cleanly periodic.                                 |
| Harmonicity IQR             |          0.207 |           0.050 | reference varies between cleaner and rougher frames more strongly. |
| Spectral centroid median    |         984 Hz |         2214 Hz | this dootdoot render is brighter overall, not darker.              |
| 2-5 kHz power share, median |         ~0.000 |           0.104 | dootdoot keeps upper-mid energy present more constantly.           |
| 2-5 kHz power share, max    |          0.834 |           0.366 | reference has rarer but more extreme upper-mid bursts.             |

The most important shape difference is visible in active islands. The reference starts
with a roughly 557 ms opening event, leaves about 1.1 s of space, then answers with
shorter bursts around 244, 151, 151, 104, and 313 ms. The dootdoot render produces nine
more evenly connected islands: about 209, 197, 569, 360, 209, 197, 186, 244, and 209 ms,
with no internal gap over roughly 232 ms.

The reference's "inquisitive then chatty" identity is therefore not just timbre. It is a
macro-shape: opener, wait, answer. Dootdoot's current phrase planner cannot infer that
kind of turn-like structure from the input, and the standalone `-` is voiced as a normal
token rather than treated as a prosodic dash.

## Cause Analysis

### 1. Pacing: the current planner lacks phrase roles

`VOICE_V6` has phrase prosody, but it is still boundary-driven. It sees words,
WordPiece continuations, and punctuation; it does not assign discourse roles such as
opening probe, reply burst, aside, hesitation, or terminal flourish.

For this prompt, the reference behaves like:

1. Inquisitive opener.
2. Long conversational gap.
3. Dense chatty response.

The dootdoot phrase behaves more like:

1. A continuous high-arousal statement until the period.
2. A normal sentence reset.
3. A high-arousal question ending.

The period adds a deterministic sentence pause, but the clip's defining pause is much
larger than dootdoot's current sentence/word timing. Also, the ASCII hyphen in the prompt
is a voiced token in `--explain`, so it consumes time and timbre as a semantic syllable
instead of acting as a hesitation or phrase separator.

### 2. Timbre: the palette exists but is selected too uniformly

The current engine has archetypes (`chatter`, `yelp`, `moan`, `stutter/burst`,
`tremble`) and texture seasoning. For this input, however, the utterance mood is
positive and fully aroused. In `archetype.rs`, positive high-arousal mood selects the
`Yelp` path before complexity can select `StutterBurst`. The result is that many
syllables share the same broad gesture family.

That explains the subjective gap: the implementation has a palette, but the selection
rule collapses this whole utterance into one high-energy color. BB-8's clip is more like
a palette performance: a longer questioning tone, then smaller chatter elements, then
little spectral and amplitude accents.

### 3. Timbre: dootdoot is constantly bright, while BB-8 is burst-bright

Earlier aggregate research correctly found that many BB-8 references need upper-mid
energy. This specific clip shows a subtler lesson: dootdoot keeps 2-5 kHz sparkle
present in most active frames, while the reference is usually lower/darker but sometimes
spikes hard into the upper-mid region.

So a simple brightness increase would move in the wrong direction. The better target is
spectral burstiness:

- darker or rounder bodies for ordinary connective syllables;
- short, high-contrast chirp or yelp accents for selected events;
- high-frequency content that appears as gestures, not as a constant layer.

### 4. Organic-ness: dootdoot is still too periodic and too regular

The reference has lower median harmonicity and much wider harmonicity variation. That
matches the listening impression of something partly vocal and performed: some frames
lock into a pitched tone, some smear, roughen, or shift formant emphasis.

Dootdoot is deterministic and smoothed, but its motion is still composed of clean,
bounded functions: oscillator phase, formant filters, compound LFO, envelope, and small
additive textures. That is good for determinism and learnability, but it can sound like a
synth patch. The "almost mammalian" quality likely requires a second layer of
vocal-tract-like behavior, not just more oscillators:

- a talkbox-like secondary formant or mouth filter after the current formant bank;
- breathy/noisy excitation mixed under the tonal source for selected gestures;
- subtle deterministic roughness that changes over a gesture rather than a steady LFO;
- more asymmetric amplitude shapes, including swells, sigh-like decays, and glottal-ish
  soft onsets.

This can remain deterministic. The target is authored irregularity, not runtime
randomness.

### 5. Semantic mapping is not the bottleneck

The semantic knobs move across tokens, and `VOICE_V6` already carries phrase, mood,
complexity, archetype, and continuity channels. The gap is not that text fails to affect
sound. The gap is that the performance channels are still too global and too shallow for
this kind of clip.

The semantic PCA layer should remain the learnable core. The next work should sit above
and around it as a performance layer.

## Recommendations

### 1. Add a deterministic discourse-performance planner

Introduce a planner that runs after tokenization and before synthesis, assigning local
phrase roles. It should be a pure function of the event stream, punctuation, word count,
and simple control tokens.

Initial roles worth supporting:

| Role                | Trigger candidates                                  | Rendering direction                                    |
| ------------------- | --------------------------------------------------- | ------------------------------------------------------ |
| `probe`             | question mark, leading short phrase, "what/why/how" | longer rising gesture, less chatter density            |
| `chatty_reply`      | phrase after a strong pause or sentence reset       | shorter events, denser burst, alternating archetypes   |
| `hesitation`        | standalone dash, ellipsis, repeated punctuation     | real pause or quiet rounded connector, no voiced token |
| `terminal_flourish` | final `?!`, `!`, or `?`                             | one accented yelp/chirp, not all syllables yelped      |
| `aside`             | comma/colon-delimited short segment                 | lower volume, rounder/darker body, shorter pitch span  |

For this exact phrase, the standalone `-` should probably become a hesitation control
marker instead of a voiced semantic token. That alone would make the prompt's intended
pacing more legible.

### 2. Make affect and archetype local, not only utterance-global

Keep the utterance-level mood row, but compute per-phrase and per-syllable performance
curves:

- arousal attack, hold, and release over a phrase;
- final-marker accent isolated near `?!`;
- local valence from nearby tokens rather than one pooled score for every syllable;
- archetype rotation or contrast rules inside high-arousal phrases.

The current `valence:+0.475 / arousal:+1.000` should not make every syllable share the
same high-energy yelp identity. A better performance would reserve the yelp for the
opening or ending accent and let the middle use chatter/stutter/tremble variants.

### 3. Add phrase-scale timing contrast

Add bounded timing mechanisms that can create the reference's macro-shape:

- strong hesitation gaps in the 600-1200 ms range for explicit dash/ellipsis or selected
  question-to-answer arcs;
- denser follow-up clusters with 30-80 ms internal rests;
- phrase-final lengthening that can go beyond the current sentence lengthening when a
  phrase role asks for a held inquisitive gesture;
- amplitude tails that can occupy space without counting as another token syllable.

This should be handled by the planner, not by increasing every punctuation pause.
Uniformly longer pauses would make simple sentences sluggish without creating the
opener-gap-answer shape.

### 4. Make upper-mid texture event-based

Revise the upper-mid sparkle layer from a mostly present layer into a gesture resource:

- lower default upper-mid mix for ordinary word-connected syllables;
- short chirp bursts for yelps, terminal flourishes, and selected chatter notes;
- per-gesture spectral envelopes so brightness has attack and decay;
- keep >6 kHz modest, consistent with previous research and this clip.

Success should sound less like "constant gleam" and more like "small bright droid
articulations inside a rounder voice."

### 5. Add a second vocal-formant stage for mammalian warmth

The design already identifies BB-8's production chain as formant synth plus talkbox-like
vocal shaping. Dootdoot currently has one formant bank and additive texture. A future
voice could add a lightweight second stage:

- a broad, moving mouth filter after the existing formant bank;
- a deterministic open-close envelope per gesture;
- optional breath/noise excitation into that stage for moans and inquisitive holds;
- mild saturation or soft clipping before the final envelope to reduce pure periodicity.

This is likely the most direct path to the "almost mammalian" quality. It should be
bounded and subtle so the output stays droid-like rather than becoming TTS-like.

### 6. Track contextual-clip acceptance separately from golden determinism

The golden WAV hashes should remain the sample-level contract. Separately, add a
directional acceptance note for this clip and phrase, similar to the V5/V6 forensic
notes. Useful directional metrics:

- max internal gap and active-island sequence shape;
- dominant peak range;
- harmonicity median and IQR;
- 2-5 kHz burstiness, especially p90/max compared with median;
- whether standalone dash is control-only;
- by-ear check for "opener, wait, answer" staging.

These metrics should not become hard CI thresholds. They are tuning instruments that
keep the work pointed at the desired perception.

## Suggested Implementation Order

1. **VOICE_V7 scope note:** document that the next voice version targets contextual
   performance, not another cleanup of word-boundary smoothing.
2. **Dash/ellipsis prosody:** treat standalone `-`, `--`, em dash, and `...` as
   control-only hesitation markers with deterministic pauses.
3. **Local performance planner:** add phrase roles and local arousal/archetype curves,
   with `--explain` rows for role decisions.
4. **Archetype contrast:** prevent high positive arousal from selecting `Yelp` for an
   entire utterance; reserve yelps for accents and rotate chatty/stutter gestures inside
   dense segments.
5. **Event-based sparkle:** reduce constant sparkle and add brighter short chirp
   accents.
6. **Second vocal stage:** prototype a subtle mouth-filter/noise/saturation layer for
   held inquisitive and moan-like gestures.
7. **Contextual acceptance doc:** regenerate the exact comparison and write a V7
   acceptance note before freezing hashes.

## Non-Recommendations

- Do not raise global brightness. This render is already brighter than the reference in
  median centroid and constant 2-5 kHz energy.
- Do not add nondeterministic randomness. Authored deterministic variation is enough and
  preserves the core promise.
- Do not change the semantic PCA mapping for this problem. The missing quality is in
  performance planning and synthesis texture.
- Do not make all punctuation pauses much longer. The reference needs contrastive
  staging, not uniformly slower speech.
