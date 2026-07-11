# Making nouns and verbs recognizable by ear — research + plan

> Status: **proposal**, pre-implementation. This document diagnoses why dootdoot's
> lexicon is hard to learn, grounds the fix in auditory-cognition research, and
> proposes a phased build (a `VOICE_V12`/`VOICE_V13` arc) that gives nouns and verbs
> distinct, learnable acoustic signatures. It touches the runtime/build split (§9 of
> [`design.md`](../design.md)) and would add the first **grammatical** channel to the
> voice — deliberately drifting from strict BB-8 mimicry toward learnability, per the
> request that prompted it.

**Validation result.** The goal is sound and worth pursuing: the proposal directly
addresses the current language's weakest learnability point by moving recurring
word identity away from a mostly continuous one-syllable pitch/vowel gesture and
toward discrete, systematic, redundant cues. The design remains compatible with
the no-runtime-tensor invariant, bit-exact determinism, and the `VOICE_V*`
contract, provided the POS source is pinned and the added class table is treated as
a committed voice asset.

The main correction is data-modeling: the shipped behavior must classify
**lexical words**, not merely independent WordPiece tokens. A per-token baked
table is still a useful storage primitive, but the renderer needs one stable
word-level class that word-initial tokens carry and continuation tokens inherit, so
multi-subword words receive one coherent onset mark and one final resolution
shape. Shipping the compound silhouette also requires revising the `FR-15` /
[`design.md` §6.4](../design.md) "one token = one syllable" contract: `VOICE_V12`
would intentionally add derived resolution syllables for marked content words
while keeping the semantic baseline pooled over the original tokenizer tokens.

---

## 1. The problem, and why it happens

**Symptom.** A listener cannot understand an utterance without also reading the text.
The hardest part is recognizing recurring **nouns and verbs**, _especially the most
common ones_.

**Root-cause diagnosis (from the literature, §2).** dootdoot is semantically grounded,
but for a listener trying to identify recurring words by ear it still behaves too much
like an **abstract earcon** system. Three compounding causes:

1. **Identity rides mostly on absolute pitch, which is the _worst_ dimension for
   recognition.** In classic one-dimensional absolute-judgment tasks, humans identify
   only ~5–6 pitch levels (~2.5 bits; Pollack, via Miller 1956), even though they can
   _discriminate_ thousands. dootdoot word recognition is not literally the same lab
   task, but a one-token, ~170 ms, continuously-pitched syllable pushes the listener
   toward that failure mode: "which word was that?" depends too much on remembering an
   absolute pitch region. A mapping whose primary lever is `pitch_center` (PCA-1, the
   most salient knob; [`design.md` §5.2](../design.md)) can only resolve a handful of
   categories by ear before neighboring words blur.
2. **The parameter space is continuous, so words blur.** Non-speech categorical
   perception is weak and must be _engineered_; a smooth embedding→sound projection
   places "words" at sub-JND spacing where neighbors collapse into one percept
   (Gygi MDS; categorical-perception literature).
3. **The mapping is arbitrary, not systematic or iconic.** Arbitrary abstract sounds
   (earcons) are the slowest-learned, most-confused class of auditory signal in every
   head-to-head; auditory icons and _systematic_ mappings are learned far faster
   (Dingler et al. 2008; Sonification Handbook ch. 14; iconicity reviews). dootdoot's
   embedding→gesture map has no compositional structure a listener can latch onto.

The most common words suffer most because they are **single WordPiece tokens** → **one
~170 ms syllable** ([`design.md` §6.4](../design.md)), so they have the least internal
structure to be distinctive, and they recur constantly, so their confusability
dominates the listening experience.

**The fix is not "more pitch range."** It is (a) move identity onto the dimensions
humans actually identify — **timbre, attack, tonal↔noise, and pitch _contour shape_**;
(b) make the code **discrete and multidimensional** (many axes, few levels each); and
(c) make it **systematic** — give a whole grammatical class one shared, salient
signature. That last point is exactly what "mark nouns and verbs" means, and it is the
single highest-leverage move available.

---

## 2. Evidence base (condensed)

Two independent literature reviews converged on the same principles. The load-bearing
ones for this plan, each with its design implication:

