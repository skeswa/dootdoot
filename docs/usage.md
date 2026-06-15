# Usage Guide

`dootdoot` turns text into deterministic droid sound. The same text and voice version
produce the same canonical 44.1 kHz, 16-bit, mono buffer used for both playback and WAV
output.

## Commands

```sh
cargo run -p dootdoot -- "hello there"
cargo run -p dootdoot -- "hello there" -o hello.wav
cargo run -p dootdoot -- "hello there" -o hello.wav --play
echo "piped text" | cargo run -p dootdoot --
cargo run -p dootdoot -- "curious?" --explain
```

Installed binaries use the same arguments without `cargo run -p dootdoot --`.

Output routing is fixed:

| Arguments        | Behavior                     |
| ---------------- | ---------------------------- |
| no `-o`          | Plays the canonical buffer.  |
| `-o FILE`        | Writes a WAV file only.      |
| `-o FILE --play` | Writes the WAV and plays it. |

## `--explain`

`--explain` prints to stderr and does not affect audio output. The table gives one row
per voiced token and one control row for prosodic punctuation.

```text
token │ pitch │ vowel │ contour │ warble
hello │ +0.185 │ -0.340 │ +0.512 │ -0.118
? │ control:question │ - │ - │ -
```

The four numeric columns are the fixed semantic sound knobs:

| Column    | Meaning                    |
| --------- | -------------------------- |
| `pitch`   | Pitch-center offset.       |
| `vowel`   | Formant/vowel position.    |
| `contour` | Glide direction and shape. |
| `warble`  | Vibrato and flutter depth. |

Prosodic punctuation rows are control-only. They can shape the preceding syllable and
pause, but they do not create a separate voiced syllable.

## Edge Cases

- Empty or whitespace-only input renders the fixed inquisitive `?` chirp and exits 0.
- No positional text with an interactive stdin also routes to the empty chirp in the
  current shell implementation.
- `Hello` and `hello` tokenize identically because the embedded model tokenizer is
  uncased.
- Literal `[PAD]`, `[CLS]`, `[SEP]`, and `[MASK]` are filtered out after tokenization.
  `[UNK]` is deliberately kept and voiced.
- Prosodic punctuation is limited to `.`, `!`, `?`, `,`, `;`, and `:`. Other symbols are
  voiced normally when the tokenizer produces a voiced token.
- Non-Latin scripts and emoji are accepted, but the embedded English-oriented semantic
  mapping often routes them through `[UNK]` or repetitive subword shapes.

## Limits

The CLI estimates rendered output size before synthesis. It warns after about 8 minutes
of audio, then refuses to render before the fixed 30 minute / 160 MB ceiling. The byte and
duration limits are normative; token-count descriptions are only rough shorthand because
punctuation and word boundaries affect timing.

## Version Contract

`dootdoot --version` surfaces the active voice identifier. The current binary reports
`dootdoot VOICE_V6`. `VOICE_V1` is still the locked v1 contract, `VOICE_V2` is the
locked expressiveness contract, `VOICE_V3` is the locked phrase-continuity contract, and
`VOICE_V4` is the locked repeated-onset smoothing contract. `VOICE_V5` is the locked
word-attack smoothing contract, and `VOICE_V6` is the active repeated-phrase smoothing
contract. Any further rendered-sample change requires a new voice identifier and
regenerated golden WAV hashes.

The `VOICE_V*` identifier names the rendered-output contract, not the binary layout of
the baked semantic mapping asset. The current `VOICE_V6` renderer still embeds the
locked token-to-axis table from `assets/dootdoot_asset_v1.doot`.
