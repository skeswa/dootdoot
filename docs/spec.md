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
  tokenizer loaded from an embedded `tokenizer.json`.
- **FR-6** The tool SHALL disable special tokens during tokenization
  (`add_special_tokens = false`); `[CLS]`/`[SEP]` SHALL NOT produce sound.
- **FR-7** The tool SHALL voice unknown (`[UNK]`) tokens using their own mapping
  (no special-casing, no skipping).
- **FR-8** The tool SHALL detect WordPiece subword-continuation marking (`##`) to
  identify word boundaries.

### 1.3 Semantic mapping

- **FR-9** The tool SHALL map each token to a 4-dimensional pre-squash vector by
  lookup in an embedded baked table keyed by token ID.
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
  syllable (not a cluster of discrete beeps).
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
  outside this set SHALL be voiced as normal tokens.
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
  SHALL NOT write a file.
- **FR-28** When both `-o/--output` and `--play` are given, the tool SHALL write the
  WAV file and play the buffer.
- **FR-29** Output WAV files SHALL be 44,100 Hz, 16-bit signed PCM, mono.
- **FR-30** Playback and file output SHALL derive from the identical buffer such that
  played audio equals saved audio sample-for-sample.

### 1.7 CLI & UX

- **FR-31** The tool SHALL provide `--explain`, printing a per-token table
  (`token │ pitch │ vowel │ contour │ warble`) to stderr.
- **FR-32** `--explain` output SHALL go to stderr only and SHALL NOT appear in any
  piped audio or file output.
- **FR-33** The tool SHALL provide `--version`, which SHALL surface the active format
  identifier (`FORMAT_V1`).
- **FR-34** The tool SHALL provide `--help`.
- **FR-35** The tool SHALL return exit code 0 on success and a non-zero code on error.

### 1.8 Input limits

- **FR-36** The tool SHALL print a warning to stderr when the input would render to more
  than ≈8 minutes of audio (≈2,000 tokens, ≈40 MB).
- **FR-37** The tool SHALL error (non-zero exit) without producing audio when the input
  would exceed the fixed output ceiling of **≈30 minutes / ≈160 MB** (≈8,000 tokens at
  44.1 kHz·16-bit·mono). The byte/duration ceiling is normative; the token count is a
  derived pre-synthesis check. (These are operational limits, not sample-affecting, so
  they are NOT part of `FORMAT_V1`.)

### 1.9 Format contract & versioning

- **FR-38** `FORMAT_V1` SHALL bundle **every** parameter or rule that can affect an
  output sample, including: the model hash; the tokenizer configuration (the
  `tokenizer.json` hash **and** the runtime tokenization flags — `add_special_tokens`,
  normalization/lowercasing, the `##` continuation convention); the control-token drop
  filter set (`[PAD]`/`[CLS]`/`[SEP]`/`[MASK]`, excluding `[UNK]`); the PCA projection
  matrix (by hash); the int16 quantization scales and rule for the 4 axes and the pooling
  weight (symmetric signed, zero-point-free, `s = max|·|/32767`, round-half-to-even,
  code −32768 unused); the sequence pooling rule (the weight-scaled mean with denominator =
  token count, and the deliberate omission of L2 normalization); the knob-assembly rule
  (per-axis modulation depths `α_k`, per-axis bounds `[lo_k, hi_k]`, and the final clamp);
  the per-axis squash statistics and squash function;
  all fixed synthesis constants; the timing constants (syllable duration, pauses,
  padding); the prosodic-punctuation rules; the empty-input "?" chirp constants; the
  float→i16 rounding rule; the WAV serialization choices (sample rate, bit depth,
  channels, header bytes); and the owned-math implementation version.
- **FR-39** Any change that alters one or more output samples SHALL require bumping the
  format identifier (e.g. `FORMAT_V1` → `FORMAT_V2`).

### 1.10 Build-time generation (xtask)

- **FR-40** An `xtask` tool SHALL generate `assets/format_v1.bin` from `potion-base-8M`
  by extracting all token embeddings, computing the top-4 PCA projection, canonicalizing
  component signs deterministically, computing squash statistics, and serializing the
  per-token vectors and weights.