| #   | Finding                                                                                                                                                                                                     | Source(s)                                                                  | Implication for dootdoot                                                                  |
| --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| P1  | One acoustic dimension caps at ~4–7 absolutely-identifiable categories (~2.5 bits for pitch).                                                                                                               | Miller 1956; Pollack                                                       | Stop leaning on absolute pitch for word identity.                                         |
| P2  | Redundant **multidimensional** coding is the biggest capacity lever: ~150 categories from 6 crudely-2-level dimensions. **Many dimensions, few levels each.**                                               | Pollack & Ficks 1954                                                       | Quantize several perceptual axes into 2–4 zones; don't finely subdivide pitch.            |
| P3  | **Timbre** (spectral centroid + attack sharpness + tonal↔noise) is the dimension of _source/object identity_ — the best carrier for "which word is this".                                                   | McAdams; Caclin 2005                                                       | Give word classes distinct **timbre families** and **attack shapes**.                     |
| P4  | **Contour** (rise/fall/flat/scoop) is more robust than exact absolute pitch when listeners recognize transformed melodies.                                                                                  | Dowling & Fujitani 1971                                                    | Encode identity in contour _shape_, a small categorical alphabet.                         |
| P5  | A fixed, salient **class tag / affix** can be processed as grammatical class information; paired with a synchronous onset cue, it is a plausible segmentation anchor.                                       | Zeller, Bylund & Lewis 2022 (Zulu noun-class agreement, EEG); Bregman/Gygi | Add a distinct **co-onset marker** per class — the core of this plan.                     |
| P6  | **Cross gesture categories** within a word (transient + tone + texture) should separate better than two variants inside one acoustic family.                                                                | Gygi; VanDerveer; Gaver                                                    | Noun/verb contrast must _cross_ families (e.g. click vs chirp), not be two similar tones. |
| P7  | Listeners can segment streams by statistical regularities; invariant multi-part word shapes with reliable boundary cues should be easier to learn than one-off blips.                                       | Saffran et al. 1996; word-length / redintegration literature               | Give common (single-token) content words deterministic **multi-part internal structure**. |
| P8  | **Iconicity/systematicity** can speed word learning; crossmodal sound symbolism supports round/sharp and related perceptual metaphors, while motion/action mappings remain a design hypothesis to validate. | Ohala; Ćwiek 2022; ideophone reviews                                       | Make noun vs verb signatures _iconic_: nouns settle (objects), verbs move (actions).      |
| P9  | An `--explain`-style training/feedback mode is likely to help recognition, but this needs project-local validation rather than being treated as already proven for dootdoot.                                | perceptual-learning literature; project validation                         | Extend `--explain` to teach the class + marker.                                           |

### Primary sources

The load-bearing anchors, each mapped to the finding(s) it supports:

- **Miller, G. A. (1956).** "The magical number seven, plus or minus two." _Psychological
  Review_ 63(2), 81–97. — P1 (single-dimension identification limit; Pollack's pitch data
  are reported here).
- **Pollack, I. & Ficks, L. (1954).** "Information of elementary multidimensional auditory
  displays." _J. Acoust. Soc. Am._ 26(2), 155–158. — P2 (multidimensional coding capacity).
- **McAdams, S. et al. (1995)** timbre-space work; **Caclin, A. et al. (2005),** "Acoustic
  correlates of timbre space dimensions," _J. Acoust. Soc. Am._ 118(1), 471–482. — P3
  (timbre = source identity: centroid, attack, tonal↔noise).
- **Dowling, W. J. & Fujitani, D. S. (1971).** "Contour, interval, and pitch recognition in
  memory for melodies." _J. Acoust. Soc. Am._ 49(2B), 524–531. — P4 (contour, not absolute
  pitch, is what's stored).
- **Zeller, J., Bylund, E. & Lewis, A. G. (2022).** "The parser consults the lexicon in
  spite of transparent gender marking: EEG evidence from noun class agreement processing in
  Zulu." _Cognition_ 226, 105148.
  <https://doi.org/10.1016/j.cognition.2022.105148> — P5 (Bantu noun-class prefixes are salient,
  reliably-processed morphological class markers). _Note: this study establishes that the
  prefix is a strong lexical/agreement cue the parser attends to; the "anchors word
  segmentation" extension is our inference from the general onset-boundary literature
  (Gaver/Gygi), not a direct finding of this paper._
- **Gygi, B.** environmental-sound work and **VanDerveer (1979)** — P6 (cross-category
  gestures are a strong design heuristic for separability; within-family sounds are
  easier to confuse).
- **Saffran, J. R., Aslin, R. N. & Newport, E. L. (1996).** "Statistical learning by
  8-month-old infants." _Science_ 274(5294), 1926–1928. — P7 (stream segmentation via
  statistical learning; the 2–3-part template is our design extrapolation from
  statistical segmentation plus word-length / redintegration work, not a direct
  requirement from this paper).
