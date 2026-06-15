# VOICE_V4 repeated-onset smoothing acceptance

Status: **Accepted for VOICE_V4**.

`VOICE_V4` keeps the VOICE_V3 phrase-continuity renderer and smooths connected token
openings that still sounded click-like on repeated subwords.

## Scope

- `dootdoot --version` reports `dootdoot VOICE_V4`.
- The embedded semantic mapping remains the locked token-to-axis table in
  `assets/dootdoot_asset_v1.doot`.
- Connected syllables no longer replay the full attack transient.
- Connected pitch and vowel openings blend from the previous rendered state.
- Connected envelope starts ramp through the early body instead of replaying the full
  attack peak.
- The committed golden WAV hashes were regenerated under `VOICE_V4`.

## Directional Check

Fixture phrase: `hahahahahahahahahahah`.

The check measures median derivative roughness in the first 12 ms after connected
subword starts relative to a nearby 12 ms body window. Lower is smoother.

| set                         | median connected-onset roughness ratio |
| --------------------------- | -------------------------------------: |
| VOICE_V3 dootdoot           |                                  3.570 |
| VOICE_V4 dootdoot           |                                  1.821 |
| acceptance regression limit |                                  2.400 |

This is a directional listening aid. The sample-level contract remains the golden WAV
hash fixture.
