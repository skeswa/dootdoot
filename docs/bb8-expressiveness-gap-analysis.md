# BB-8 Expressiveness Gap Analysis

> Status: **analysis / exploration** (pre-decision). This document studies the
> distance between what dootdoot synthesizes today and the expressive, in-context
> BB-8 voice we want, and lays out the technical mechanisms that could close that
> distance. It is a successor to
> [`bb8-sound-signature-analysis.md`](./bb8-sound-signature-analysis.md), which
> covered the **per-syllable timbre** gap that Phase 7 (T-45–T-54) closed. This
> document covers the next layer up: **phrasing, word structure, emotion, and
> texture** — the things that make BB-8 read as _communicating_ rather than
> _beeping_.
>
> It does **not** decide anything or change `FORMAT_V1`. It is input to a future
> planning pass. Where a proposal would alter output samples, this document says so.
> Closing these gaps belongs in **`FORMAT_V2`** (§8) and, in a few places, requires
> revisiting normative requirements fixed in v1.

---

## 1. The four gaps, stated precisely

After Phase 7, dootdoot is a structurally faithful BB-8 _timbre_: a deterministic
formant voice with portamento, compound warble, body, sparkle, and a faint
electronic edge (see `voice-tuning.md`). The remaining shortfall is expression and
structure. The user named four gaps:

| #     | Gap (as observed)                                                                                         | One-line technical restatement                                                                                                           |
| ----- | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| **A** | "Everything sounds staccato and uniformly paced — no flow, conjunction, or pausing."                      | No **phrase-level prosody**: durations, pauses, and pitch baseline are uniform and context-free.                                         |
| **B** | "All known words read as one distinct sound; simple words should sound singular, complex words compound." | Word **complexity is not mapped to sonic complexity** beyond the coarse WordPiece split; each syllable is one clean gesture.             |
| **C** | "No sentiment — sad text should sound morose, exciting text faster / more inflected."                     | No **affect channel**: the 4 PCA axes carry lexical semantics, not emotional valence/arousal, and nothing drives prosody from sentiment. |
| **D** | "BB-8 mixes in other kinds of sounds for texture and expression."                                         | A **fixed, narrow timbre palette**: one syllable archetype, no contrastive gesture types (yelps, moans, stutters, servo blips).          |

These gaps overlap. A, C, and D share the same root: dootdoot renders every token
through **one** fixed syllable archetype and lays them out on a **metronome**. The
only per-token performance variation comes from four bounded semantic knobs tuned for
_learnability_, not _performance_. Section 7 proposes a single architecture that
addresses all four; Sections 3–6 analyze them one at a time.

---

## 2. What dootdoot does today (the precise baseline)

So the gaps are measured against ground truth, here is the exact current pipeline
for the structural/expressive layer (timbre internals are in
`bb8-sound-signature-analysis.md`).

### 2.1 Token → knob (`engine.rs`, `mapping.rs`)

1. Tokenize (WordPiece, uncased, `add_special_tokens=false`); drop control tokens;
   recognize prosodic punctuation `.!?,;:` as control markers (`engine.rs:98`).
2. Every voiced token → a dequantized 4-axis `TokenVector` (`mapping.rs:79`).
3. **One** sequence baseline = weight-scaled mean of all token vectors, squashed
   (`engine.rs:146`, `mapping.rs:267`). This is the utterance's single "mood" center.
4. Each token's knobs = `clamp(B + α·(T − B))` around that baseline
   (`mapping.rs:291`), `α = [0.85, 0.90, 1.10, 1.20]` for pitch/vowel/contour/warble.

The 4 knobs are the only **per-voiced-token semantic** quantities that drive the synth.
Their ranges are `[-1, 1]`. Text still affects structure through token count,
continuation flags, prosodic punctuation, and syllable position; the missing channel is
not "text dependence" in general, but text-dependent **performance** beyond the semantic
knobs.

### 2.2 Knob → syllable (`synth.rs`)

`render_syllable_with_final_glide` (`synth.rs:491`) renders **exactly one**
`BASE_SYLLABLE_SAMPLES = 7497` (170 ms) buffer per token, identical structure every
time: source osc → 3 formants (vowel trajectory) → body + attack transient + sparkle
→ ring-mod → one amplitude envelope. Internal motion (pitch swoop, vowel bloom,
compound warble) exists but is a **fixed micro-gesture template** scaled by the knobs.

### 2.3 Syllable → utterance (`sequence.rs`)

`sequence_utterance` (`sequence.rs:203`) concatenates syllables with **fixed** gaps:

- `WORD_PAUSE_SAMPLES = 4851` (110 ms) between words; **zero** between WordPiece
  continuation subtokens (`sequence.rs:233`).
- Punctuation → a fixed final glide on the prior syllable + a fixed long/medium pause.
- Fixed 30 ms lead / 90 ms trail silence.
- Portamento carries the _previous token's target pitch_ into the next syllable's
  45 ms glide (`sequence.rs:218`).