- **Ćwiek, A. et al. (2022).** "The bouba/kiki effect is robust across cultures and writing
  systems." _Phil. Trans. R. Soc. B_ 377(1841), 20200390.
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC8591387/> — P8 (sound↔shape iconicity;
  round/sharp crossmodal correspondences). See also Ohala's frequency code and
  ideophone/whistled-language reviews for the broader, more speculative size/motion
  mappings.
- **Dingler, T. et al. (2008)** and the **Sonification Handbook, ch. 14** — earcon vs.
  auditory-icon learnability (§1, cause 3): arbitrary abstract earcons (the class Brewster's
  HCI earcon work formalized) are the slowest-learned, most-confused auditory signals.

---

## 3. Design strategy — three levers

A content word's identity is built from **two co-equal pillars**: a **layered onset mark**
(Lever A) that tags the class in the first instant, and a **compound multi-syllabic
silhouette** (Levers B+C, merged below) that gives the whole word a class-consistent
rhythmic/melodic shape a listener can learn and re-derive from a partial hearing. Function
words get neither — they stay light single blips, so the marked content words pop out. The
onset mark is the _tag_; the compound silhouette is the _word_.

### Lever A — discrete class **onset markers**, **layered co-onset** (the noun-class prefix)

Add a short, fixed, **cross-category** foley transient to the first syllable of every
**word-initial** noun and verb token. The marker is categorically different between the
two classes _and_ different from the tonal body — so it does double duty (P5 + P6):
it tags grammatical class **and** anchors the word boundary. It is scarce (only content
nouns/verbs get one; function words stay bare), which preserves its distinctiveness.

This is the part that directly answers "use specific sound effects for nouns and specific
others for verbs." dootdoot today has breath, sparkle, and noise-excitation layers, but
still has **no true click, pop, or broadband burst** ([taxonomy §Mapping](bb8-sound-vocabulary-taxonomy.md))
as a lexical marker — exactly the palette needed. The local BB-8 foley corpus
(`~/repos/anddav87/bb8-sounds/`, mechanical clicks/servo ticks/chirps) can be used as
analysis/reference material; do not ship sampled foley unless its license is explicitly
cleared. The default implementation should synthesize deterministic click/chirp markers
with owned math.

**The marker is _layered_ (co-onset), not pre-rolled.** It is mixed into the syllable's
first ~15–60 ms — starting _together_ with the tonal body — rather than played as a
separate event before it. Three reasons this is the stronger design:

- **Common-onset binding (Bregman auditory-object formation).** Sounds that begin
  together fuse into one auditory object, so the marker becomes part of the word's
  **attack timbre** — and attack + spectral shape is the _single best-identified_
  dimension for "which sound is this" (P3, McAdams/Caclin). A pre-rolled tag risks
  reading as a _separate_ source announced before the word; a layered one _is_ the word's
  identity.
- **Zero added duration.** A pre-roll lengthens every content word and fights the
  `VOICE_V11` breathing-pace work; a layered transient reshapes the onset _within_ the
  existing syllable duration, so sample counts are unchanged.
- **It reuses a proven primitive.** dootdoot's existing `attack_transient_sample`
  (`synth.rs:815`) is _already_ a layered co-onset transient — a dual-sine chirp with a
  30 ms quadratic decay mixed in at 4% at the onset. Class markers **generalize that one
  function into class-conditioned variants**, rather than introducing new sequencing
  machinery.

Concretely, three co-onset cases (all mixed into the onset window; values are
`VOICE_V12` tuning targets):

| Class                | Layered onset flavor                                                         | Temporal envelope (crosses categories, P3/P6)                  |
| -------------------- | ---------------------------------------------------------------------------- | -------------------------------------------------------------- |
| **Noun**             | broadband **click/pop** splash (dense partials or a short filtered impulse)  | very fast, ~15–25 ms, near-instant attack — _impact = a thing_ |
| **Verb**             | **rising chirp** — the dual-sine, frequency-_swept upward_ across the window | slightly longer, ~40–60 ms, gliding — _motion = an action_     |
| **Other / function** | the current softened breathy transient (or none)                             | unchanged                                                      |

Optionally each mark is itself **two layered components** (cross-category _within_ the
mark, P6): noun = click + low thud (impact + mass); verb = chirp + brief air-whoosh
(motion + breath).

**Gain tension to resolve by ear.** `VOICE_V11` deliberately _quieted_ the transient
(mix 0.07 → 0.04) so it would stop reading as a percussive pluck on every word. The class
markers must be **louder than that** to be a reliable cue — but they now fire _only_ on
scarce word-initial content tokens, not every word, so the "pluck on everything" problem
does not recur. The marker gain is its own `VOICE_V12` tuning slice, layered _over_ the
existing softened onset so the breathy articulation is preserved and the class flavor
rides on top.

