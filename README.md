<h1 align="center">dootdoot</h1>

<p align="center">Turn text into deterministic, BB-8-style droid speech, right from your terminal.</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Status: early development" src="https://img.shields.io/badge/status-early%20development-orange.svg">
  <img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust-dea584.svg">
</p>

<!-- Add these once the project ships: CI status, crates.io version, downloads. -->

> **Status: early development.** The design is complete; implementation is in progress.
> Expect things to change. See [the roadmap](docs/plan.md).

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

Prebuilt binaries, `cargo install`, and Homebrew are planned for the first release.

## Usage

```sh
dootdoot "hello there"               # synthesize and play it
dootdoot "hello there" -o out.wav    # render to a WAV file (no playback)
dootdoot "hello there" -o out.wav --play   # do both
echo "piped text" | dootdoot         # read from stdin
dootdoot "curious?" --explain        # print the per-token sound breakdown
```

## How it works

Text is tokenized, each token is mapped to a semantic vector, those vectors are reduced
to four perceptual "knobs" (pitch, vowel, glide, warble), and a fixed formant-synthesis
voice turns the knobs into sound. Because the whole mapping is frozen and uses only
pinned math, identical input yields identical output everywhere.

Full rationale lives in [`docs/design.md`](docs/design.md); the requirements are in
[`docs/spec.md`](docs/spec.md).

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
