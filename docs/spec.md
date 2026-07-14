# dootdoot — Requirements Specification

> Derived from [`design.md`](./design.md). Each requirement is concise, testable, and
> uniquely identified. **FR** = functional, **NFR** = non-functional. IDs are stable;
> append new ones rather than renumbering.

---

## 1. Functional requirements

### 1.1 Input

- **FR-1** The tool SHALL accept input text as a positional command-line argument:
  `dootdoot "TEXT"`.
- **FR-2** When no positional text is given and stdin is piped (non-interactive), the
  tool SHALL read the input text from stdin.
- **FR-3** When no positional text is given and stdin is an interactive TTY, the tool
  SHALL print help and exit non-zero.
- **FR-4** The tool SHALL treat empty or whitespace-only input by emitting a fixed
  inquisitive "?" chirp gesture and exiting 0 (it SHALL NOT error or emit a silent
  file).

### 1.2 Tokenization

- **FR-5** The tool SHALL tokenize input using the `potion-base-8M` WordPiece
  tokenizer loaded from the tokenizer JSON embedded inside the committed `.doot` asset.
- **FR-6** The tool SHALL disable special tokens during tokenization
  (`add_special_tokens = false`); `[CLS]`/`[SEP]` SHALL NOT produce sound.
- **FR-7** The tool SHALL voice unknown (`[UNK]`) tokens using their own mapping
  (no special-casing, no skipping).
- **FR-8** The tool SHALL detect WordPiece subword-continuation marking (`##`) to
  identify word boundaries.

### 1.3 Semantic mapping

- **FR-9** The tool SHALL map each token to a 4-dimensional pre-squash vector by
  lookup in the baked table embedded inside the committed `.doot` asset, keyed by token ID.
- **FR-10** The baked table SHALL store, per token, a 4-dimensional int16 PCA vector
  and a scalar pooling weight.
- **FR-11** The tool SHALL compute a sequence baseline vector in PCA space as the
  token-weight-scaled mean of the per-token 4-dimensional vectors, with the **denominator
  being the token count `n`** (i.e. `(1/n) · Σ_i (w_i · v_i)`), and SHALL NOT apply an L2
  normalization. This is a dootdoot-specific, model2vec-derived pooling rule and is
  deliberately NOT byte-equivalent to `model2vec.encode()` for the L2-normalized
  `potion-base-8M` model.
- **FR-12** The tool SHALL squash each axis to a bounded perceptual range using
  frozen per-axis statistics, applied (a) per token for local gestures and (b) on the
  pooled sequence vector for the baseline.
- **FR-13** The 4 axes SHALL map to knobs in fixed order: PCA-1 → pitch center;
  PCA-2 → vowel/formant position; PCA-3 → contour/glide shape; PCA-4 → warble depth.
- **FR-14** Each token's gesture SHALL modulate around the sequence baseline (baseline
  = center, per-token vector = offset).

### 1.4 Synthesis (voice)

- **FR-15** The tool SHALL synthesize each token as a single continuous formant-glide
  syllable (not a cluster of discrete beeps). _`VOICE_V12` intentionally supersedes the
  one-token-one-syllable rule for **marked content words**, which gain a derived
  class-resolution syllable (FR-117); every syllable remains a continuous formant-glide
  gesture._
- **FR-16** The voice SHALL be a signal graph in which a control layer (pitch center +
  portamento glide + warble LFO) computes the instantaneous pitch that drives the audio
  path: harmonically-rich source/oscillator → formant filter bank (2–3 resonant
  bandpasses) → faint ring-mod → amplitude envelope.
- **FR-17** The following parameters SHALL be fixed (identical for all inputs):
  formant character/structure, portamento glide time, warble rate, ring-mod frequency
  and mix, envelope shape, high-register pitch bias, source waveform.
- **FR-18** The following parameters SHALL vary only with the 4 semantic axes: pitch
  center, vowel position, contour/glide shape, warble depth.
- **FR-19** Pitch SHALL glide (portamento) between consecutive syllables rather than
  stepping discretely.

### 1.5 Timing

- **FR-20** _(Removed.)_ Syllable duration is no longer required to be fixed. Variable,
  deterministic duration (e.g. phrase-final lengthening, complexity- or affect-driven
  pacing) is permitted. The ID is retired and not reused. Rationale and direction:
  [`bb8-expressiveness-gap-analysis.md`](./research/bb8-expressiveness-gap-analysis.md).
- **FR-21** Consecutive subword tokens within the same word SHALL be connected by
  portamento with no intervening silence.
- **FR-22** Distinct words SHALL be separated by a deterministic inter-word pause so the
  ear can segment words. The pause length MAY vary with phrase structure, boundary
  strength, or affect (e.g. word vs clause vs sentence boundaries) rather than being a
  single fixed constant, provided it remains a pure function of the input.
- **FR-23** Prosodic punctuation tokens (`.` `!` `?` `,` `;` `:`) SHALL be treated as
  control-only markers: they SHALL NOT be voiced as their own syllable and SHALL NOT
  count toward the voiced-syllable total. Each SHALL shape the **preceding** syllable
  and the following pause: `?` → rising final glide + longer pause; `.`/`!` → falling
  final glide + longer pause; `,`/`;`/`:` → medium pause, no contour change. Symbols
  outside this set SHALL be voiced as normal tokens. (`VOICE_V9` refines the per-mark
  boundary signature — clause continuation rise, period/exclamation split — per §1.18.)
- **FR-23a** In `--explain` output, prosodic-punctuation markers SHALL be shown as
  distinct control rows, separate from the per-token knob rows.
- **FR-24** The utterance SHALL include short fixed leading and trailing silence
  padding.

### 1.6 Output