### 2.4 The contracts that bear on the four gaps

These are frozen in v1 and are what the four gaps push against:

| Frozen decision                               | Where                | Consequence for expression                                  |
| --------------------------------------------- | -------------------- | ----------------------------------------------------------- |
| **Single fixed syllable duration** (V1 impl.) | every token = 170 ms | uniform rhythm; no emphasis, no lengthening (Gap A)         |
| **Single fixed inter-word pause** (V1 impl.)  | 110 ms, always       | metronomic phrasing; no clause/breath structure (Gap A)     |
| **Only the 4 axes vary** (NFR-16)             | `mapping.rs`         | no channel for affect or complexity (Gaps B, C)             |
| **One syllable archetype** (FR-15/17)         | `render_syllable*`   | no contrastive gesture types (Gap D)                        |
| **Single utterance baseline**                 | `engine.rs:146`      | no pitch declination, reset, or arc across a phrase (Gap A) |

None of these is wrong: each buys determinism, droid identity, or learnability.
Together, they define the limits the requested expressiveness now runs into. Closing
the gaps is therefore a **`FORMAT_V2`** conversation that revisits a few of these
choices (§8).

> **Spec update (resolved):** the requirement that fixed the syllable duration (**FR-20**)
> has been **removed**, and the fixed-inter-word-pause requirement (**FR-22**) has been
> **revised** to allow deterministic, boundary-dependent pauses. Variable duration and
> pacing are therefore no longer blocked at the requirements level; the top two rows above
> are now `FORMAT_V1` _implementation_ choices, free to change in `FORMAT_V2`. The
> remaining policy questions (NFR-16, archetype palette) are unchanged. §8 reflects this.

---

## 3. What "real BB-8" actually does (the evidence)

Two evidence streams: how the voice was produced, and what the audio measures.

### 3.1 Production: emotion is authored in _language_, then transcoded

