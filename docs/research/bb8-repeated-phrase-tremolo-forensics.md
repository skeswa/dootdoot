# BB-8 repeated-phrase tremolo forensics

## Prompt

After `VOICE_V5`, the phrase
`I am so excited I am so excited I am so excited I am so excited` was smoother at word
starts but still sounded repetitive and severe, almost like rapid bowing or tremolo.
The comparison target was the local clip
`/Users/skeswa/repos/anddav87/bb8-sounds/bb8-clips/inquisitive-then-chatty.mp3`.

## Method

The investigation used the canonical WAV render and a 44.1 kHz mono decode of the BB-8
clip. The main measurements were:

- 20 ms frame RMS with a 5 ms hop, measured over active frames.
- A small DFT over the log-RMS envelope to find low-rate modulation peaks.
- Exact word-cycle segmentation from `LEADING_SILENCE_SAMPLES`, syllable duration, and
  `WORD_PAUSE_SAMPLES`.
- Bridge RMS divided by the preceding syllable RMS.

## Finding

The harshness was no longer a word-attack spike. It was a repeated phrase-level pulse.
The rendered word cycle is about 270 ms: one syllable body plus one word bridge. In V5
the bridge often became louder than the syllable it connected, so the ear heard two
regular loudness lobes per word cycle.

Measured on the repeated excited phrase:

| set               | median bridge/syllable RMS | word-cycle energy at 3.705 Hz | double-cycle energy at 7.409 Hz | double/word |
| ----------------- | -------------------------: | ----------------------------: | ------------------------------: | ----------: |
| VOICE_V5 dootdoot |                      1.522 |                         1.463 |                          16.535 |      11.305 |
| VOICE_V6 dootdoot |                      0.444 |                        15.556 |                           8.583 |       0.552 |

The reference clip's strongest envelope motion was slower, around 0.6-1.2 Hz, not a
regular 7 Hz pulse.

## Fix Direction

`VOICE_V6` makes the bridge a low, flatter connector instead of a foreground syllable:

- The bridge keeps oscillator and formant state alive but reduces direct source,
  upper-mid sparkle, and warble.
- The bridge envelope changes from a strong half-sine peak to a flatter bed.
- Word-connected syllables damp repeated complexity articulation, internal pitch swoop,
  archetype pitch motion, and texture recovery.
- Word-connected pitch inherits prior state for longer, so repeated semantic jumps do
  not snap fully into place at every word.
- The connected-word amplitude envelope has reduced local contrast, keeping syllables
  vowel-like without reintroducing a hard attack.

The important balance is not "make bridges silent." Too little bridge energy returns the
phrase to short active islands. The accepted V6 target keeps bridges audible but below
the syllable body.
