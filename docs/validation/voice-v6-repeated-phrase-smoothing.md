# VOICE_V6 repeated-phrase smoothing acceptance

Status: **Accepted for VOICE_V6**.

`VOICE_V6` keeps the `VOICE_V5` word-attack smoothing and reduces regular tremolo-like
pulsing in repeated high-arousal phrases.

## Scope

- `dootdoot --version` reports `dootdoot VOICE_V6`.
- The embedded semantic mapping remains the locked token-to-axis table in
  `assets/dootdoot_asset_v1.doot`.
- Word bridges render as low, flatter connectors rather than foreground pulses.
- Word-connected syllables keep smoother pitch inheritance over a longer opening.
- Word-connected complexity, internal pitch, archetype pitch, and texture motion are
  damped so repeated words do not fully re-articulate every local gesture.
- The committed golden WAV hashes were regenerated under `VOICE_V6`.

## Directional Check

Fixture phrase:
`I am so excited I am so excited I am so excited I am so excited`.

The check measures bridge RMS relative to the preceding syllable and compares low-rate
envelope energy at the word-cycle rate against energy at twice that rate. Lower
double/word values mean the phrase is not dominated by the tremolo-like two-pulse word
cycle heard in `VOICE_V5`.

| set               | median bridge/syllable RMS | word-cycle energy at 3.705 Hz | double-cycle energy at 7.409 Hz | double/word |
| ----------------- | -------------------------: | ----------------------------: | ------------------------------: | ----------: |
| VOICE_V5 dootdoot |                      1.522 |                         1.463 |                          16.535 |      11.305 |
| VOICE_V6 dootdoot |                      0.444 |                        15.556 |                           8.583 |       0.552 |

This is a directional listening aid. The sample-level contract remains the golden WAV
hash fixture.