- **FR-25** The engine SHALL produce a single canonical in-memory audio buffer
  (`Vec<i16>`, 44,100 Hz, mono) that is the sole source of truth for both file output
  and playback.
- **FR-26** When `-o/--output <FILE>` is given without `--play`, the tool SHALL write
  the buffer as a WAV file and SHALL NOT play audio.
- **FR-27** When no `-o/--output` is given, the tool SHALL play the buffer live and
  SHALL NOT write a file, **except** that `--explain` (a non-listening inspection mode)
  SHALL suppress the default playback. Audio SHALL still play when `--play` is given
  explicitly. In short, audio plays only when `--play` is set, or on a bare render with
  neither `-o` nor `--explain`.
- **FR-28** When both `-o/--output` and `--play` are given, the tool SHALL write the
  WAV file and play the buffer.
- **FR-29** Output WAV files SHALL be 44,100 Hz, 16-bit signed PCM, mono.
- **FR-30** Playback and file output SHALL derive from the identical buffer such that
  played audio equals saved audio sample-for-sample.

### 1.7 CLI & UX

- **FR-31** The tool SHALL provide `--explain`, printing a per-token table
  to stderr that is a complete account of the sound profile: an utterance-level `mood` and
  `complexity` summary; an aligned per-token grid
  (`token │ pitch │ vowel │ contour │ warble │ role │ archetype`); a per-token `curves`
  grid of the planner's bounded performance curves; and the glide/pause each control
  marker imposes.
- **FR-32** `--explain` output SHALL go to stderr only and SHALL NOT appear in any
  piped audio or file output.
- **FR-33** The tool SHALL provide `--version`, which SHALL surface the active voice
  identifier.
- **FR-34** The tool SHALL provide `--help`.
- **FR-35** The tool SHALL return exit code 0 on success and a non-zero code on error.

### 1.8 Input limits

- **FR-36** The tool SHALL print a warning to stderr when the input would render to more
  than ≈8 minutes of audio (≈2,000 tokens, ≈40 MB).
- **FR-37** The tool SHALL error (non-zero exit) without producing audio when the input
  would exceed the fixed output ceiling of **≈30 minutes / ≈160 MB** (≈8,000 tokens at
  44.1 kHz·16-bit·mono). The byte/duration ceiling is normative; the token count is a
  derived pre-synthesis check. (These are operational limits, not sample-affecting, so
  they are NOT part of `VOICE_V1`.)

### 1.9 Voice contract & versioning

- **FR-38** `VOICE_V1` SHALL bundle **every** parameter or rule that can affect an
  output sample, including: the model hash; the tokenizer configuration (the tokenizer
  JSON hash embedded in the `.doot` asset **and** the runtime tokenization flags —
  `add_special_tokens`, normalization/lowercasing, the `##` continuation convention);
  the control-token drop filter set (`[PAD]`/`[CLS]`/`[SEP]`/`[MASK]`, excluding
  `[UNK]`); the PCA projection matrix (by hash); the int16 quantization scales and rule
  for the 4 axes and the pooling weight (symmetric signed, zero-point-free,
  `s = max|·|/32767`, round-half-to-even, code −32768 unused); the sequence pooling rule
  (the weight-scaled mean with denominator = token count, and the deliberate omission of
  L2 normalization); the knob-assembly rule (per-axis modulation depths `α_k`, per-axis
  bounds `[lo_k, hi_k]`, and the final clamp); the per-axis squash statistics and squash
  function;
  all fixed synthesis constants; the timing constants (syllable duration, pauses,
  padding); the prosodic-punctuation rules; the empty-input "?" chirp constants; the
  float→i16 rounding rule; the WAV serialization choices (sample rate, bit depth,
  channels, header bytes); and the owned-math implementation version.
- **FR-39** Any change that alters one or more output samples SHALL require bumping the
  voice identifier (e.g. `VOICE_V1` → `VOICE_V2` → `VOICE_V3` → `VOICE_V4` →
  `VOICE_V5` → `VOICE_V6`).

### 1.10 Build-time generation (xtask)

- **FR-40** An `xtask` tool SHALL generate `assets/dootdoot_asset_v1.doot` from
  `potion-base-8M` by extracting all token embeddings, computing the top-4 PCA
  projection, canonicalizing component signs deterministically, computing squash
  statistics, and serializing the tokenizer JSON plus per-token vectors and weights into
  the dootdoot asset spec Protocol Buffers payload.