- **FR-41** PCA component signs SHALL be canonicalized by a deterministic rule (e.g.
  force each component's largest-magnitude loading positive) so generation is
  reproducible.
- **FR-42** `assets/format_v1.bin` and `assets/tokenizer.json` SHALL be committed to
  the repository and embedded into the shipped binary.
- **FR-43** A committed `assets/source_manifest.toml` SHALL pin the immutable upstream
  source: HF repo, exact commit revision (SHA, not a branch/tag), expected SHA-256 of
  `model.safetensors` and `tokenizer.json`, and the structural expectations
  `hidden_dim = 256`, `normalize = true`, and dtype. `xtask` SHALL validate the acquired
  files against this manifest (revision, file hashes, structural fields) **before**
  computing or writing any asset, and SHALL abort on any mismatch, so that regeneration is
  reproducible.

### 1.11 FORMAT_V2 expressiveness

- **FR-44** `FORMAT_V2` MAY add deterministic, bounded performance channels for phrase
  timing, affect, complexity, and a small gesture-archetype palette, while keeping the
  four PCA-derived semantic axes as the learnable meaning core.
- **FR-45** Every `FORMAT_V2` performance channel SHALL be a pure function of the
  token/control-event stream and SHALL NOT depend on runtime randomness, clocks, seeds,
  external services, or platform-dependent state.
- **FR-46** `FORMAT_V2` explain output SHALL keep semantic token rows visible and MAY add
  stderr-only control/performance rows for mood, phrase, complexity, or archetype
  decisions where useful for learnability.
- **FR-47** `FORMAT_V2` phrase prosody SHALL apply deterministic phrase metadata to
  synthesis by varying pause length, pre-boundary syllable lengthening, phrase-level pitch
  offsets, final lowering, pitch reset, and sparse emphasis within fixed bounds.
- **FR-48** `FORMAT_V2` affect analysis SHALL pool VADER-derived token valence and owned
  arousal proxies into deterministic utterance-level valence and arousal scores.
- **FR-49** `FORMAT_V2` synthesis SHALL map utterance arousal to deterministic duration
  rate, pitch lift, brightness, warble amount, and sub-gesture density, and SHALL map
  valence to contour and darker/brighter vowel/texture bias within fixed bounds.
- **FR-50** `FORMAT_V2` complexity analysis SHALL compute a deterministic bounded scalar
  from owned inputs only: non-whitespace character count and continuation `WordPiece`
  subtoken count. Frequency, Zipf, iconicity, or other third-party psycholinguistic
  assets SHALL remain excluded until an explicit asset-license policy admits them.
- **FR-51** `FORMAT_V2` synthesis SHALL map the complexity scalar to deterministic
  compound articulation by increasing bounded sub-gesture count, articulation density,
  and optional duration scaling without changing the semantic meaning-timbre axes.
- **FR-52** `FORMAT_V2` gesture archetype selection SHALL use a fixed bounded palette
  (`chatter`, `yelp`, `moan`, `stutter/burst`, `tremble`) plus sparse non-vocal seasoning
  flags, and SHALL be a pure function of affect, complexity, punctuation, and phrase
  position.
- **FR-53** `FORMAT_V2` synthesis SHALL render selected gesture archetypes with bounded,
  deterministic yelp, moan, stutter/burst, and tremble texture paths plus sparse servo
  and noise-tail seasoning, all inside finite BB-8-family parameter bounds.

---

## 2. Non-functional requirements

### 2.1 Determinism

- **NFR-1** For a fixed format version, identical input text SHALL produce
  byte-identical audio output across repeated runs on the same machine.
- **NFR-2** For a fixed format version, identical input text SHALL produce
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
- **NFR-7** The embedded baked mapping table SHALL be on the order of ~300 KB (header +
  ~30k per-token records of 4×int16 + int16 weight).
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
  parameter space, preserving a consistent BB-8-family identity. `FORMAT_V1` achieves
  this by varying only the four bounded semantic axes; `FORMAT_V2` MAY additionally vary
  deterministic, bounded phrase, affect, complexity, and archetype channels.

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
