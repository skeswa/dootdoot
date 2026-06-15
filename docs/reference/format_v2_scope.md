# `FORMAT_V2` Scope

`FORMAT_V2` broadens the performance contract without replacing the semantic core. The
four PCA-derived knobs remain the learnable meaning layer:

1. pitch center,
2. vowel/formant position,
3. contour/glide shape,
4. warble depth.

V2 may add only deterministic, bounded performance channels around that core:

| Channel       | Role                                                                                       |
| ------------- | ------------------------------------------------------------------------------------------ |
| Phrase timing | Boundary strength, pause length, pitch reset, declination, and pre-boundary lengthening.   |
| Affect        | Utterance valence and arousal from licensing-safe lexical and punctuation signals.         |
| Complexity    | WordPiece/character-shape scalar for articulation density and duration scaling.            |
| Archetype     | A small palette of gesture shapes such as chatter, yelp, moan, stutter/burst, and tremble. |

Each channel must be a pure function of the token/control-event stream. No runtime
randomness, clock input, user seed, external service, or platform-dependent data is part
of the contract.

## Bounds

Every V2 channel must publish fixed bounds or a finite palette. Renderers clamp to those
bounds before audio generation, so every input stays inside the BB-8-family parameter
space even when affect, complexity, or punctuation is extreme.

## Explain Output

`--explain` should expose V2 channels when they help a listener learn the language. Token
semantic rows remain first-class. Mood, phrase, complexity, or archetype rows may be
added as control/performance rows, but they must go to stderr and must not change output
routing.

## Format Rule

Any V2 channel that can alter a rendered sample belongs to the format contract and
requires regenerated golden WAV hashes. V1 assets and constants remain locked.

## Implemented Phrase Prosody

The first V2 performance channel applies the pure phrase plan to synthesis:

- word boundaries keep the base syllable duration and use the existing word pause;
- clause punctuation uses `8,397` syllable samples plus the medium punctuation pause;
- sentence punctuation uses `9,371` syllable samples plus the long punctuation pause;
- phrase declination, pitch reset, final lowering, and sparse emphasis are deterministic
  scalar offsets applied before syllable rendering;
- consecutive punctuation keeps the first marker's glide/lengthening role while the
  longest single pause wins.

The active CLI version string is `FORMAT_V2`; the embedded semantic mapping artifact is
still the locked `format_v1.bin` table.

## Implemented Affect Analysis

V2 affect analysis reads the committed MIT VADER valence table and owned arousal-signal
configuration. It returns per-token valence/arousal rows plus an utterance mood:

- valence is the average normalized VADER score across matched tokens;
- arousal combines punctuation density, repeated markers, all-caps words, owned
  intensifiers/dampeners, token count, character/WordPiece complexity, and valence energy;
- all scores are deterministic and clamped to the documented fixed bounds.