### Levers B + C — the **compound multi-syllabic class silhouette**

Levers B (class-consistent body) and C (compounding) are one mechanism: give every content
word a deterministic **2–3-syllable silhouette** whose shape reads as its class. The class
iconicity (B) is expressed _as_ the compound structure (C) — a distinct resolution
syllable — rather than a vague continuous bias, which makes it more categorical and more
learnable (P2, P4, P7, P8).

**The template: `stem → class-resolution`.**

- Syllable 1 (**stem**) carries the word's semantics (its `KnobSet`) and the layered
  onset mark from Lever A.
- Syllable 2 (**resolution**) is a **frozen per-class transform of the stem's own knobs**:
  - **Noun → _settle_** (object at rest): vowel rounds toward `oo`, pitch steps down,
    contour flattens, sustained tail; size-iconic (a semantically "big" noun settles
    lower/duller, a "small/sharp" one higher/brighter — the frequency code, P8).
  - **Verb → _push_** (action carries forward): brighter, a rising/gliding continuation
    (the wide swoop dootdoot already wants more of —
    [taxonomy](bb8-sound-vocabulary-taxonomy.md)); **reduplicated** (stem→stem) with a
    tremble for repeated/ongoing aspect.

Because the resolution is a fixed function of the stem, **every noun shares a "settling"
melodic shape and every verb a "pushing" one** — a class-level signature the ear can learn
— while individual words still differ by their stem knobs (systematic, not arbitrary; the
fix for the earcon problem, §1).

**Deterministic syllable source (the crux).** The extra syllable is _never_ random padding
(that would break "similar words sound similar"). It is derived from the semantic knobs the
word already has, plus its class. So:

- **Single-token common words** (the ones that collapse into one blip today, and the user's
  main pain) expand from 1 syllable to 2 via the stem→resolution template — this is where
  the biggest legibility gain lands.
