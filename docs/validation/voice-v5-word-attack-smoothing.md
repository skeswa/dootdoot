# VOICE_V5 word-attack smoothing acceptance

Status: **Accepted for VOICE_V5**.

`VOICE_V5` keeps the VOICE_V4 repeated-subword smoothing and changes word-boundary
connected starts so they bloom into the syllable body instead of jumping out of the
quiet bridge.

## Scope

- `dootdoot --version` reports `dootdoot VOICE_V5`.
- The embedded semantic mapping artifact remains the locked `assets/format_v1.bin`.
- Subword and word-boundary connections are distinct renderer states.
- Word-boundary starts ramp from a low bridge-matched floor instead of reusing the high
  subword connection floor.
- Word-boundary vowels begin from a rounded `oo`-leaning pre-shape and open into the
  semantic vowel target.
- Upper-mid sparkle and archetype texture are damped during the word-opening bloom.
- The committed golden WAV hashes were regenerated under `VOICE_V5`.

## Directional Check

Fixture phrase:
`I am so excited I am so excited I am so excited I am so excited`.

The check measures median word-start energy and derivative roughness in the first 18 ms
after each bridged word boundary relative to the following 45-85 ms body window. Lower
values mean the word start swells into the body instead of spiking over it.

| set               | median word-start/body level ratio | median word-start/body roughness ratio |
| ----------------- | ---------------------------------: | -------------------------------------: |
| VOICE_V4 dootdoot |                              2.714 |                                  2.376 |
| VOICE_V5 dootdoot |                              0.375 |                                  0.361 |

This is a directional listening aid. The sample-level contract remains the golden WAV
hash fixture.
