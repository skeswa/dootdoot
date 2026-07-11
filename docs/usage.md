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

`--explain` prints to stderr and does not affect audio output. It is a complete
account of why an utterance sounds the way it does: an utterance-level mood and
complexity summary, one row per voiced token (with control rows for prosodic
punctuation and hesitation markers), a planner-curves table, and — when the utterance
contains classed content words — the `VOICE_V12` class table:

```text
mood        valence:+0.000  arousal:+0.208
complexity  scalar:+0.288  subtokens:0  chars:12

token  │  pitch │  vowel │ contour │ warble │ role              │ archetype
verify │ -0.570 │ +0.797 │  -0.969 │ -0.279 │ chatty-reply      │ chatter
the    │ -0.055 │ +0.007 │  +0.025 │ +0.126 │ chatty-reply      │ tremble
bug    │ -0.306 │ -0.556 │  +0.745 │ -0.882 │ chatty-reply      │ chatter

curves │ p.bias │  p.vel │  f.tgt │  f.vel │ bright │  mouth │   tens │ gap
verify │ +0.000 │ +0.562 │ +0.000 │ +0.341 │  0.712 │  0.485 │  1.000 │ -
the    │ +0.064 │ +0.158 │ +0.000 │ +0.238 │  0.258 │  0.423 │  0.458 │ 40 ms
bug    │ +0.138 │ +0.235 │ +0.000 │ +0.290 │  0.335 │  0.454 │  0.535 │ -

class  │ pos  │ marker │ silhouette
verify │ verb │ chirp  │ stem→push ×2
the    │ -    │ -      │ blip
bug    │ noun │ click  │ stem→settle ×2
```

The four numeric columns of the token table are the fixed semantic sound knobs:

| Column    | Meaning                    |
| --------- | -------------------------- |
| `pitch`   | Pitch-center offset.       |
| `vowel`   | Formant/vowel position.    |
| `contour` | Glide direction and shape. |
| `warble`  | Vibrato and flutter depth. |

The class table is the `VOICE_V12` learnability training aid (FR-120):

| Column       | Meaning                                                                                                      |
| ------------ | ------------------------------------------------------------------------------------------------------------ |
| `pos`        | The word's baked class: `noun`, `verb`, or `-` (function words, unknown or ambiguous vocabulary).            |
| `marker`     | The layered co-onset marker the word-initial token fires: `click` (noun), `chirp` (verb), or none.           |
| `silhouette` | The compound shape: `stem→settle ×2` / `stem→push ×2`, `stem…`/`→settle` across subwords, or a plain `blip`. |

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
- Word classes (`VOICE_V12`) match baked surface forms case-insensitively. Words outside
  the baked table — including deliberately unmarked ambiguous coding lemmas like
  `build`, `fix`, `run`, and `update` — render as unclassified blips.

## Limits

The CLI estimates rendered output size before synthesis. It warns after about 8 minutes
of audio, then refuses to render before the fixed 30 minute / 160 MB ceiling. The byte and
duration limits are normative; token-count descriptions are only rough shorthand because
punctuation and word boundaries affect timing.

## Version Contract

`dootdoot --version` surfaces the active voice identifier. The current binary reports
`dootdoot VOICE_V12`, the noun/verb recognizability contract. Every earlier `VOICE_V*`
identifier (`VOICE_V1` through `VOICE_V11`) is a locked historical contract point — see
the acceptance notes in the [validation archive](README#validation) for what each froze. Any
further rendered-sample change requires a new voice identifier and regenerated golden
WAV fixtures.

The `VOICE_V*` identifier names the rendered-output contract, not the binary layout of
the baked assets. The current `VOICE_V12` renderer still embeds the locked
token-to-axis table from `assets/dootdoot_asset_v1.doot`, plus the `VOICE_V12`
noun/verb class table from `assets/dootdoot_pos_v1.doot`.
