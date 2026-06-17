# BB-8 Sound Signature Analysis

This report compares the current dootdoot synthesis output against a local BB-8
reference set of source recordings. It is intended as input to
Phase 7 voice tuning, especially T-45 and T-47. It does not complete T-45 by itself,
because that task still requires by-ear tuning against the target identity.

## Method

Reference material:

- 32 MP3 files from the source recordings, decoded to mono
  44.1 kHz WAV (via `ffmpeg -ac 1 -ar 44100`) for analysis. 31 are named
  `bb8-*`; one is `bb2-02`. The clips are not uniform: durations range from
  roughly 0.8 s to ~15 s, and several are long multi-burst performances rather
  than single utterances, so the duration median understates how long the
  longest references run.
- Six dootdoot clips rendered through the current CLI:
  - `hello there`
  - `where are you`
  - `this is very exciting`
  - `cat dog airplane`
  - `playing`
  - `?`

Measurements were frame-level diagnostics, not perceptual ground truth: RMS
segmentation, active-island duration, spectral centroid, spectral rolloff,
dominant-frequency tracks, autocorrelation harmonic salience, and broad energy bands.
They are useful for explaining why the current output misses the target, but final
tuning still needs listening.

Metric conventions (so the numbers below are reproducible). Frames are
Hann-windowed, 2048 samples with a 512-sample hop. A frame is "active" when its
RMS exceeds a relative noise gate (a fixed fraction of the clip's peak RMS);
spectral metrics are taken over active frames only. **Spectral centroid and 85%
rolloff are computed on the magnitude spectrum** (the common `librosa` default);
recomputing them on the power spectrum yields much lower numbers (e.g. BB-8
centroid ~857 Hz instead of ~2407 Hz), so the convention matters when these are
re-run. **Band-power shares are computed on the power spectrum.** Two metrics —
active fraction and active-island duration — depend on the exact gate threshold
and should be read as directional (BB-8 is sparser, with roughly 2x longer
islands), not as hard targets; the other metrics reproduce to within rounding
across reasonable settings. BB-8 brightness mainly lives in the 2-5 kHz upper-mid
region, not above 6 kHz.

## Executive Summary

The current dootdoot voice is structurally correct for the design sketch: it is a
deterministic formant synth with portamento, warble, a high register, and a faint
electronic edge. The problem is that those ingredients are arranged too cleanly and too
narrowly. The result is closer to a short, periodic, vowel-filtered synth syllable than
to the official BB-8 sound-effect vocabulary.

The biggest gaps are:

1. BB-8 sounds like layered performed gestures; dootdoot sounds like one repeated
   oscillator/filter patch.
2. BB-8 has broad pitch excursions and mixed registers; dootdoot stays in a narrow
   fundamental range centered near 880 Hz.
3. BB-8 has strong low body plus upper-mid brightness; dootdoot is concentrated
   in the 500-2000 Hz band and has almost no sub-500 Hz energy. BB-8's brightness
   lives mostly in the 2-5 kHz region (its 85% rolloff is ~4840 Hz), not above
   6 kHz — both BB-8 and dootdoot carry almost no energy over 6 kHz.
4. BB-8 is less perfectly periodic; dootdoot is almost perfectly harmonic frame to frame.
5. BB-8 phrase timing has uneven chirps, tails, and rests; dootdoot has dense,
   fixed-duration token syllables.
6. BB-8 formants move like a performed vocal tract/talkbox; dootdoot holds a static vowel
   per syllable.

The semantic layer is probably not the primary culprit. The `--explain` output shows the
four knobs moving across tokens. The missing identity lives mostly in the fixed voice DNA:
source, formant motion, modulation, envelope, and phrase/rhythm templates.

## Measured Differences

Aggregate medians from the local analysis:

| Metric                      | BB-8 references | dootdoot clips | Interpretation                                     |
| --------------------------- | --------------- | -------------- | -------------------------------------------------- |
| Clip duration               | 1.52 s          | 0.59 s         | dootdoot phrases are much shorter (but see note)   |
| Active audio fraction       | 0.53            | 0.81           | dootdoot is denser, with less air (gate-dependent) |
| Active island duration      | 522 ms          | 209 ms         | dootdoot gestures are token-sized (gate-dependent) |
| Median spectral centroid    | 2407 Hz         | 1839 Hz        | references are brighter overall (magnitude spec)   |
| 85% spectral rolloff        | 4840 Hz         | 2584 Hz        | dootdoot loses upper-mid energy (magnitude spec)   |
| Dominant-frequency span     | 476 Hz          | 213 Hz         | dootdoot dominant-peak motion is smaller           |
| Autocorrelation harmonicity | 0.75            | 0.96           | dootdoot is too perfectly periodic                 |
| Sub-500 Hz power share      | 0.49            | ~0.00          | dootdoot lacks low body                            |
| 500-2000 Hz power share     | 0.29            | 0.92           | dootdoot over-focuses the mid register             |
| Over-6000 Hz power share    | 0.003           | ~0.00          | negligible in both; not a real differentiator      |

The clip-duration comparison mixes long multi-burst reference performances (up to
~15 s) against short 1-4 word dootdoot test inputs, so part of the gap reflects the
chosen test corpus, not only the synth. The conclusion that dootdoot renders shorter
per token still holds; the specific 1.52 s figure is a property of this reference set.
The dominant-frequency span tracks the strongest spectral peak per frame, which for
formant-filtered audio is often a formant rather than the fundamental, so it indexes
overall gesture motion rather than pitch alone.

Some reference clips are extreme outliers in ways that are important to the identity:
`bb8-27` and `bb8-28` have much higher spectral centroids and lower harmonic salience,
which indicates noisy or highly inharmonic effects. Several long clips, such as
`bb8-09`, `bb8-10`, and `bb8-23`, are multi-burst phrases rather than a single short
utterance. The official set is not one homogeneous "voice patch"; it is a family of
related droid gestures.

Current dootdoot constants explain the measured shape:

- Pitch center is `880 Hz +/- 7 semitones`, or roughly 587-1318 Hz before warble.
- Warble is a single fixed 8.5 Hz sinusoid, up to 45 cents.
- Ring modulation is only 72 Hz at 8% wet mix.
- Each voiced token is exactly 150 ms, with 80 ms word pauses.
- The source is a 65% saw / 35% pulse oscillator through three static formant filters.
- The formant filter bank strongly emphasizes the first formant, which keeps energy in
  the speech-mid band.

## First-Principles Cause Analysis

### 1. Exciter: too much stable harmonic oscillator, not enough droid event

dootdoot begins with a band-limited saw/pulse source. That source is periodic by design.
After formant filtering, it still carries a clear harmonic structure. This is why the
current harmonic salience is near 0.96.

BB-8 does use pitched material, but the reference clips behave more like composite sound
events: whistled tonal cores, formant articulations, transient chirps, occasional noisy
or inharmonic bursts, and small mechanical/electronic artifacts. The sound is still
pitched enough to feel vocal, but not as cleanly periodic as a single oscillator.

Implication: keep a pitched core, but stop making the pitched core the whole sound. The
voice needs at least a transient/noise layer and a separate high chirp/sparkle layer.

### 2. Register: fundamental is high, but the spectrum is not bright

The current pitch register is centered around 880 Hz. That is high relative to human
speech fundamentals, but the analyzed dootdoot clips still have little energy below
500 Hz and almost none above 6 kHz. This produces a narrow "midrange vowel synth" result.

The references often have dominant low components around 300-600 Hz while also carrying
upper energy out to several kilohertz. That combination is more BB-8-like than simply
"higher pitch": it has body plus gleam.

Implication: the fix is not just raising pitch. Dootdoot needs a wider spectral stack:
lower tonal body, animated mid formants, and controlled high-frequency air/sparkle.

### 3. Pitch motion: portamento exists, but gestures are too small and too uniform

BB-8's emotional identity comes from large non-linear swoops. The current pitch range is
bounded to a 14-semitone total semantic span, and most real inputs do not hit the
extremes. For common text, the resulting dominant-frequency track often moves only a few
hundred hertz.

Also, portamento only glides from previous token pitch to target token pitch during a
fixed 45 ms window. If two tokens map near each other, there is little audible swoop. A
single-token input has no inter-token movement except the fixed empty chirp or punctuation
final glide.

Implication: BB-8 likeness needs internal gesture curves inside each syllable, not only
between tokens. A syllable should have a designed pitch contour even when there is no
neighboring token.

### 4. Formants: static vowel per syllable misses the talkbox behavior

The design correctly identifies formants as load-bearing. The current implementation,
however, computes one vowel position per token and holds the formant centers static for
the whole syllable. Only pitch moves continuously.

A talkbox-like sound is a moving vocal tract. BB-8's vowel identity comes from formants
that sweep, pinch, open, and close during the gesture. Static formants make dootdoot
sound like a filtered oscillator saying one frozen vowel.

Implication: vowel position should become a time-varying trajectory. The semantic vowel
knob can still choose the locus, but the fixed DNA should impose a small vowel-motion
template per syllable.

### 5. Timing: token syllables are too regular and too dense

The reference set has much more air. Median active fraction is about 0.53 for BB-8 and
0.81 for the generated dootdoot clips. Reference clips also have longer active islands
and more varied pauses, while dootdoot uses exact 150 ms token bursts and fixed pauses.

This makes dootdoot feel like a tokenizer metronome. BB-8 sounds more like a performed
phrase made of uneven chirps, tails, and rests.

Implication: the design can keep deterministic timing, but the fixed timing template
needs more phrasing. A token can render as a shaped micro-phrase rather than one flat
150 ms note.

### 6. Envelope: attack and sustain are too synth-like

The current envelope is a 12 ms attack, 80 ms decay, 35% sustain, and 25 ms release over
a 150 ms syllable. That creates a quick onset followed by a steady held tone.

Many BB-8 clips have more elaborate amplitude shapes: chirped attacks, ramps, dips,
short repeated pulses, and longer decaying tails. The envelope often participates in the
meaning of the gesture; it is not just a note gate.

Implication: replace the simple ADSR-like envelope with a droid gesture envelope:
asymmetric attack, internal pulse/dip, and a tail that can ring after the tonal core.

### 7. Warble: fixed sine vibrato is too regular

The current warble is a single 8.5 Hz sinusoid, restarted per syllable. That gives a
predictable vibrato. BB-8's warble is more like expressive hand motion: uneven,
compound, and sometimes fluttery.

Implication: use a deterministic but richer LFO stack: slow drift plus faster flutter,
with phase carried across the utterance or seeded deterministically from position. The
semantic warble knob should control amount, not make the modulation mechanically uniform.

### 8. Electronic edge: 72 Hz ring-mod is the wrong dominant artifact

The 72 Hz, 8% ring mod adds low-rate tremolo/sidebands. It is audible as an electronic
seasoning, but it does not create the crisp high chirp, servo-like, or inharmonic texture
present in parts of the reference set.

Implication: the electronic layer should include higher-rate or pitch-related sidebands,
short resonant blips, and/or very light saturation. Ring mod can remain, but it should
not be the only non-organic cue.

## Recommended Tuning Direction

The next tuning pass should treat dootdoot as a small layered droid instrument, not a
single oscillator through a vowel filter.

Recommended fixed voice DNA:

1. Tonal core:
   - Broaden usable pitch motion beyond the current common-case range.
   - Add an internal pitch contour per syllable, so one-token inputs still swoop.
   - Consider a lower core component around the 300-700 Hz region for body.

2. Moving formants:
   - Keep semantic vowel position as the target color.
   - Add a fixed time-varying vowel trajectory around that target.
   - Reduce the "static human vowel" impression by moving formants independently of
     pitch.

3. Transient/noise layer:
   - Add a short band-passed noise or click component at attacks.
   - Add a very small breath/noise tail or filtered hiss for official-sound-effect
     texture.
   - Keep it deterministic and bounded so the output does not become random noise.

4. High chirp/sparkle layer:
   - Add controlled energy primarily in the 2-5 kHz band, which is where the
     references' actual brightness lives (rolloff ~4840 Hz); keep only modest
     content above 6 kHz, since the references carry almost none there either.
   - Make this layer gesture-shaped, not constant broadband brightness.

5. Richer modulation:
   - Replace single-rate sine warble with compound deterministic LFOs.
   - Carry modulation phase across syllables to avoid every token beginning the same way.
   - Let the warble knob scale depth and complexity.

6. Phrasing:
   - Increase air between phrase units or reduce active density.
   - Let a token render as a short micro-gesture with internal motion, not only a held
     note.
   - Preserve deterministic timing; the improvement should come from fixed templates, not
     runtime randomness.

7. Electronic sidebands:
   - Move some electronic edge from low 72 Hz tremolo toward brighter, pitch-related
     sidebands or short resonant chirps.
   - Keep ring modulation faint; BB-8 should stay vocal first, electronic second.

## Likely Implementation Order

For the next Phase 7 work, the highest-leverage order is:

1. Add internal pitch and vowel trajectories to `render_syllable_with_final_glide`.
2. Add a deterministic transient/noise layer and high sparkle layer.
3. Broaden pitch/register constants and rebalance formant gains.
4. Replace simple sine warble with compound deterministic modulation.
5. Re-run the same metrics and then tune by ear against the reference clips.

This order attacks the core identity first. Adjusting semantic PCA ranges or squash
statistics before fixing the voice DNA is likely premature: the current semantic knobs
already move, but the synthesizer turns them into a too-clean, too-regular sound.

## Format Note

All recommended changes alter output samples. Because T-48 has not locked the final
format yet, these can still be treated as part of pre-freeze `VOICE_V1` tuning. Once
the format is locked, the same changes would require a version bump and regenerated
golden fixtures.
