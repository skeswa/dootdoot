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

`--explain` writes a table to stderr so it never pollutes file output or shell
pipelines. The columns are the four learnable sound knobs:

```text
token │ pitch │ vowel │ contour │ warble
hello │ +0.185 │ -0.340 │ +0.512 │ -0.118
? │ control:question │ - │ - │ -
```

The exact numbers depend on the frozen mapping and active voice. Punctuation and
performance rows are control markers: they shape neighboring voiced syllables and pauses,
but do not produce their own voiced tokens.

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
