<h1 align="center">dootdoot</h1>

<p align="center">Turn text into deterministic, BB-8-style droid speech, right from your terminal.</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Status: early development" src="https://img.shields.io/badge/status-early%20development-orange.svg">
  <img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust-dea584.svg">
</p>

<!-- Add these once the project ships: CI status, crates.io version, downloads. -->

> **Status: VOICE_V11 natural-voice tuning is active.** `VOICE_V11` softens syllable
> attacks, adds deterministic intra-phrase rubato, localizes dash hesitation breath, and
> integrates aspiration so breath reads as part of the voice instead of a separate hiss.
> Earlier `VOICE_V*` contracts remain historical lock points, and any sample-affecting
> change still requires a new voice identifier plus regenerated golden WAV fixtures. macOS
> release automation is committed; the first tagged release will publish Homebrew and
> prebuilt installer artifacts. See
> [the roadmap](docs/plan.md) and
> [packaging notes](docs/reference/packaging.md).

`dootdoot` reads text and emits short bursts of warbly droid chatter. It's a small,
learnable sound language with three defining properties:

- **Deterministic.** The same text always produces the same audio, bit-for-bit, on every platform.
- **Semantic.** Text with similar _meaning_ sounds similar, so you can learn to "hear" words.
- **Droid by design.** However you phrase the input, the output is unmistakably the same character.

## Installation

**Homebrew** (recommended on macOS, available after the first tagged release):

```sh
brew install skeswa/tap/dootdoot
brew upgrade dootdoot
```

**Installer script** (macOS fallback without Homebrew, available after the first tagged release):

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/skeswa/dootdoot/releases/latest/download/dootdoot-installer.sh | sh
dootdoot-update
```

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
- Prosodic punctuation (`.`, `!`, `?`, `,`, `;`, `:`) is control-only; question,
  statement, exclamation, and clause marks each shape the preceding syllable and pause.
- Standalone dash forms (`-`, `--`, en dash, em dash) and ellipses (`...`, `…`) are
  control-only hesitation markers. A dash clips the previous tail; an ellipsis trails off.
- Other symbols are tokenized and voiced normally when the tokenizer produces a voiced
  token.
- Neutral multi-word text still gets deterministic semantic accents, short word rests, and
  V11 rubato; punctuation is not required for the voice to move.
- Non-Latin text and emoji are accepted, but the semantic mapping is English-oriented and
  will often route through `[UNK]` or repetitive WordPiece shapes.
- Very large input warns past about 8 minutes of rendered audio and errors before the
  fixed 30 minute / 160 MB ceiling.

## How it works

Text is tokenized, each token is mapped through the embedded `.doot` asset to a semantic
vector, those vectors are reduced to four perceptual knobs (pitch, vowel, glide, warble),
and a deterministic performance planner adds phrase roles, local affect/archetypes,
pauses, rests, and motion curves. A fixed formant-synthesis voice then renders the plan
to one canonical 44.1 kHz / 16-bit / mono buffer for both playback and WAV output.

The runtime does not load `model2vec`, `candle`, or any tensor framework. `xtask`
generates `assets/dootdoot_asset_v1.doot` ahead of time; the shipped CLI embeds that
asset and uses pinned math, so identical input yields identical output on the
CI-verified platforms.

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