The production model matters for dootdoot's architecture: **BB-8's emotion and timing
were authored as English, then converted to chirps.** During production,
Ben Schwartz wrote and recorded English dialogue for BB-8's scenes; editor Lindsey
Alvarez cut it to picture to "establish the timing patterns for how the droid
communicated," and only then did J.J. Abrams improvise synth takes (a Bebot-style X/Y
touch synth) live to that cut, fed through a talkbox performed by Bill Hader for a second
layer of live vowel formants.
([SlashFilm](https://www.slashfilm.com/542580/bb-8-voice/),
[Post Magazine](https://www.postmagazine.com/Publications/Post-Magazine/2016/January-1-2016/Sound-Editing-Star-Wars-The-Force-Awakens.aspx),
[Time](https://time.com/4151880/bb-8-voice-star-wars/))

R2-D2 was built the same way: Ben Burtt wrote equivalent English lines and "performed"
them through an ARP 2600 (self-oscillating filter + sample-and-hold + ring mod, pitch
slides via the glide/slew processor), describing the result as ~50% human performance,
50% machine — "because there was a human performance in it, you had a sense of R2
being alive."
([Hollywood Reporter](https://www.hollywoodreporter.com/movies/movie-news/locarno-award-ben-burtt-star-wars-sound-designer-1235959811/),
[Attack Magazine](https://www.attackmagazine.com/technique/hardware-focus/hardware-wars-the-gear-behind-the-sounds-of-star-wars/))
The same English-first pattern recurs in Simlish (emotional timing locked with a
non-verbal pass _before_ gibberish is layered)
([ACMI](https://www.acmi.net.au/stories-and-ideas/simlish-sound-and-the-performance-of-emotion-in-the-sims/)).

**Why this matters:** dootdoot already fits this model. Text is the emotional and
timing script; synthesis is the transcode. The gap is that dootdoot currently
transcodes only the _lexical identity_ of each token and discards the two things the
human performers actually rendered: **emotional state** (Gap C) and **conversational
timing/phrasing** (Gap A). The architecture is validated; the channels are missing.

One more production note: emotion was carried primarily by **pitch contour and
rhythm**, not timbre — higher/rising = positive/curious, lower/falling = sad/cautious,
abrupt bursts = alarm; BB-8's famous "little sad moan" on learning Poe won't return is
a single short descending gesture. BB-8 is a _chirp_ (bird-like, child-like, emotive)
against R2's mechanical _beep_.

### 3.2 Acoustics: what the contextual clips measure

We analyzed the full local sample set in two layers:

- The 32 clean top-level MP3 references, using the committed `scripts/bb8-metrics`
  workflow. These reproduce the Phase 7 comparison numbers: BB-8 has longer active
  islands (median ≈ 290 ms vs dootdoot ≈ 186 ms), much wider dominant-peak motion
  (≈ 1335 Hz vs ≈ 668 Hz), and lower harmonicity (≈ 0.81 vs ≈ 0.95). Those aggregate
  metrics are reliable enough for directional tuning.
- The seven emotionally-labeled contextual clips in
  `anddav87/bb8-sounds/bb8-clips/`, decoded to mono 44.1 kHz and checked with the same
  RMS-gated metrics plus rough autocorrelation pitch tracking. These are useful
  evidence for _communicative context_, but they are not clean lab recordings: music,
  SFX, and dialogue beds keep many frames "active" (the context-only run measured an
  active fraction near 1.0), and formant-heavy droid audio can fool simple F0 tracking.
  The table below is directional, not a set of golden acoustic constants.

**Neutral baseline (clean refs):** rough autocorrelation over the clean set clusters
around **330 Hz (E4-ish)**, and by-ear inspection plus dominant-peak tracking show large
internal motion inside many continuous bursts. Pitch-span and micro-inflection counts
depend on tracker settings, but the structural point holds: a neutral BB-8 "word" is
rarely a static tone; it is usually a compact compound of micro-swoops.

**Per-emotion signature** (directional, not exact F0 ground truth):

| Emotion / clip                  | Directional acoustic read                        | One-line identity                   |
| ------------------------------- | ------------------------------------------------ | ----------------------------------- |
| **Sad** (`lost-friends-sad`)    | low/dark, sparse, little internal animation      | stripped-down low moan/blip         |
| **Excited explanation**         | higher, denser, broad motion, repeated chirps    | fast animated babble                |
| **Found/fixed excitement**      | gushing density, bright relative to nearby clips | excited up-swooping chatter         |
| **Anxious** (`left-behind`)     | rougher/less harmonic, unresolved, low-to-mid    | trembling unresolved complaint      |
| **Surprise** (`explosion`)      | single high yelp/stab, little phrase development | one bare gesture at extreme urgency |
| **Alarm** (`enemy-approaching`) | repeated sharp stabs with wide dominant motion   | urgent repeated warning             |
| **Inquisitive then chatty**     | separated opening gesture, then denser follow-up | question-to-response phrase arc     |

The table points to three structural lessons:

1. **Emotion occupies distinct corners of a small acoustic space** whose axes are
   the same affective-prosody axes (§3.3): register, span, rate/density, glide
   speed, warble depth, brightness. These map onto dootdoot's existing knobs
   _plus_ timing — which is why an affect channel is tractable (§6).
2. **Pacing is itself an emotional channel.** Dense, nearly continuous chatter reads as
   excitement; separated repeated stabs read as alarm; a pause between an inquisitive
   opening and a chatty follow-up reads as a phrase arc. dootdoot's single fixed 110 ms
   word pause cannot express any of this.
3. **Compounding is normal, and _de_-compounding is meaningful.** Single-gesture
   archetypes exist, but they are expressive markers such as surprise yelps or sad
   stripped-down moans. dootdoot has the opposite default: every token is one clean
   gesture regardless of complexity or affect.

### 3.3 The affective-prosody literature (how emotion → acoustics, generally)

The clip findings line up with the speech-emotion literature, which gives us
_directional_ parameter targets we can apply deterministically. Consolidated from
Murray & Arnott (1993), Banse & Scherer (1996), and the Juslin & Laukka (2003)
meta-analysis (104 studies):

| Parameter              | Sadness       | Excitement / Joy | Anger            | Fear / Anxiety | Surprise         |
| ---------------------- | ------------- | ---------------- | ---------------- | -------------- | ---------------- |
| Speech rate            | slower        | faster           | slightly faster  | much faster    | faster           |
| Mean pitch (F0)        | lower         | much higher      | higher           | very high      | much higher      |
| Pitch range            | narrower      | much wider       | much wider       | much wider     | wider            |
| Contour                | falling       | rising           | downward, abrupt | rising         | sharp rise→fall  |
| Intensity              | softer        | higher           | much higher      | mixed          | higher           |
| Pauses                 | more / longer | fewer            | fewer            | fewer          | brief then burst |
| Voice quality          | lax, breathy  | bright, tense    | harsh, tense     | irregular      | tense            |
| HF energy / brightness | less          | more             | much more        | more           | more             |
| Jitter (micro-tremor)  | low           | low              | high             | high           | moderate         |

Sources:
[Banse & Scherer 1996 (PDF)](http://www.columbia.edu/~rmk7/HC/HC_Readings/Scherer.pdf),
[Juslin & Laukka 2003 (PDF)](https://www.brainmusic.org/EducationalActivities/Juslin_emotion2003.pdf),
[Scherer 2004 (PDF)](https://www.isca-archive.org/speechprosody_2004/scherer04_speechprosody.pdf).
(Murray & Arnott directions are corroborated across these; cite verbatim adjectives as
"after Murray & Arnott 1993.")

The design point: mean F0, F0 range, rate, intensity, and brightness primarily index
physiological **arousal**, not emotion identity. That is why anger/fear/joy look
acoustically similar (all high-arousal) and sadness/tenderness cluster low.
**Valence is the harder, finer cue**, carried by contour direction
(falling = negative, rising = positive), voice-quality texture (tense vs breathy), and
micro-regularity (jitter). Engineering guidance: render arousal with several parameters
moving together; render valence with contour and texture. The local clips follow the
same pattern: the excited/sad contrast is large and multi-parameter, while same-arousal
alarm/surprise/anxious separate mostly on contour and texture.

---

## 4. Gap A — pacing, flow, and pausing (phrase-level prosody)

### 4.1 The mechanism gap

dootdoot has **no phrase model**. Every syllable is 170 ms; every word gap is 110 ms;
the pitch baseline is one constant for the whole utterance. There is no declination,
no pitch reset at boundaries, no pre-boundary lengthening, no emphasis, no breath
groups. Natural speech — and BB-8 — is the opposite of uniform.

The TTS literature names the exact deterministic, table-drivable levers that separate
"flowing" from "robotic," all of which dootdoot lacks:

- **Phrase-final lengthening (Klatt's ×1.4 rule):** the syllable before a boundary is
  lengthened ~40%. This is a primary cue distinguishing flowing from staccato
  speech. ([Penn prosodic duration notes](https://www.ling.upenn.edu/courses/ling620/ProsodicDuration.html))
- **Declination + final lowering:** F0 drifts gradually down across an utterance (tens
  of Hz/s), with an upward **pitch reset** at phrase boundaries and an extra drop at the
  very end. Listeners _expect_ declination; flat F0 reads as unnatural.
  ([Pierrehumbert 1979 (PDF)](https://www.phon.ox.ac.uk/jpierrehumbert/publications/f0_declination.pdf))
- **Break-indexed pauses:** pause length scales with boundary strength — minor phrase
  ≈ 150 ms, comma ≈ 250–500 ms, sentence ≈ 0.4–1 s.
  ([Parlikar & Black 2012 (PDF)](https://www.cs.cmu.edu/~awb/papers/IS2012/738_Paper.pdf),
  [Festival tutorial](http://festvox.org/festtut/notes/festtut_6.html))
- **Continuation rise vs final fall:** clause-internal commas take a rising boundary
  tone (L-H%, "more coming"); statements fall (L-L%). dootdoot has a rough version
  (punctuation final glide) but only per-marker, not as a phrase tune.
- **Sparse prominence:** accent only _some_ tokens (nuclear stress), not all equally.
  Uniform per-token gestures are a robotic tell.

Game precedents reinforce the same lever: Animalese, Undertale, and Celeste all map
punctuation → pause and use author-/code-controlled dramatic pauses, never a fixed
inter-unit gap. ([Undertale OBJ_WRITER](https://github.com/fachinformatiker/undertale/blob/master/objects/OBJ_WRITER.object.gmx),
[Celeste/FMOD](https://www.fmod.com/docs/2.03/studio/appendix-a-celeste.html))

### 4.2 What to build

A deterministic **phrase planner** that runs between tokenization and synthesis,
producing per-token _timing and pitch-baseline modifiers_ (all functions of position
and punctuation, so still a pure function of the text):

1. **Declination curve.** Replace the single utterance baseline with a baseline that
   declines linearly (a frozen Hz/s slope) from phrase start, resets up at each
   prosodic boundary, and applies an extra final-lowering at the last syllable. This is
   a pitch _offset_ layered on the semantic pitch knob — semantics still sets relative
   pitch, the phrase sets the global arc.
2. **Variable pauses.** Make the inter-word/clause pause a function of boundary strength
   (word vs comma vs sentence) instead of a single constant — partly already present for
   punctuation; extend to clause structure and a small deterministic "breath group"
   rule (e.g. insert a phrase boundary every N words if none occurs).
3. **Pre-boundary lengthening.** Allow the syllable before a pause to render longer
   (e.g. ×1.3). FR-20 (the fixed-duration requirement) has been **removed**, so this is
   now permitted — see §8.
4. **Sparse emphasis.** Mark one token per phrase (e.g. the highest-weight or
   highest-arousal token) for a small duration/pitch-range boost.

### 4.3 Contract impact

Declination, variable pauses, and emphasis are **timing/pitch-offset templates** — they
fit the "fixed deterministic template" philosophy that Phase 7 already embraced for
micro-gestures, and require only new frozen constants. **Pre-boundary lengthening** needs
variable duration, which the removal of FR-20 now permits (revised FR-22 likewise frees
the inter-word pause). All of this remains a `FORMAT_V2` change — the v1 golden hashes are
unaffected — but it is no longer blocked at the requirements level (§8).

---

## 5. Gap B — word complexity → simple vs compound sounds

### 5.1 What we already have, and where it stops

The design _intends_ this mapping and partly delivers it: WordPiece splits rare/long
words into multiple subtokens, each rendered as a glided continuation syllable, so
`playing` → `play` + `##ing` is already two flowing syllables while `cat` is one. That
is the "frequent word = one compact syllable, rare word = multi-syllable utterance"
property from design §3.1.

It falls short in two concrete ways:

1. **The 30k WordPiece vocab keeps many moderately-complex words whole.** Common
   multi-syllable words (`airplane`, `remember`, `because`) are often a single token →
   a single 170 ms gesture, so they sound as "singular" as `cat`. Token count
   under-discriminates complexity.
2. **Each syllable is one _clean_ gesture.** Even when a word _does_ split, every
   sub-syllable is the same archetype. The reference clips often pack many pitch and
   formant inflections into one compact burst (§3.2). Internal richness, more than
   syllable count alone, is what makes a word sound "compound."

So the perception "all known words read as one distinct sound" is accurate: complexity
is quantized too coarsely (token count only) and each unit is too uniform.

### 5.2 The mechanism to add

Compute a per-token **complexity scalar** from deterministic, offline-available
signals and let it drive _internal sub-gesture count and articulation_, independent of
the semantic knobs:

- **Inputs (all deterministic):** subword-token count (already have it), character
  length, and optionally **rarity** via the Zipf frequency scale (log10
  freq-per-billion, ~1–7) from an open frequency corpus such as **SUBTLEX-US** (~74k
  words, CC-BY-SA; requires license policy before baking). Roughly
  `complexity = f(token_count, char_len, clamp(7 − Zipf))`.
  ([Zipf scale](https://www.wellformedness.com/blog/zipf-scale/),
  [SUBTLEX-US](https://github.com/chrplr/openlexicon/blob/master/datasets-info/SUBTLEX-US/README-SUBTLEXus.md))
- **Effect:** complexity selects how many internal sub-swoops/articulation points a
  syllable renders (a level-of-detail knob — the established procedural-audio pattern of
  "more input → more layers/segments";
  [Farnell, _Designing Sound_](https://designingsound.org/2012/01/18/procedural-audio-interview-with-andy-farnell/)).
  A simple/common word → one clean swoop (current behavior). A rare/long word → a
  compound of chained sub-gestures, matching the BB-8 norm. Optionally also lengthens
  the syllable for complex words (couples to Gap A's variable duration).

Sound-symbolism work points in the same direction: the bouba/kiki effect reduces to two
acoustic cues, spectral balance and **temporal continuity** (round = continuous; spiky =
segmented/discontinuous). Mapping complexity to internal segmentation uses the same
lever. ([Ćwiek et al. 2021](https://royalsocietypublishing.org/doi/10.1098/rstb.2020.0390),
[Anikin 2022](https://www.nature.com/articles/s41598-022-23623-w)). Winter et al.'s
~14k-word iconicity ratings could even be baked as an extra per-token scalar if we want
sound-symbolic words to render more vividly.
([Winter 2024](https://link.springer.com/article/10.3758/s13428-023-02112-6))

### 5.3 Contract impact

A complexity scalar is a **new baked per-token value** (or a runtime function of token
length + a baked Zipf table) plus new synthesis logic that varies sub-gesture count.
This is a **`FORMAT_V2`** change (new mapping input + new synthesis behavior) and
**broadens NFR-16** (more than 4 axes now vary) — but complexity is _orthogonal to the
learnable semantic language_: it changes how _articulated_ a word is, not its
_meaning-timbre_, so the learnable property is preserved (§7.2). Variable sub-gesture
count also softens FR-15's "single continuous formant-glide warble" — a word becomes a
short micro-phrase, which is arguably what FR-15 already gestures at for multi-token
words.

---

## 6. Gap C — sentiment / emotional expression (the affect channel)

This gap carries the most design risk because it changes the performance layer, not
just the per-token sound.

### 6.1 Why emotion does not "come for free" from the existing semantics

A plausible shortcut would be to rely on model2vec embeddings, since sad words cluster
near other sad words. That does not give us reliable emotional prosody. PCA picks the
directions of maximum _semantic_ variance over the whole 30k vocab; there is no
guarantee any of the top-4 components aligns with valence or arousal. Even when one
loosely correlates, it maps to an arbitrary perceptual knob (pitch/vowel/contour/warble)
with no relationship to the _prosodic conventions_ of emotion (sad → low + flat + slow +
dark). The `learnability-spread.md` work confirms the axes carry _some_ structured
meaning, but "structured lexical meaning" ≠ "emotional prosody." Empirically, the
current output has no consistent sad-sounds-sad behavior because nothing connects text
sentiment to the §3.3 acoustic directions.

The production evidence says the same thing structurally: BB-8's emotion was a
**separate authored channel** (Schwartz's English performance), not a byproduct of which
words were said. dootdoot needs an explicit affect channel too.

### 6.2 The mechanism: a baked VAD affect vector, pooled to an utterance mood

1. **Per-token affect.** Bake a **valence/arousal** (optionally dominance) scalar per
   token from an offline sentiment lexicon, alongside the existing semantic vector.
   This is the same precompute-and-quantize pattern as the PCA table.
2. **Utterance mood.** Pool per-token affect (weight-scaled mean, like the semantic
   baseline) into a sentence-level (valence, arousal) — the "mood" of the phrase.
3. **Affect → prosody, by the §3.3 directions.** Drive the _global performance_
   from mood, not the per-token meaning-timbre:
   - **Arousal** (multi-parameter) → speech rate (syllable duration + pause
     length), pitch register bias, pitch _range_ (scales how far the semantic pitch knob
     swings), warble depth/rate, brightness (sparkle/upper-mid mix), and sub-gesture
     density. High arousal = faster, higher, wider, brighter, more inflected — matching
     the excited/contextual clips directionally. Low arousal = slower, sparser.
   - **Valence** (finer) → contour-direction bias (positive → rising/up-swoop tendency;
     negative → falling/declination + final lowering) and voice-quality/brightness
     (negative → darker, lower-pass; the sad clip is the clearest local example).
     Strong negative + low arousal → the "morose" target: low, flat, dark, slow.

This follows the measured emotion directions (§3.2), the literature (§3.3), and the
production intent (§3.1).

### 6.3 Lexicon choice — licensing is the deciding constraint

The richest academic VAD lexicons are **not redistributable**, which matters for a
shipped, committed-asset binary:

| Lexicon              | Dimensions                  | Coverage              | License                               | Shippable in a binary?                    |
| -------------------- | --------------------------- | --------------------- | ------------------------------------- | ----------------------------------------- |
| **NRC-VAD** v2.1     | Valence, Arousal, Dominance | 55k+                  | non-commercial, **no redistribution** | ❌ (commercial license, no raw redist)    |
| **Warriner 2013**    | V, A, D                     | 13,915                | ambiguous (Springer supp.)            | ⚠️ needs license review                   |
| **VADER**            | valence (−4..+4)            | ~7,500 (+emoji/slang) | **MIT**                               | ✅ cleanest first-pass choice             |
| **AFINN**            | valence (−5..+5)            | ~3,382                | **ODbL**                              | ⚠️ possible, but needs database policy    |
| **SentiWordNet 3.0** | pos/neg/obj                 | ~117k synsets         | CC BY-SA 4.0                          | ⚠️ possible, but share-alike + WSD burden |

Sources: [NRC-VAD](https://saifmohammad.com/WebPages/nrc-vad.html),
[VADER](https://github.com/cjhutto/vaderSentiment),
[AFINN](https://github.com/fnielsen/afinn),
[Warriner 2013](https://link.springer.com/article/10.3758/s13428-012-0314-x).

**Recommendation:** make the first affect pass **licensing-safe by construction**.
Bake **VADER (MIT)** for a clean valence axis and derive a coarse arousal proxy from
features dootdoot already owns: punctuation density (`!`, repeated markers), all-caps,
intensifier words from a small hand-curated MIT-compatible list, token count, and
character/WordPiece complexity. That gives enough signal to render the high-level
positive/negative contour and calm/agitated pacing split deterministically, with no
runtime dependency and no share-alike database question.

Treat **AFINN, SentiWordNet, SUBTLEX-US, and any VAD-frequency table** as a second-phase
asset decision after a license policy exists for committed derivative tables. **Arousal**
is the larger expressive win (it drives rate/range/brightness together), but the strongest
human-rated arousal source (NRC-VAD) cannot be redistributed raw. Options are (a) obtain
a commercial license and verify whether a _quantized derivative_ may ship, (b) use
Warriner if its license clears review, or (c) keep improving the deterministic arousal
proxy. The proxy is weaker than a human-rated VAD table, but it is shippable and still
separates "calm" from "agitated."

### 6.4 Contract impact

A new baked affect table + an affect→prosody driver is a **`FORMAT_V2`** change and the
biggest expansion of the input-dependent surface. It must be folded into the
`FORMAT_V1`→`V2` contract (new mapping input, new synthesis behavior) and surfaced in
`--explain` (an extra mood row supports the learnability goal). It
also **broadens NFR-16** — but, as with complexity, affect is a _separate orthogonal
channel_ (mood) layered over the learnable semantic gesture, so it deepens expression
without dissolving the learnable language (§7).

---

## 7. Gap D — additional sound textures and gesture types

### 7.1 The palette gap

Phase 7 added body/transient/sparkle layers _inside_ the one syllable archetype, but
every token still renders the **same** archetype. BB-8's vocabulary is a _family of
related gestures_: continuous warbling chatter, single high yelps (surprise), short
descending moans (sadness), rapid stutters (excitement), buzzy/rough trembles (anxiety;
the anxious clip measured low 0.57 harmonicity), and percussive rising stabs (alarm).
The acoustic analysis points to structure, not only knob values. Surprise is _one bare
sustained tone_, sad is _one dark blip_, and alarm is _repeated transient-onset stabs_.
They are **structurally different gesture archetypes**.

### 7.2 The mechanism: a small archetype palette selected by affect/structure

Introduce a **bounded set of deterministic gesture archetypes** (still all within the
droid parameter space), selected per token/phrase by the affect + complexity channels
rather than chosen freely:

- **Chatter** (default) — the current continuous warbling syllable.
- **Yelp** — short, high, sustained, single inflection; triggered by very high arousal +
  surprise context (e.g. a `!` after a short utterance).
- **Moan** — low, dark, falling glissando, slow; triggered by strong negative valence +
  low arousal.
- **Stutter/burst** — rapid sub-gesture repetition; high arousal + high complexity.
- **Tremble** — added jitter + slight inharmonicity (a deterministic rough texture);
  fear/anxiety (negative valence + high arousal).

Plus **non-vocal seasoning** used sparingly: short servo/mechanical blips or a brief
filtered-noise breath tail at phrase boundaries — these are what give the references
their "sound-effect" texture without making BB-8 read as non-vocal (R2's identity was
explicitly a _blend_ of human and machine).

This is the hardest gap to square with the project's core promise: a palette of
archetypes pushes against NFR-16's "one consistent droid family" and the v1 thesis that
_fixedness_ makes the language learnable. The mitigation is to keep the palette **small,
bounded, and deterministically selected by the affect/complexity channels** (not free
variation), so the archetype itself becomes _part of the learnable language_ ("BB-8
yelps when surprised") rather than noise. Determinism is preserved: archetype selection
is a pure function of the text.

### 7.3 Contract impact

New synthesis archetypes + a selection rule = **`FORMAT_V2`**, and the most significant
**reinterpretation of NFR-16** (the "bounded droid parameter space" now includes a
discrete archetype dimension). Recommend doing this _last_ and conservatively, after the
affect channel exists to drive selection — an archetype palette with nothing principled
selecting it would just be variety for its own sake.

---

## 8. Determinism and contract implications

None of the proposals threaten determinism: every new input (complexity scalar, affect
vector, phrase position, archetype choice) is a **pure deterministic function of the
text** plus frozen tables/constants, computed with the existing owned-math path. The
buffer-as-source-of-truth and bit-exact guarantees (§8.1 of `design.md`) are untouched.

What they _do_ require is honest contract accounting:

1. **This is a `FORMAT_V2`.** Every proposal alters output samples. Per the freeze rule,
   the v1 golden fixtures stay as the v1 contract; v2 gets its own version id, header,
   and regenerated golden hashes. The `FORMAT_V1` lock (T-54) already established the
   machinery for that.
2. **Requirement changes — two already resolved, one still open:**
   - **FR-20 (fixed syllable duration) — removed.** This was the hard one: pre-boundary
     lengthening, complexity-driven length, and arousal-driven rate all need duration to
     vary, and the §3 evidence is unambiguous that uniform duration is the primary
     staccato cause. The requirement has been **eliminated** from the spec, so variable
     duration is now permitted. Guidance for the implementation: keep duration a function
     of _structure and affect_, never a free per-token semantic axis — so rhythm stays
     _learnable_ (predictable from the text) even though it is no longer _uniform_.
   - **FR-22 (fixed inter-word pause) — revised.** Now allows deterministic,
     boundary-strength-dependent pauses (word vs clause vs sentence) instead of one fixed
     constant. Low risk; punctuation already varies pauses.
   - **NFR-16 ("only the 4 bounded axes vary") — still open.** Recommendation: **broaden**
     to "a fixed set of deterministic, bounded channels vary" — semantic (4 axes,
     learnable) + affect (valence/arousal, mood) + complexity (articulation) + archetype
     (gesture type). The droid-family identity is preserved by keeping every channel
     bounded and deterministic; it just stops being _only_ four axes. This is the
     remaining requirements decision to make before the affect/complexity/archetype work.
3. **New committed assets / licensing review.** Start with assets that are clean to
   redistribute (e.g. VADER under MIT plus hand-curated heuristics). Any ODbL,
   CC-BY-SA, NRC, Warriner, Zipf, or VAD-derived table needs explicit license policy
   before it is committed and `include_bytes!`-embedded (§6.3). The
   `source_manifest.toml` pattern extends naturally to pin approved sources.

### 8.1 The central tension: learnability vs expressiveness

The v1 thesis is that _fixedness_ makes the sound-language learnable and shareable.
Every gap here adds variability. The way to reconcile that is **orthogonal channels**:

- Keep the **4 semantic axes as the learnable core** (the "words"). Untouched.
- Add **affect, complexity, and archetype as separate deterministic channels** (the
  "performance") that shape _how_ the words are delivered, not _what_ they mean.

A listener still learns "this timbre = this meaning" from the semantic axes; the affect
channel layers a learnable "this delivery = this mood" on top. The production path for
BB-8 used the same split: a lexical/timing script (the words) performed with an emotional
overlay (Schwartz's English, Abrams' and Hader's live performance). Expressiveness and
learnability can coexist if expression is its own structured, deterministic layer.

The residual tradeoff noted in `voice-tuning.md` — "dootdoot remains cleaner and more
regular than the reference set" — is what these channels would address, at the cost of a
more complex but still deterministic contract.

---

## 9. Recommended direction and rough sequencing

Order the work by perceptual impact and dependencies. Each phase is independently
shippable as part of a v2 effort and testable red-green (value tests for the planners,
`insta` snapshots for `--explain`, golden-WAV hashes once frozen). This is a _suggested_
order, not a committed plan.

1. **Phrase prosody planner (Gap A)** — declination + final lowering + boundary reset +
   variable pauses + pre-boundary lengthening. Biggest perceptual win for the least new
   data; no new lexicon. Already unblocked at the requirements level (FR-20 removed, FR-22
   revised), so this can lead.
2. **Licensing-safe affect channel (Gap C)** — bake VADER valence, derive arousal from
   owned punctuation/case/intensifier/complexity signals, pool to mood, and drive
   rate/register/range/brightness/contour by the §3.3 directions. This adds emotional
   expression without blocking on NRC-VAD, ODbL, CC-BY-SA, or ambiguous
   supplemental-data licenses. Surface mood in `--explain`.
3. **Complexity → compound articulation (Gap B)** — start with WordPiece count +
   character length; add Zipf/frequency only after asset licensing is settled. This makes
   long words sound compound, is self-contained, and synergizes with arousal-driven
   density.
4. **Gesture archetype palette (Gap D)** — yelp/moan/stutter/tremble + sparing non-vocal
   seasoning, _selected by the affect/complexity channels_. Last, because it depends on
   the others to be principled rather than arbitrary, and is the biggest hit to the
   "consistent family" promise — so do it conservatively.

Cross-cutting: extend `--explain` to show the new channels (mood, complexity, archetype),
which directly serves the learnability goal and aids by-ear tuning; and reuse the Phase 7
`scripts/bb8-metrics` harness, adding the per-emotion corner metrics from §3.2 as
directional acceptance aids (with by-ear review still the gate).

---

## 10. Reliability caveats

- The seven contextual clips carry a media noise bed: event counts, active fractions, and
  some F0/span estimates are **noise- and tracker-limited**. Treat the per-emotion rows
  in §3.2 as directional by-ear/metric synthesis, not exact acoustic constants. The most
  stable local signals are relative density, obvious contour/archetype, dominant-peak
  motion, harmonicity, and broad spectral darkness/brightness.
- The §3.3 emotion→acoustics directions are well-corroborated across three independent
  reviews, but exact Murray & Arnott (1993) adjectives should be cited as "directional."
- Game-system numbers (§4.1) come from reference reimplementations / decompiled source,
  not vendor documentation.
- This document recommends a _direction_; the v1 thesis that fixedness aids learnability
  is real. FR-20 has been removed and FR-22 revised (§8), so variable pacing is settled;
  the remaining NFR-16 broadening is a tradeoff that needs an explicit decision before
  affect, complexity, or archetype channels become normative.

---

## Appendix A — source index

**Production / sound design:**
SlashFilm BB-8 voice · Post Magazine TFA sound editing · Time BB-8 voice · Hollywood
Reporter / Attack Magazine / Designing Sound (Ben Burtt, R2-D2 ARP 2600) · ACMI
(Simlish).

**Affective prosody:**
Banse & Scherer 1996 · Juslin & Laukka 2003 · Scherer 2004 · Murray & Arnott 1993
(directional).

**Synthesis prosody:**
Penn prosodic duration (Klatt) · Pierrehumbert 1979 (declination) · Parlikar & Black
2012 / Festival (pauses) · ToBI.

**Complexity / sentiment / sound symbolism:**
Zipf scale · SUBTLEX-US · Ćwiek 2021 / Anikin 2022 (bouba/kiki) · Winter 2024
(iconicity) · NRC-VAD · Warriner 2013 · VADER · AFINN · SentiWordNet · Farnell,
_Designing Sound_ (procedural LOD).

**Game voice systems:**
Animalese (Acedio / equalo) · Undertale OBJ_WRITER · Celeste (Regamey / FMOD) ·
Banjo-Kazooie · Ace Attorney (Capcom dev blog).

(Full URLs inline in §§3–7.)
