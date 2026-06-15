# BB-8 word-attack forensics

## Prompt

`VOICE_V4` made repeated subword phrases more continuous, but the phrase
`I am so excited I am so excited I am so excited I am so excited` still produced sharp,
regular word attacks. The reference clip
`bb8-clips/inquisitive-then-chatty.mp3` has a softer onset that reads more like
`wah` or `woh`.

## Method

The investigation used three complementary checks:

- `ffmpeg` spectrograms and waveform crops for the generated phrase and
  `inquisitive-then-chatty.mp3`.
- A frame-RMS onset detector to locate large energy rises.
- A focused word-boundary metric: RMS level and derivative RMS in the first 18 ms after
  a bridged word boundary, divided by the following 45-85 ms body window.

The existing `scripts/bb8-metrics` report was also rerun to keep the finding in context.
It still showed dootdoot as cleaner and more harmonically regular than the reference
set, so the onset problem was treated as the first local fix rather than the whole
timbre gap.

## Finding

The harsh word attack was not the explicit transient fixed in `VOICE_V4`. Word starts
were already connected and skipped that transient. The sharpness came from a level and
shape mismatch:

- `VOICE_V3`/`VOICE_V4` render word pauses as quiet bridges.
- The next connected syllable reused the high subword connection floor.
- The first 18 ms of the word body therefore jumped above both the bridge and the later
  vowel body.
- The vowel trajectory started near the semantic vowel target, so the onset sounded like
  a synth block beginning, not a rounded vocal opening.

Measured on the repeated excited phrase, `VOICE_V4` word starts had a median
word-start/body level ratio of `2.714` and roughness ratio of `2.376`.

## Fix Direction

`VOICE_V5` separates word-boundary connections from subword connections:

- Subword connections keep the high floor needed for same-word continuity.
- Word-boundary starts ramp from a low bridge-matched floor into the syllable body.
- Word-boundary vowels start from a rounded `oo`-leaning pre-shape, then open into the
  semantic vowel target over a bounded window.
- Upper-mid sparkle and archetype texture are damped during that opening, then bloom
  back in.

After the change, the same phrase measured `0.375` for word-start/body level ratio and
`0.361` for roughness ratio. The first phrase start remains allowed to be sharper; the
fix targets bridged word starts inside a phrase.

## Future Tools

Useful next forensic upgrades:

- Add a small in-repo onset report that emits level ratio, derivative ratio, zero-crossing
  rate, and spectrogram thumbnails for named fixtures.
- Add mel-band or Bark-band onset flux so upper-mid harshness can be tracked without
  over-weighting inaudible high harmonics.
- Add LPC/formant-trace estimates for reference clips so vowel opening can be compared
  directly instead of inferred from waveform shape.