- **Multi-token (rare) words** keep their natural WordPiece subword syllables (which already
  glide together) but shape the **last** subword as the resolution, so the class silhouette
  is consistent at any length. Syllable target = `max(subword_count, class_minimum)`,
  **capped at 3** (P7's rehearsal/word-length ceiling).

**Emergent bonus — heavy content vs. light function.** Content words become rhythmically
"heavy" (2–3 syllables + onset mark); function words stay light single blips. That
content/function weight contrast is itself a strong, universal segmentation cue and further
separates the words that carry meaning from the connective tissue.

**Pacing guardrail.** More syllables = longer words; unchecked this bloats the utterance and
fights `VOICE_V11`'s pace. Mitigations: cap at 2–3 syllables; shorten the per-syllable base
duration when a word compounds (a 2-syllable word should not be 2× a single blip); keep
function words single and quick; and lean on the existing `syllable_rubato_scale`
(`sequence.rs`) for the intra-word lilt.

---

## 4. Concrete acoustic recipes

A first, tunable proposal (all values become `VOICE_V12`/`V13` constants; final values
land by-ear per the project's tuning convention). The two classes are engineered to
**cross categories** on every axis so they cannot blur (P6):

| Channel                                         | **Noun** signature                                                                                                     | **Verb** signature                                                                                    | Function/other                    |
| ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | --------------------------------- |
| **Onset marker** (layered co-onset, §3 Lever A) | **Click/pop** — broadband transient, near-instant attack, ~15–25 ms, mechanical (BB-8 servo tick). _Impact = a thing._ | **Rising chirp** — up-swept pitched transient, ~40–60 ms, into the bright band. _Motion = an action._ | current softened transient / none |
| **Word structure** (Levers B+C)                 | **2 syllables: stem → settle**                                                                                         | **2 syll: stem → push** (or **stem → stem** reduplicated for aspect)                                  | **1 light blip**                  |
| **Body contour (P4)**                           | resolution syllable settles / flattens                                                                                 | resolution syllable rises / swoops                                                                    | unchanged                         |
| **Vowel/timbre (P3)**                           | rounder (`oo`/`ah`), warmer                                                                                            | brighter (`ee`-leaning), more open                                                                    | unchanged                         |
| **Tonal↔noise (P3)**                            | clean tonal                                                                                                            | clean tonal, optional tremble on aspect                                                               | unchanged                         |
| **Tail**                                        | sustained; low-hum size cue for "big" nouns                                                                            | clipped or gliding away                                                                               | unchanged                         |
| **Iconicity (P8)**                              | pitch/brightness ∝ smallness (frequency code)                                                                          | rising contour = "up/approach"; faster = faster action; reduplication = repeated                      | —                                 |

Design guardrails from the evidence:

- **Quantize, don't smooth (P1, P2).** The marker is one of exactly two discrete
  transients; body contour picks from a _small_ alphabet (settle / rise / rise–fall /
  scoop). More axes, few levels each — not a finer pitch continuum.
- **Keep the shared "voice" (P6, Bregman streaming).** Markers and bodies stay inside the
  droid timbre so a phrase is still _one droid talking_, while adjacent words carry enough
  onset/contour contrast to segment.
- **Mark scarcely.** Only word-initial noun/verb tokens. Over-marking destroys the
  distinctiveness that makes a marker a marker.

---

## 5. The enabling decision — where does "noun vs verb" come from?

dootdoot has **no grammatical information anywhere today** (confirmed: zero
`noun`/`verb`/`pos` references in `dootdoot-core`, `dootdoot`, or `xtask`). The runtime
also **forbids a tensor framework** — the whole architecture precomputes and bakes
(model2vec is build-time-only; [`design.md` §4.2](../design.md)). So POS must respect that
invariant. Options:

| Option                                       | How                                                                                                                                                                    | Determinism                                                          | Blast radius                                                                                                              | Recommendation                                                                                                     |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| **A. Build-time baked word/POS class table** | `xtask` derives a dominant class `{Noun, Verb, Other}` for whole-word tokens and word-initial stems; continuation subtokens inherit the current word class at runtime. | ✅ pure lookup, tensor-free — mirrors the semantic bake exactly.     | Sidecar embedded table (small) **or** `.doot` spec-version bump (`DOOT_ASSET_SPEC_VERSION` 1→2, widen/extend the record). | **Recommended.** On-brand; per-lemma consistency is a _feature_ for learnability (a word always sounds its class). |
| B. Runtime rule/lexicon tagger               | Small embedded high-frequency lexicon + suffix rules, evaluated per token.                                                                                             | ✅ deterministic if fixed.                                           | No asset change.                                                                                                          | **Good for Phase 0 spike** — cheap, validates the acoustics before committing to the data pipeline.                |
| C. Runtime context-aware tagger              | A Brill/perceptron tagger picks POS in context.                                                                                                                        | ⚠️ heavier; context-sensitive so a word's sound changes by sentence. | New dependency; risks non-determinism.                                                                                    | **Rejected** — hurts learnability (inconsistent per-word sound) and fights the no-runtime-model invariant.         |

**Recommended path: B to bootstrap, then A to ship.** Use a tiny hard-coded lexicon of
the ~50–100 most common nouns/verbs to prototype and tune the _acoustics_ first (the
user's pain is precisely the common words), then graduate to a full build-time baked
table for coverage. **Per-lemma dominant POS, not context-aware** — consistency is what
makes it learnable. Store and carry the result as a **word class** in the engine even if
the backing asset is keyed by token ID: a word-initial token determines the class, and
continuation tokens inherit it so the word receives one onset marker and one terminal
resolution shape.

Validation caveat: the shipped static class table needs an explicit **closed-class /
function-word override** and an **ambiguity policy**. High-frequency English words such
as `can`, `will`, `light`, `play`, and `run` are exactly where dominant-POS tagging can
be wrong in a way listeners will hear constantly. A conservative rule is better for
learnability: closed-class words stay `Other`; genuinely ambiguous high-frequency lemmas
fall back to `Other` unless the source distribution is decisively noun- or verb-heavy.

Open sub-question for A: pick a permissively-licensed POS source for build time (e.g. a
frequency-tagged POS wordlist, WordNet lexnames, or a one-time tagged-corpus pass in
`xtask`), pinned in `source_manifest.toml` the same way the model is, so regeneration
stays reproducible. If this lands as a sidecar table rather than a `.doot` record bump,
the sidecar is still a committed runtime asset and part of the voice contract.

### 5.1 Empirical check — coverage on a coding-domain corpus (2026-07-03)

The primary intended use is **narrating coding work** (commit messages, build/test
status, agent output), so the table-sizing intuitions from the conversational
literature (≈2,000 word families ≈ 95% coverage; verb usage extremely top-heavy —
Adolphs & Schmitt 2003; Biber et al. 1999) were checked empirically against that
register: ~552k alphabetic tokens of real commit-message text (subjects + bodies across
all local repos, trailers/URLs stripped), POS-tagged with spaCy `en_core_web_sm`
(tagger + lemmatizer), with candidate tables modeled as the top-N words of the
`wordfreq` general-English ranking. Three findings **overturn the general-English
assumptions**, one confirms the risk already flagged above:

1. **Coding text is noun-heavy, not verb-heavy.** 40.7% `NOUN`, 13.8% lexical `VERB`,
   10.1% `PROPN` — the opposite of conversation (~12–15% nouns). Commit messages talk
   about _things_ (APIs, workflows, deployments). Noun coverage matters at least as
   much as verb coverage; do not weight the lexicon toward verbs for this register.
2. **A general-English frequency table transfers badly.** Coverage of the corpus's
   noun/verb _tokens_ by a top-N general-English word table, versus an oracle table
   built from the coding corpus's own top lemmas:

   | Table                             | Noun cov | Verb cov | Combined |
   | --------------------------------- | -------- | -------- | -------- |
   | top-2,000 general English         | 40%      | 54%      | **~44%** |
   | top-5,000 general English         | 61%      | 73%      | 64%      |
   | top-1,000-each from coding corpus | 89%      | 97%      | —        |
   | top-2,000-each from coding corpus | **95%**  | **99%**  | —        |

   The misses are the daily vocabulary: _deployment, sync, api, workflow, bump, doc,
   error, path, log_ (nouns); _update, remove, verify, skip, generate, fail, deploy,
   merge, render_ (verbs). A 50+50 spike lexicon picked by **general** frequency covers
   only ~5% of coding noun tokens / ~15% of verb tokens; picked from **coding-domain**
   frequency it covers ~35% / ~49%. Consequences: the Phase 0 spike lexicon and the
   baked table's entry ranking must be **domain-weighted** (e.g. a pinned snapshot of a
   commit-message/dev-text corpus), with the ranking source hash-pinned in
   `source_manifest.toml` so regeneration stays reproducible. Domain top-2,000-each
   reaches the ~95% coverage the conversational literature promised from a general list.

3. **Noun/verb ambiguity is the dominant cost, not coverage.** 17.4% of open-class
   tokens would be mis-marked under single-dominant-POS, and the ambiguous lemmas are
   the _core_ of the register — English zero-derivation ("the build" / "to build")
   concentrates exactly in dev vocabulary: _sync_ (46% minority-class use), _build_
   (48%), _fix_ (43%), _update_ (43%), _run_ (41%), _delete_, _deploy_, _share_, _log_,
   _filter_, _gate_, _check_. The conservative fall-back-to-`Other` policy above would
   therefore **unmark much of the highest-frequency coding vocabulary**, so the marking
   rate actually heard would sit far below the raw coverage numbers. This turns the
   ambiguity policy from a footnote into a first-order design decision: "always marked
   as its dominant class" (consistent, sometimes grammatically wrong) may beat "usually
   unmarked" for learnability, and the Phase 0 evaluation should A/B exactly that by
   ear (see §9.6).

Caveats: single-user corpus; spaCy `sm` mis-tags sentence-initial imperatives ("**Fix**
release workflow" → `NOUN`), which inflates both the noun share and the ambiguity
figures somewhat (commit subjects are imperative-heavy) — a better tagger or an
imperative-aware heuristic would soften but not eliminate finding 3; occasional
lemmatizer artifacts (_datum_ for _data_) don't move the aggregates.

---

## 6. Where it plugs into the code

From the pipeline map (all under `dootdoot-core/src` unless noted):

- **Per-token metadata today** flows on `TokenizedToken { id, text, continuation }`
  (`tokenizer.rs:29`) → `VoicedToken` (`engine.rs:41`, built at `engine.rs:240`) → `SyllableEvent`
  (`sequence.rs:234`). A word-level POS class should ride here as a new field, set
  around `engine.rs:330` where knobs are assembled. Do not model it as a property of
  every syllable independently: word-initial tokens establish the class and continuation
  tokens inherit it.
- **The planner is the injection point** (`performance.rs:202 plan_discourse_performance`).
  It already reads per-syllable `KnobSet`; add the POS class to `Segment`
  (`performance.rs:92`) / `PerformanceSyllable` (`performance.rs:79`) and let it steer
  role/curve choice and gate the onset marker.
- **The onset marker is a layered co-onset primitive** — a class-conditioned
  generalization of `attack_transient_sample` (`synth.rs:815`), mixed in at the same
  point the current transient is added (`synth.rs:1508`, inside
  `render_performance_sample`). It is **not** pre-rolled by the sequencer, so no new
  sequencing path and no added duration (Lever A rationale). The class flag rides on
  `SyllablePerformance` (`synth.rs:295`) so `SyllableRenderControls` can select the noun
  (broadband click) vs verb (up-swept chirp) vs neutral variant; add the marker gain as a
  new `VOICE_V12` constant near `ATTACK_TRANSIENT_MIX` (`synth.rs:249`). Body
  contour/timbre biasing reuses the existing `PerformanceCurves` channels
  (`performance.rs:67`).
- **Compound syllables affect more than synthesis.** Expanding one token into a
  stem→resolution unit changes event counts, duration estimation, planner indexing,
  `--explain`, and golden corpus expectations. Keep the semantic baseline pooled over
  original tokenizer tokens, not over derived resolution syllables, so the added
  learnability layer does not distort the model2vec-derived sequence mood. This also
  requires source-of-truth updates before implementation: `spec.md` needs new `VOICE_V12`
  FRs for POS source, class markers, derived resolution syllables, explain rows, and
  acceptance; `design.md` §6.4 must explicitly supersede the current "one token = one
  syllable" statement for marked content words.
- **Determinism rules apply unchanged:** owned math only (`mathx`), `f64`, no libm, one
  rounding rule. Onset markers are fixed deterministic gestures — no randomness.
- **Contract:** `ACTIVE_VOICE` in `asset.rs:50`; any audio change regenerates the golden
  WAV fixtures (`dootdoot-core/tests/fixtures/golden/`, verified byte-exact by
  `tests/golden_wav.rs:22`) and needs a `docs/validation/voice-v12-*.md` acceptance note
  (asserted by `tests/voice_tuning_acceptance.rs`). Keep the neutral/empty-chirp path
  byte-identical by gating the new behavior off when no POS class is present, so only the
  text-path goldens move.

---

## 7. Phased plan (proposed tasks)

Slots after the current head (T-114); a two-version arc. Estimates are rough.

**Phase 0 — spike & validate the acoustics (no freeze).**

- **T-115** Hard-code a ~50–100-word high-frequency noun/verb lexicon (Option B) as a
  runtime lookup behind a local spike gate. Thread a `PosClass` through the planner.
  Do not ship a feature-selectable alternate voice without its own explicit voice
  contract. _~2h_
- **T-116** Prototype **both pillars** behind the gate: the two **layered co-onset** marker
  variants (generalizing `attack_transient_sample` — click/pop, up-swept chirp) _and_ the
  **compound `stem → class-resolution` silhouette** (2-syllable noun-settle / verb-push).
  Render A/B minimal pairs (e.g. _dog_ vs _run_, _the cat sits_ vs _cats sit_) and confirm
  by ear that (a) the mark fuses into the word's attack rather than reading as a separate
  pre-beat, and (b) the compound silhouette makes common single-token words distinct
  without bloating the pace. _~5h_
- **T-117** Evaluate: by-ear, plus `scripts/acoustics` and `scripts/sound_taxonomy.py`
  to confirm the marker lands as a distinct gesture and the two classes separate on
  contour/timbre/attack (not just pitch). Decide go/no-go and lock the recipe. _~2h_

**Phase 1 — POS data pipeline (Option A).**

- **T-118** Choose + pin a build-time POS source in `source_manifest.toml`; `xtask`
  derives dominant `{Noun, Verb, Other}` classes for whole-word tokens / word-initial
  stems, with continuation inheritance and a closed-class/ambiguity policy. _~4h_
- **T-119** Bake the class table (sidecar embedded table **or** `.doot`
  `DOOT_ASSET_SPEC_VERSION` 1→2 + wider record); update asset spec tests
  (`tests/asset_spec_layout.rs`, `tests/embedded_asset.rs`) and contract tests for the
  sidecar if that route is chosen. _~4h_

**Phase 2 — `VOICE_V12`: the complete content-word unit (layered mark + compound
silhouette).** Both pillars ship together so nouns/verbs land as coherent words, not a
tag bolted onto an unchanged blip.

- **T-120** Implement the **layered co-onset** noun click/pop and verb up-swept chirp as
  class-conditioned variants of `attack_transient_sample` (deterministic, owned math),
  mixed at `synth.rs:1508`, gated to word-initial content tokens; add the marker-gain
  constant and tune it by ear against the `VOICE_V11` softened-transient level. _~3h_
- **T-121** Implement the **compound `stem → class-resolution` silhouette** (Levers B+C):
  expand single-token content words to 2 syllables via the frozen per-class knob transform
  (noun settle / verb push); shape the last subword of multi-token words as the resolution;
  wire the syllable-count rule (`max(subword_count, class_min)`, cap 3), the shortened
  compound base duration, rubato integration, duration estimation, planner index mapping,
  and explain rows in `engine.rs`/`sequence.rs`. This is the structural core — the biggest
  slice. _~6–8h_
- **T-122** Extend `--explain` to show each token's POS class, marker, and syllable
  silhouette (training aid, P9). _~2h_
- **T-123** Freeze `VOICE_V12`: bump `ACTIVE_VOICE`, regenerate goldens, write
  `voice-v12-noun-verb.md`. _~2h_

**Phase 3 — `VOICE_V13`: aspect, size iconicity, learnability eval, tuning.**

- **T-124** Verb **reduplication**/tremble for repeated/ongoing aspect; noun **size-iconic**
  settle depth keyed to semantic magnitude (P8). _~3h_
- **T-125** A learnability regression: a minimal-pair discrimination metric (in the spirit
  of `tests/learnability_spread.rs`) asserting noun/verb pairs separate on the identity
  dimensions (onset category, syllable silhouette, resolution contour). _~3h_
- **T-126** Freeze `VOICE_V13` + acceptance doc. _~2h_

> **Phasing tradeoff.** Folding the compound silhouette into `VOICE_V12` (rather than
> deferring it to V13) makes V12 a larger, harder-to-bisect version, but ships nouns/verbs
> as _whole words_ instead of tagged blips — the user's stated priority. The Phase 0 spike
> (T-115–T-117) de-risks it by validating both pillars by ear before the data pipeline and
> freeze.

---

## 8. Risks & non-goals

- **This drifts from BB-8** (clicks/pops/chirp-tags are not canonical droid grammar).
  Accepted by request; the shared droid timbre and bounded parameter space (NFR-16) keep
  it recognizably the same droid.
- **Every golden fixture moves** across V12 and V13 — expected; that's what the version
  bump and regeneration are for.
- **POS mislabeling** (a word tagged the wrong class) will _consistently_ mis-sound that
  word. Per-lemma static tagging makes errors predictable and fixable via the pinned
  source; context-aware tagging (Option C) would make them less learnable because the same
  word could change sound by sentence. The highest-risk cases are common ambiguous lemmas,
  so the closed-class/ambiguity policy in §5 is mandatory.
- **External foley licensing.** The BB-8 corpus can guide the acoustic target, but shipped
  assets must be synthetic or explicitly licensed. Otherwise the proposal would conflict
  with the repo's committed-artifact/release posture even if the audio design is sound.
- **Non-goal:** intelligible speech or a full grammatical system. This marks two classes
  (plus "other"); adjectives/adverbs/tense are out of scope for this arc.

---

## 9. Open decisions for you

1. **POS source & storage** — sidecar embedded table (smaller blast radius, leaves the
   semantic `.doot` contract untouched) vs `.doot` spec-version bump (single canonical
   asset). Recommendation: sidecar for V12, fold into `.doot` later if desired. Per the
   §5.1 empirical check, whatever source is chosen, the table's **entry ranking must be
   coding-domain-weighted** (general-English top-2,000 covers only ~44% of coding
   noun/verb tokens vs ~95% for a domain-built table), with the ranking corpus snapshot
   pinned in `source_manifest.toml`.
2. **Marker aggressiveness** — content nouns/verbs only (recommended), or also mark a
   third class? Marking everything defeats the purpose.
3. **How far to lean into foley** — subtle servo-tick/chirp vs. bolder pop/whistle. Bolder
   is more learnable (P1/P6) but less BB-8. Recommendation: start bold in the Phase 0
   spike, dial back by ear.
4. **Compound syllable count & pacing** — default 2 syllables (stem → resolution) with a
   cap of 3; should any words go to 3 (e.g. high-salience nouns get a held third syllable),
   and how much to shorten the compound base duration to protect the pace? Recommendation:
   ship uniform 2 in V12, tune the duration by ear, revisit a semantic-length rule in V13.
5. **Aspect marking (Phase 3)** — verb reduplication + noun size iconicity as a V13
   refinement, or fold into V12? Recommendation: ship the V12 unit first, judge by ear
   before committing to V13.
6. **Ambiguity policy under coding-domain data** — the §5 conservative rule (ambiguous
   lemmas → `Other`) versus always marking the dominant class. The §5.1 empirical check
   shows the conservative rule would unmark much of the highest-frequency coding
   vocabulary (_build, fix, run, update, sync, deploy, log, check_ are all >25%
   minority-class use), so this is a first-order learnability tradeoff: consistently
   marked-but-sometimes-grammatically-wrong vs consistently unmarked. Recommendation:
   A/B both by ear during the Phase 0 evaluation before locking the policy into the
   contract.