- **FR-41** PCA component signs SHALL be canonicalized by a deterministic rule (e.g.
  force each component's largest-magnitude loading positive) so generation is
  reproducible.
- **FR-42** `assets/dootdoot_asset_v1.doot` SHALL be the only committed runtime asset
  needed for tokenization and mapping, and it SHALL be embedded into the shipped binary.
  _`VOICE_V12` adds one further committed, embedded runtime asset — the sidecar
  noun/verb class table `assets/dootdoot_pos_v1.doot` (FR-114) — which serves word
  classification only; tokenization and mapping still need only the `.doot` asset._
- **FR-43** A committed `assets/source_manifest.toml` SHALL pin the immutable upstream
  source: HF repo, exact commit revision (SHA, not a branch/tag), expected SHA-256 of
  `model.safetensors` and `tokenizer.json`, and the structural expectations
  `hidden_dim = 256`, `normalize = true`, and dtype. `xtask` SHALL validate the acquired
  files against this manifest (revision, file hashes, structural fields) **before**
  computing or writing any asset, and SHALL abort on any mismatch, so that regeneration is
  reproducible. _`VOICE_V12` extends the same manifest with a `[pos]` section pinning
  the class-table pipeline (FR-114): ranking-corpus repo/revision/file/SHA-256, tagger
  name/version, and the committed tagged-counts snapshot hash._

### 1.11 VOICE_V2 expressiveness

- **FR-44** `VOICE_V2` MAY add deterministic, bounded performance channels for phrase
  timing, affect, complexity, and a small gesture-archetype palette, while keeping the
  four PCA-derived semantic axes as the learnable meaning core.
- **FR-45** Every `VOICE_V2` performance channel SHALL be a pure function of the
  token/control-event stream and SHALL NOT depend on runtime randomness, clocks, seeds,
  external services, or platform-dependent state.
- **FR-46** `VOICE_V2` explain output SHALL keep semantic token rows visible and MAY add
  stderr-only control/performance rows for mood, phrase, complexity, or archetype
  decisions where useful for learnability.
- **FR-47** `VOICE_V2` phrase prosody SHALL apply deterministic phrase metadata to
  synthesis by varying pause length, pre-boundary syllable lengthening, phrase-level pitch
  offsets, final lowering, pitch reset, and sparse emphasis within fixed bounds.
- **FR-48** `VOICE_V2` affect analysis SHALL pool VADER-derived token valence and owned
  arousal proxies into deterministic utterance-level valence and arousal scores.
- **FR-49** `VOICE_V2` synthesis SHALL map utterance arousal to deterministic duration
  rate, pitch lift, brightness, warble amount, and sub-gesture density, and SHALL map
  valence to contour and darker/brighter vowel/texture bias within fixed bounds.
- **FR-50** `VOICE_V2` complexity analysis SHALL compute a deterministic bounded scalar
  from owned inputs only: non-whitespace character count and continuation `WordPiece`
  subtoken count. Frequency, Zipf, iconicity, or other third-party psycholinguistic
  assets SHALL remain excluded until an explicit asset-license policy admits them.
- **FR-51** `VOICE_V2` synthesis SHALL map the complexity scalar to deterministic
  compound articulation by increasing bounded sub-gesture count, articulation density,
  and optional duration scaling without changing the semantic meaning-timbre axes.
- **FR-52** `VOICE_V2` gesture archetype selection SHALL use a fixed bounded palette
  (`chatter`, `yelp`, `moan`, `stutter/burst`, `tremble`) plus sparse non-vocal seasoning
  flags, and SHALL be a pure function of affect, complexity, punctuation, and phrase
  position.
- **FR-53** `VOICE_V2` synthesis SHALL render selected gesture archetypes with bounded,
  deterministic yelp, moan, stutter/burst, and tremble texture paths plus sparse servo
  and noise-tail seasoning, all inside finite BB-8-family parameter bounds.
- **FR-54** The frozen `VOICE_V2` contract SHALL be documented with final
  phrase/affect/complexity/archetype acceptance, contextual-clip directional checks,
  the surfaced `dootdoot VOICE_V2` version string, and regenerated golden WAV hashes.

### 1.12 VOICE_V3 phrase continuity

- **FR-55** `VOICE_V3` SHALL render connected token sequences with phrase-continuous
  oscillator/filter state across syllables, except after punctuation boundaries that
  intentionally reset the phrase.
- **FR-56** `VOICE_V3` word boundaries SHALL keep deterministic boundary duration but
  SHALL render quiet transition bridges instead of hard zero-filled inter-word gaps.
- **FR-57** `VOICE_V3` connected syllable edges SHALL use a deterministic nonzero
  envelope floor so token boundaries do not restart every syllable from silence.
- **FR-58** `VOICE_V3` SHALL keep the droid gesture envelope's internal dip, but the
  dip SHALL NOT clamp the envelope to silence inside the voiced body.
- **FR-59** The frozen `VOICE_V3` contract SHALL be documented with phrase-continuity
  acceptance, the surfaced `dootdoot VOICE_V3` version string, and regenerated golden
  WAV hashes.

### 1.13 VOICE_V4 repeated-onset smoothing

- **FR-60** `VOICE_V4` SHALL keep `VOICE_V3` connected phrase state while smoothing
  connected syllable openings in repeated subword sequences.
- **FR-61** `VOICE_V4` connected syllables SHALL NOT replay the explicit attack
  transient used for phrase starts.
- **FR-62** `VOICE_V4` connected syllable pitch and vowel openings SHALL blend from the
  previous rendered state into the new token gesture.
- **FR-63** `VOICE_V4` connected envelope starts SHALL ramp through the early body and
  SHALL NOT replay the full attack peak at each connected token boundary.
- **FR-64** The frozen `VOICE_V4` contract SHALL be documented with repeated-onset
  acceptance, the surfaced `dootdoot VOICE_V4` version string, and regenerated golden
  WAV hashes.

### 1.14 VOICE_V5 word-attack smoothing

- **FR-65** `VOICE_V5` SHALL keep `VOICE_V4` repeated-subword smoothing while
  distinguishing subword connections from word-boundary connections in the renderer.
- **FR-66** `VOICE_V5` word-boundary starts SHALL ramp from a lower bridge-matched
  envelope floor rather than reusing the high subword connection floor.
- **FR-67** `VOICE_V5` word-boundary vowel openings SHALL begin from a rounded
  `oo`-leaning pre-shape and open into the semantic vowel target over a bounded
  deterministic window.
- **FR-68** `VOICE_V5` word-boundary starts SHALL damp upper-mid sparkle and selected
  archetype texture during the opening bloom.
- **FR-69** `VOICE_V5` SHALL preserve the `VOICE_V4` repeated connected-subword
  roughness acceptance behavior.
- **FR-70** The frozen `VOICE_V5` contract SHALL be documented with word-attack
  acceptance, the surfaced `dootdoot VOICE_V5` version string, and regenerated golden
  WAV hashes.

### 1.15 VOICE_V6 repeated-phrase smoothing

- **FR-71** `VOICE_V6` SHALL keep `VOICE_V5` word-attack smoothing while reducing
  regular tremolo-like pulsing in repeated high-arousal word sequences.
- **FR-72** `VOICE_V6` word-boundary bridges SHALL remain audible connectors but SHALL
  stay below the surrounding syllable body instead of becoming foreground pulses.
- **FR-73** `VOICE_V6` word-boundary bridges SHALL use a flatter bounded envelope and
  reduced source, sparkle, and warble contribution.
- **FR-74** `VOICE_V6` connected word starts SHALL damp repeated internal pitch,
  complexity articulation, archetype pitch, and texture motion beyond the initial
  word-opening bloom.
- **FR-75** `VOICE_V6` connected word starts SHALL inherit pitch state over a longer
  bounded window than same-word subword starts.
- **FR-76** The frozen `VOICE_V6` contract SHALL be documented with repeated-phrase
  acceptance, the surfaced `dootdoot VOICE_V6` version string, and regenerated golden
  WAV hashes.

### 1.16 VOICE_V7 contextual performance & expanded synthesis range

> Derived from
> [`bb8-inquisitive-chatty-gap-analysis.md`](./research/bb8-inquisitive-chatty-gap-analysis.md).
> `VOICE_V7` targets contextual performance, expanded synthesis dynamic range, and mouth
> articulation. It keeps the four semantic PCA axes as the learnable core (FR-11) and adds
> deterministic, bounded performance channels on top. Every requirement below is
> sample-affecting and stays inside the fixed droid parameter space (NFR-16). Explicit
> non-goals: `VOICE_V7` SHALL NOT raise global brightness as a level (this render already
> exceeds the reference's median centroid), SHALL NOT introduce unseeded randomness, SHALL
> NOT change the semantic PCA mapping, SHALL NOT use a speech vocoder over English text,
> SHALL NOT center ring modulation as the main voice, and SHALL NOT import sample
> libraries.

- **FR-77** `VOICE_V7` SHALL provide a deterministic rising-chirp/whistle gesture that
  sweeps the **oscillator fundamental itself** (not only the upper-mid sparkle layer)
  toward the 2–4 kHz region, so the dominant tonal peak can climb into whistle range for
  selected gestures.
- **FR-78** `VOICE_V7` SHALL allow a wider per-gesture pitch span, bounded by named
  constants, so selected events can leave the prior ~0.5–1.1 kHz band while ordinary
  syllables remain in the established register.
- **FR-79** `VOICE_V7` SHALL provide a deterministic noise/breath excitation source
  blended under the tonal oscillator for selected gestures, so harmonicity can swing
  clean→rough within a gesture; the blend SHALL be authored (no runtime randomness),
  bounded, and finite, and ordinary syllables SHALL remain cleanly periodic.
- **FR-80** `VOICE_V7` SHALL support role-gated long pauses (≈600–1200 ms) for selected
  hesitation/turn arcs, gated so simple sentences do not become uniformly slower.
- **FR-81** `VOICE_V7` word-boundary bridging SHALL be suppressible so staged reply
  phrases can use short (≈30–80 ms) internal rests and the active-sound fraction can fall
  toward the reference's staged level; phrase-final lengthening and amplitude tails MAY
  occupy time without counting as an additional voiced syllable.
- **FR-82** `VOICE_V7` SHALL treat standalone `-`, `--`, en/em dash, and the
  single-character ellipsis (`…`) as control-only hesitation markers with a deterministic
  pause, instead of voiced semantic tokens; such markers SHALL NOT appear with four-axis
  values in `--explain`. (A multi-character ASCII `...` was _not_ normalised to an ellipsis
  by `VOICE_V7`; that is delivered by `VOICE_V9`, FR-96.)
- **FR-83** `VOICE_V7` MAY apply an optional, bounded code-talkbox mouth stage after the
  formant bank — a broad moving mouth filter (2–4 resonances) with a deterministic
  open/close envelope, tongue/front-back curves linked to the semantic/formant axes,
  optional breath/noise excitation, and mild bounded saturation — kept subtle and
  droid-like (not TTS), off by default until driven by the performance planner.
- **FR-84** `VOICE_V7` SHALL include a deterministic discourse-performance planner that
  runs after tokenization and before synthesis and assigns local phrase roles (`probe`,
  `chatty_reply`, `hesitation`, `terminal_flourish`, `aside`) as a pure function of the
  event stream, punctuation, word count, and control tokens.
- **FR-85** The `VOICE_V7` planner SHALL emit bounded, deterministic continuous
  performance curves (at least pitch center/velocity, formant target/velocity, brightness
  pressure, mouth openness, and archetype tension/release) that drive the synthesis
  primitives.
- **FR-86** `VOICE_V7` affect and archetype SHALL be localized per phrase/syllable (local
  arousal attack/hold/release and local valence) while retaining the utterance-level mood
  row; high positive arousal SHALL NOT select `Yelp` for the entire utterance — whistle
  and yelp SHALL be reserved for opener and terminal accents while the middle rotates
  chatter/stutter/tremble variants.
- **FR-87** `VOICE_V7` SHALL convert the always-on upper-mid sparkle into an event-based
  gesture resource (lower default mix, shaped attack/decay, reserved for
  chirps/flourishes/selected chatter) and MAY add sparse, phrase-aware seasoning families
  one at a time (self-oscillating/sine-resonator chirps, envelope-controlled ring-mod
  stress, deterministic breath/noise bands, bounded saturation blooms, tape-speed-style
  pitch/formant curves), keeping >6 kHz modest and no single family dominant.
- **FR-88** `VOICE_V7` MAY add a bounded, deterministic imperfection pass (gesture
  roughness, slight filter mismatch, or tape-speed-style pitch/formant curves) applied
  only after the dynamic-range, timing, mouth, and planner baselines are stable; it SHALL
  remain finite and inside the droid parameter bounds.
- **FR-89** The frozen `VOICE_V7` contract SHALL be documented with a contextual-clip
  acceptance note (tracked separately from the golden hashes), the surfaced
  `dootdoot VOICE_V7` version string, and regenerated golden WAV hashes; the planner's
  role and curve decisions SHALL be surfaced in `--explain` where useful.

### 1.17 VOICE_V8 semantic engagement & bursty texture

> Derived from
> [`bb8-corpus-timbre-texture-analysis.md`](./research/bb8-corpus-timbre-texture-analysis.md).
> `VOICE_V8` targets the residual corpus-wide gap: `VOICE_V7` already ships the expressive
> primitives (whistle, roughness, bursty sparkle, staged rests) but engages them only from
> punctuation/affect, so neutral text renders flat. `VOICE_V8` engages those primitives
> from semantics and makes the upper-mid layer bursty rather than constant. It keeps the
> four semantic PCA axes as the learnable core (FR-11) and reuses the V7 channels. Every
> requirement below is sample-affecting and stays inside the fixed droid parameter space
> (NFR-16). Explicit non-goals: `VOICE_V8` SHALL NOT raise the global brightness _level_
> (the corpus median centroid already matches), SHALL NOT introduce unseeded randomness,
> SHALL NOT change the semantic PCA mapping, SHALL NOT over-noise the body (the roughness
> floor stays bounded and subtle), and SHALL NOT fully de-bridge structured/punctuated
> phrases (only neutral input gains word rests).

- **FR-90** The `VOICE_V8` planner SHALL derive bounded per-syllable expressive engagement
  from the semantic PCA axes (salience) and word-to-word axis movement, so a neutral,
  unpunctuated utterance still receives per-syllable curve variation and at least one
  semantic accent per chatty-reply/probe segment. This SHALL NOT change the discourse
  roles that punctuation and position assign (FR-84); it only widens the per-syllable
  curves (FR-85).
- **FR-91** A `VOICE_V8` body-syllable semantic accent SHALL be able to engage the whistle
  sweep (FR-77) and wider pitch span (FR-78) without terminal punctuation, gated by a named
  archetype-tension threshold so only accents reach the whistle band.
- **FR-92** `VOICE_V8` SHALL lower the default upper-mid brightness of ordinary body
  syllables and make the event-based sparkle (FR-87) burstier (sharper envelope, lower
  floor, higher accent peak), so the constant 2–5 kHz share falls toward the BB-8 corpus
  while accent bursts rise; `>6 kHz` energy SHALL stay modest.
- **FR-93** Engaged (planner-driven) body syllables SHALL carry a small, named, always-on
  roughness floor so neutral text is not pinned to pure periodicity (harmonicity can swing
  off ~0.95 toward the corpus). Neutral-curve rendering (the empty chirp and hand-built
  events) SHALL keep a zero roughness floor and remain cleanly periodic.
- **FR-94** `VOICE_V8` SHALL insert short (≈30–80 ms) word-boundary rests on neutral
  multi-word input independent of punctuation, so the active-sound fraction falls toward
  the BB-8 library level; structured (punctuated/staged) utterances SHALL keep their
  longer staged rests and tonal bridges.
- **FR-95** The frozen `VOICE_V8` contract SHALL be documented with a corpus timbre/texture
  acceptance note (tracked separately from the golden hashes), the surfaced
  `dootdoot VOICE_V8` version string, and regenerated golden WAV hashes.

### 1.18 VOICE_V9 audible punctuation

> Five marks a writer reaches for — **question, exclamation, period, dash, ellipsis** —
> must each be audible as a distinct prosodic gesture, not collide (period ≡ exclamation),
> route to the wrong gesture (ASCII `...`), or differ only by a masked gap length (dash vs
> ellipsis). `VOICE_V9` reshapes each mark's boundary signature; it does not touch the PCA
> mapping or the discourse-role assignment. Derived from
> [`punctuation-prosody-audibility.md`](research/punctuation-prosody-audibility.md).

- **FR-96** `VOICE_V9` SHALL normalise a run of two or more ASCII periods (`...`, `....`)
  into a single control-only **ellipsis** hesitation marker, and SHALL collapse any other
  run of consecutive prosodic-punctuation tokens (including `?!`, `!!!`) to its first
  contour. This SHALL be deterministic and SHALL match the engine's existing first-wins
  behaviour on consecutive punctuation.
- **FR-97** `VOICE_V9` clause marks (`,` `;` `:`) SHALL carry a shallow **continuation
  rise** (a bounded upward final glide) and SHALL NOT impose a final lowering, so a clause
  boundary reads as open ("more coming") against a period's closed fall.
- **FR-98** `VOICE_V9` SHALL distinguish the period from the exclamation: a period SHALL
  fall deeper to a quiet settle, while an exclamation SHALL fall only shallowly from its
  raised, emphasized peak. A question SHALL keep its suppressed (non-lowered) rising close.
- **FR-99** `VOICE_V9` SHALL shape the trailing edge of the syllable preceding a hesitation
  marker by marker type: a dash SHALL clip the tail to silence abruptly, an ellipsis SHALL
  decay it gradually. The default (non-hesitation) tail SHALL be a transparent unity gain so
  all other syllables stay byte-identical. This contrast SHALL NOT depend on the marker's
  rest length, which the role-gated turn gap can mask.
- **FR-100** `VOICE_V9` SHALL give the question a dedicated terminal rise wider than the
  generic punctuation glide, with a bounded pre-final dip, and SHALL keep declination
  suppressed across the question's whole final segment.
- **FR-101** The frozen `VOICE_V9` contract SHALL be documented with an audible-punctuation
  acceptance note (tracked separately from the golden hashes), the surfaced
  `dootdoot VOICE_V9` version string, and regenerated golden WAV hashes (including `dash`
  and `ellipsis` corpus fixtures).

### 1.19 VOICE_V10 bidirectional whistle & gesture vocabulary

> A frame-by-frame, gesture-level comparison against the BB-8 corpus found dootdoot has the
> right gesture families but is **rising-biased and register-shy**: it never produced a
> falling whistle, its accents barely left the register and spanned ~1 octave where BB-8
> spans 3–4, neutral gestures ran long, and it never crossed into a rough/noisy burst.
> `VOICE_V10` widens that vocabulary without new instruments or any change to the PCA
> mapping, staying inside the fixed, deterministic, bounded droid parameter space (NFR-16).
> Derived from
> [`bb8-sound-vocabulary-taxonomy.md`](research/bb8-sound-vocabulary-taxonomy.md). Explicit
> non-goals: `VOICE_V10` SHALL NOT raise the global brightness _level_, SHALL NOT boost the
> upper-mid sparkle mix, SHALL NOT alter the warble, and SHALL NOT add unseeded randomness.

- **FR-102** `VOICE_V10` SHALL make the whistle sweep **signed**: a positive amount rises
  toward the whistle ceiling (the `VOICE_V7` climb), and a negative amount descends toward a
  named bounded floor (`WHISTLE_FLOOR_HZ`). A zero amount SHALL remain a no-op and the
  positive path SHALL stay byte-identical to `VOICE_V7`–`V9`.
- **FR-103** The exclamation terminal flourish SHALL descend (a falling whistle) while the
  question flourish SHALL keep rising; the direction SHALL be carried deterministically by
  the sign of the syllable's pitch velocity. Only exclamation-final flourishes SHALL move off
  `VOICE_V9`.
- **FR-104** A body semantic accent that engages the whistle SHALL sweep from a substantial
  engaged floor (not a near-zero ramp) and SHALL begin its sweep earlier in the syllable, so
  the swept pitch dwells in the whistle band; the engagement gate SHALL continue to isolate
  the one promoted accent from non-accent body syllables (no shrill every-syllable whistle).
  The whistle ceiling SHALL be unchanged.
- **FR-105** The single promoted semantic accent per phrase SHALL use a wider per-gesture
  pitch span (`ACCENT_PITCH_SPAN_SEMITONES`) than the terminal flourish, bounded inside the
  droid register, so a single accent gesture can approach BB-8's multi-octave excursions;
  non-accent gestures SHALL keep their existing spans.
- **FR-106** `VOICE_V10` SHALL pace neutral (text-path) syllables shorter than the base so
  neutral gestures read as quick blips rather than long held tones. The hand-built /
  empty-chirp / neutral-curve path (no explicit mood) SHALL keep a duration scale of exactly
  `1.0` and stay byte-identical. Output-length estimation SHALL remain consistent with render.
- **FR-107** A body semantic accent in an agitated utterance (high arousal **and** negative
  valence) SHALL be able to burst its noise/breath roughness toward the noisy band, then
  recover, so a single gesture leaves the tonal band. Non-accent, calm, and positive-valence
  syllables SHALL keep the steady-state roughness; the burst SHALL be deterministic and
  bounded.
- **FR-108** The frozen `VOICE_V10` contract SHALL be documented with a gesture-vocabulary
  acceptance note (tracked separately from the golden hashes, re-running the
  `sound_taxonomy.py` harness), the surfaced `dootdoot VOICE_V10` version string, and
  regenerated golden WAV hashes.

### 1.20 VOICE_V11 natural voice: softer onset, breathing pace, integrated breath

> By-ear feedback found the voice **percussive and metronomic** (syllable onsets clicked; an
> unpunctuated phrase had no tempo variation), a dash made the **whole preceding clause** a
> wall of breath noise, and the breath itself sounded **artifacty** (a separate hiss layered
> over the voice). `VOICE_V11` softens the onset, lets per-syllable pacing breathe, localizes
> the dash's breath, and integrates the aspiration noise into the voice — all inside the fixed,
> deterministic, bounded droid parameter space (NFR-16). Explicit non-goals: `VOICE_V11` SHALL
> NOT change the semantic PCA mapping, SHALL NOT alter the pitch/formant/warble constants,
> SHALL NOT change punctuation pause lengths, and SHALL NOT add unseeded randomness.

- **FR-109** `VOICE_V11` SHALL soften the syllable onset so it blooms rather than clicks: the
  envelope attack ramp SHALL be lengthened, and the per-word onset transient SHALL be both
  quieter and spread over a longer window. The onset SHALL remain a shaped, deterministic
  attack (no change to the decay/sustain/release shape).
- **FR-110** `VOICE_V11` SHALL vary per-syllable duration across a phrase so pacing breathes
  without punctuation: a deterministic positional lilt, agogic lengthening on emphasized
  syllables, and phrase-final lengthening on the last syllable. The variation SHALL be a
  closed-form function of syllable index, count, and the existing emphasis flag, bounded
  inside the droid range. A single-syllable phrase SHALL have no internal rubato (scale
  exactly `1.0`). The hand-built / empty-chirp / neutral-curve path (no explicit mood) SHALL
  keep a rubato scale of exactly `1.0` and stay byte-identical; output-length estimation SHALL
  remain consistent with render.
- **FR-111** `VOICE_V11` SHALL localize a dash/ellipsis hesitation to the syllable that carries
  it: the clause that precedes the marker SHALL read as a plain statement that trails off (not
  a breathy hesitation across the whole clause, and not an inquisitive probe). A single-word
  pre-marker filler SHALL still read as a hesitation.
- **FR-112** `VOICE_V11` SHALL make aspiration breath read as integrated voice rather than a
  separate hiss: the breath noise SHALL be amplitude-modulated pitch-synchronously (peaking at
  the glottal closure instant), sourced from a near-white spectrum (no fixed-stride comb
  coloration) shaped by the formant filter, and mixed additively over the tone. The breath
  SHALL stay deterministic and bounded, and `roughness_amount == 0` SHALL remain exactly clean.
- **FR-113** The frozen `VOICE_V11` contract SHALL be documented with a by-ear acceptance note,
  the surfaced `dootdoot VOICE_V11` version string, and regenerated golden WAV fixtures.

### 1.21 VOICE_V12 noun/verb recognizability

> Recurring content words were hard to recognize by ear because their identity rode on
> absolute pitch in a continuous, arbitrary mapping
> ([research](./research/noun-verb-recognizability.md) §1). `VOICE_V12` gives nouns and
> verbs a systematic two-pillar signature — a layered co-onset class marker and a
> compound `stem → class-resolution` silhouette — validated by the T-118 spike and
> by-ear evaluation ([worksheet](./research/voice-v12-spike-evaluation.md)). Explicit
> non-goals: `VOICE_V12` SHALL NOT change the semantic PCA mapping, SHALL NOT introduce
> a runtime tensor framework, and SHALL NOT add unseeded randomness. Verb
> reduplication/aspect, noun size iconicity, and the learnability regression are
> `VOICE_V13` follow-ons.

- **FR-114** The noun/verb class data SHALL come from a **pinned build-time POS
  source**: `xtask` SHALL derive a per-lemma dominant class table from a
  permissively-licensed POS source ranked by a pinned coding-domain corpus snapshot
  (not general English), with both pinned by hash in `assets/source_manifest.toml`, and
  SHALL bake the result as a **committed sidecar class-table asset** (its own spec
  version + source hashes; the semantic `.doot` asset stays at spec v1). The shipped
  binary SHALL contain no tagger and no tensor runtime — the class lookup is a pure
  baked table.
- **FR-115** The POS class SHALL be a **word-level** property: the word-initial token
  establishes the class for its whole word (keyed by the assembled word/lemma) and
  continuation tokens inherit it. Closed-class/function words SHALL classify `Other`.
  Noun/verb-ambiguous lemmas SHALL follow the **conservative policy** locked by the
  T-118 A/B: they fall back to `Other` (unmarked) rather than being marked with a
  dominant class.
- **FR-116** Word-initial content (noun/verb) syllables SHALL carry a **layered
  co-onset class marker** mixed into the syllable's first milliseconds, starting
  together with the tonal body (zero added duration) and rising with a short attack
  ramp so it fuses into the word's attack rather than reading as a separate pre-beat:
  noun = broadband click/pop splash (~30 ms window, ~8 ms ramp), verb = up-swept
  dual-sine chirp (~50 ms window, ~25% attack fraction). Markers SHALL be deterministic
  owned-math gestures, louder than the `VOICE_V11` softened word transient and layered
  over it (not replacing it); continuation tokens, function words, and the
  neutral/hand-built path SHALL never fire a marker.
- **FR-117** Marked content words SHALL render as a **compound
  `stem → class-resolution` silhouette**: the resolution syllable derives from the
  stem's own knobs via a frozen per-class transform (noun **settle**: pitch steps down,
  vowel rounds toward `oo`, contour flattens, steadier tail; verb **push**: brighter
  toward `ee`, rising/gliding continuation) — never random padding. A word's syllable
  target SHALL be `max(subword_count, 2)` capped at 3, with multi-token words shaping
  their **last** subword as the resolution. This supersedes FR-15's
  one-token-one-syllable rule for marked content words only.
- **FR-118** The sequence semantic baseline SHALL remain pooled over the **original
  tokenizer tokens**; derived resolution syllables SHALL NOT join the pool or alter
  the mood/complexity analyses.
- **FR-119** Compound words SHALL shorten their per-syllable base duration (scale
  ~0.62) so a two-syllable word reads as one heavier gesture rather than two blips,
  preserving the `VOICE_V11` breathing pace; function words SHALL stay single light
  blips. Output-length estimation SHALL remain exactly consistent with rendering, and
  unscaled syllables (scale exactly `1.0`) SHALL stay byte-identical.
- **FR-120** `--explain` SHALL show each token's POS class, onset marker, and syllable
  silhouette as a learnability training aid.
- **FR-121** The frozen `VOICE_V12` contract SHALL be documented with a by-ear
  acceptance note (`docs/validation/voice-v12-noun-verb.md`) including directional
  `scripts/acoustics` + `scripts/sound_taxonomy.py` checks on the minimal pairs, the
  surfaced `dootdoot VOICE_V12` version string, and regenerated golden WAV fixtures.
  Until that freeze, the class-conditioned behavior SHALL stay behind the local
  default-off spike gate with the no-class path byte-identical.

### 1.22 Browser renderer

- **FR-122** The documentation site SHALL accept arbitrary text and render it locally by
  compiling `dootdoot-core` to WebAssembly. The browser binding SHALL use the normal
  `VOICE_V12` canonical-buffer and WAV-serialization path, and SHALL NOT implement a
  JavaScript approximation or a second synthesis engine. Browser playback SHALL consume
  only the WAV bytes returned by the core.

---

## 2. Non-functional requirements

### 2.1 Determinism

- **NFR-1** For a fixed voice version, identical input text SHALL produce
  byte-identical audio output across repeated runs on the same machine.
- **NFR-2** For a fixed voice version, identical input text SHALL produce
  byte-identical audio output across the **CI-verified platforms (macOS and Linux)**.
  The math is designed to be bit-exact on other platforms (incl. Windows), but such
  platforms are NOT guaranteed until added to the golden-hash CI matrix (NFR-17).
- **NFR-3** All transcendental math in the audio path (`sin`, `exp`, `tanh`, and any
  others) SHALL use owned, pinned implementations; libm transcendentals SHALL NOT be
  used in the audio path.
- **NFR-4** Synthesis SHALL be computed in `f64` and converted to `i16` by a single
  fixed rounding rule with no dithering.
- **NFR-5** The audio path SHALL NOT use fast-math or FMA contraction that could
  change results across platforms.

### 2.2 Architecture & footprint

- **NFR-6** The shipped binary SHALL NOT depend on `model2vec-rs`, `candle`, or any
  tensor framework at runtime.
- **NFR-7** The committed `.doot` runtime asset SHALL be on the order of ~1 MB, carrying
  the tokenizer JSON plus ~30k compact per-token records of 4×int16 + int16 weight.
- **NFR-8** A **normal build** (compiling from the committed assets) and **runtime**
  SHALL require no network access. **Asset regeneration** via `xtask` is exempt and MAY
  require a one-time model download, depending on the acquisition decision (FR-40/T-11).
- **NFR-9** The codebase SHALL be a Cargo workspace of three crates: `dootdoot-core`
  (pure engine library), `dootdoot` (CLI binary), and `xtask` (build-time generator,
  not shipped).
- **NFR-10** `dootdoot-core` SHALL contain no filesystem or audio-device I/O and SHALL
  be unit-testable in isolation. WAV serialization in core SHALL target an in-memory
  byte buffer or a generic `impl std::io::Write`; the `dootdoot` binary SHALL own the
  actual file write and the playback device.

### 2.3 Platform & performance

- **NFR-11** The tool SHALL build and run on macOS (Apple Silicon) as the primary
  target.
- **NFR-12** Live playback SHALL use `rodio`/`cpal` (CoreAudio on macOS).
- **NFR-13** For typical short inputs, end-to-end latency from invocation to first
  sound SHALL be perceptibly immediate (sub-second), aided by the no-tensor-runtime
  design.

### 2.4 Quality goals (verifiable)

- **NFR-14** Semantically similar tokens SHALL be closer in 4-axis space than
  dissimilar ones (e.g. distance `cat↔dog` < `cat↔airplane`), verified by test.
- **NFR-15** Semantically similar short sequences SHALL be closer in baseline-axis
  space than dissimilar ones, verified by test. The metric is dootdoot's own sequence
  baseline (FR-11), not `model2vec.encode()`; the property asserted is _relative_
  similarity ordering, which the model2vec-derived PCA-space pool preserves.
- **NFR-16** Every output, regardless of input, SHALL remain within the fixed droid
  parameter space, preserving a consistent BB-8-family identity. `VOICE_V1` achieves
  this by varying only the four bounded semantic axes; `VOICE_V2` MAY additionally vary
  deterministic, bounded phrase, affect, complexity, and archetype channels; `VOICE_V3`
  MAY additionally smooth connected phrase rendering without changing the semantic
  mapping core; `VOICE_V4` MAY additionally smooth repeated connected onsets; `VOICE_V5`
  MAY additionally smooth bridged word-boundary attacks; `VOICE_V6` MAY additionally
  smooth repeated-phrase bridge and motion pulsing; `VOICE_V7` MAY additionally vary
  deterministic, bounded contextual-performance channels (expanded synthesis dynamic
  range — whistle-range tonal sweep, wider per-gesture pitch span, noise/breath
  excitation; role-gated long pauses with suppressible word-boundary bridging;
  dash/ellipsis hesitation markers; an optional code-talkbox mouth stage; a
  discourse-performance planner emitting local phrase roles and continuous performance
  curves; and localized per-phrase/per-syllable affect and archetype), each a pure
  function of the text and clamped to the fixed droid parameter space.

### 2.5 Testing

- **NFR-17** A golden-WAV test suite SHALL assert SHA-256 hashes of outputs for a fixed
  input corpus, serving as the cross-platform determinism contract, and SHALL run in CI
  on macOS and Linux.
- **NFR-18** A determinism test SHALL run each corpus input twice and assert
  byte-identical buffers.
- **NFR-19** Owned-math functions SHALL be tested for correctness within tolerance
  against references and SHALL have pinned exact outputs at sample points.
- **NFR-20** The `--explain` table SHALL have a golden snapshot test.

### 2.6 Documentation

- **NFR-21** Documented behaviors SHALL include: uncased tokenization
  (`Hello`==`hello`), English-oriented handling of non-Latin/emoji input, the empty-
  input "?" chirp, and the input warning/cap thresholds.
- **NFR-23** The project documentation SHALL build as a static VitePress site from the
  authoritative Markdown files, generate collection navigation from the docs tree, provide
  local full-text search, and expose an arbitrary-text browser renderer backed by the compiled
  Rust core. Adding or renaming a supporting document SHALL NOT require a manual sidebar edit.
- **NFR-24** The generated browser module SHALL render the covered native golden fixture
  byte-for-byte and SHALL return byte-identical WAV bytes for repeated arbitrary text. The
  WebAssembly package SHALL be generated during the site build rather than maintained as
  hand-edited JavaScript or a committed binary.
- **NFR-25** The landing page and documentation reader SHALL share the KotoR-inspired
  aural-protocol visual system: self-hosted pinned display/mono typography, near-black teal
  surfaces, cyan telemetry, restrained amber controls, and industrial panel geometry. The
  experience SHALL remain responsive, keyboard-accessible, semantically labelled, readable
  without decorative effects, and respectful of `prefers-reduced-motion`.

### 2.7 Distribution

- **NFR-22** The release process SHALL provide no-Rust-toolchain macOS install and update
  paths via Homebrew and prebuilt release artifacts for Apple Silicon and Intel Macs.
  The release configuration SHALL ship only the `dootdoot` binary crate, never `xtask`,
  SHALL route macOS binary builds to the dedicated `dootdoot-macos-arm64` self-hosted
  Apple Silicon runner, and SHALL be guarded by a committed release-packaging smoke check.
