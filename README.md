<h1 align="center">dootdoot</h1>

<p align="center">Turn text into deterministic, BB-8-style droid speech, right from your terminal.</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Status: early development" src="https://img.shields.io/badge/status-early%20development-orange.svg">
  <img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust-dea584.svg">
</p>

<!-- Add these once the project ships: CI status, crates.io version, downloads. -->

> **Status: VOICE_V6 repeated-phrase smoothing is active.** `VOICE_V1` remains the locked
> v1 contract, `VOICE_V2` remains the locked expressiveness contract, `VOICE_V3` remains
> the locked phrase-continuity contract, `VOICE_V4` remains the locked repeated-onset
> smoothing contract, `VOICE_V5` remains the locked word-attack smoothing contract, and
> the active branch now smooths repeated phrase pulsing. Packaging work is still in
> progress. See
> [the roadmap](docs/plan.md) and
> [packaging notes](docs/reference/packaging.md).

`dootdoot` reads text and emits short bursts of warbly droid chatter. It's a small,
learnable sound language with three defining properties:

- **Deterministic.** The same text always produces the same audio, bit-for-bit, on every platform.
- **Semantic.** Text with similar _meaning_ sounds similar, so you can learn to "hear" words.
- **Droid by design.** However you phrase the input, the output is unmistakably the same character.

## Installation

**From source** (requires the [Rust toolchain](https://rustup.rs)):

```sh
git clone https://github.com/skeswa/dootdoot
cd dootdoot
cargo build --release
# binary at ./target/release/dootdoot
```

**Local Cargo install** from a checkout:

```sh
cargo install --path dootdoot --locked
```

Prebuilt binaries and Homebrew are deferred until tag-based release automation exists.

## Usage

```sh
dootdoot "hello there"               # synthesize and play it
dootdoot "hello there" -o out.wav    # render to a WAV file (no playback)
dootdoot "hello there" -o out.wav --play   # do both
echo "piped text" | dootdoot         # read from stdin
dootdoot "curious?" --explain        # print the per-token sound breakdown
```

`--explain` writes a full account of the sound profile to stderr so it never pollutes file
output or shell pipelines. It is an inspection mode, so it does **not** play audio (pass
`--play` if you also want to hear it). It reports every channel that affects the rendered
samples: the utterance-level `mood` and `complexity`; per voiced token the four learnable
knobs, the discourse `role`, and the gesture `archetype`; the planner's continuous
performance `curves`; and the glide/pause each control marker imposes:

```text
mood        valence:+0.325  arousal:+0.357
complexity  scalar:+0.192  subtokens:0  chars:8

token   │  pitch │  vowel │ contour │ warble │ role              │ archetype
curious │ -0.660 │ +0.058 │  -0.390 │ +0.762 │ terminal-flourish │ yelp
?       │ control:question · rising glide · pause 240 ms

curves  │ p.bias │  p.vel │  f.tgt │  f.vel │ bright │  mouth │   tens │ gap
curious │ +0.350 │ +0.550 │ +0.300 │ +0.350 │  0.550 │  0.550 │  0.600 │ -
```

The four knob columns depend only on the words (the learnable semantic core), so two
inputs that differ only in punctuation share them — but their `role`, `archetype`, `curves`,
`mood`, and control rows reveal why they still sound different. The `mood`, `complexity`,
and `control:` rows are not voiced tokens; they shape neighboring syllables and pauses.

## Documented behavior

- `Hello` and `hello` sound the same because the embedded `potion-base-8M` tokenizer is
  uncased.
- Empty or whitespace-only input emits the fixed inquisitive `?` chirp and exits 0.
- Literal `[PAD]`, `[CLS]`, `[SEP]`, and `[MASK]` are dropped. `[UNK]` is kept and voiced
  with its own mapping.
- Prosodic punctuation (`.`, `!`, `?`, `,`, `;`, `:`) is control-only; other symbols are
  tokenized and voiced normally.
- Non-Latin text and emoji are accepted, but v1 is English-oriented and will often route
  through `[UNK]` or repetitive WordPiece shapes.
- Very large input warns past about 8 minutes of rendered audio and errors before the
  fixed 30 minute / 160 MB ceiling.

## How it works

Text is tokenized, each token is mapped to a semantic vector, those vectors are reduced
to four perceptual "knobs" (pitch, vowel, glide, warble), and a fixed formant-synthesis
voice turns the knobs into sound. Because the whole mapping is frozen and uses only
pinned math, identical input yields identical output everywhere.

Full rationale lives in [`docs/design.md`](docs/design.md); the requirements are in
[`docs/spec.md`](docs/spec.md). The full documentation map is
[`docs/README.md`](docs/README.md).

## Contributing

Development conventions, the architecture, and the test-first (red-green) workflow are
documented in [`CLAUDE.md`](CLAUDE.md) and [`docs/style.md`](docs/style.md). The build
plan is in [`docs/plan.md`](docs/plan.md).

```sh
cargo test    # run the suite (see docs/style.md §11 for the full toolchain)
```

## License

[MIT](LICENSE).

---

<sub>dootdoot is an independent project that produces sounds _reminiscent of_ droids from
science fiction. It is not affiliated with, endorsed by, or associated with Lucasfilm or
The Walt Disney Company. "BB-8" is a trademark of its respective owner.</sub>
