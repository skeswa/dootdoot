# VOICE_V3 phrase-continuity acceptance

Status: **Accepted for VOICE_V3**.

`VOICE_V3` keeps the VOICE_V2 semantic, phrase, affect, complexity, and archetype
channels, then changes the sample-affecting phrase renderer to reduce token-to-token
staccato.

## Scope

- `dootdoot --version` reports `dootdoot VOICE_V3`.
- The embedded semantic mapping remains the locked token-to-axis table in
  `assets/dootdoot_asset_v1.doot`.
- Word boundaries keep deterministic duration, but render as quiet transition bridges
  instead of hard zero-filled gaps.
- Connected syllables share oscillator/filter phase and use connected envelope edges.
- The envelope keeps its internal droid dip, but the dip no longer clamps to silence.
- The committed golden WAV hashes were regenerated under `VOICE_V3`.

## Directional Check

Fixture phrase: `I am so excited wooohooo!`.

| set                    | hard zero runs in phrase body | active islands | median active island ms |
| ---------------------- | ----------------------------: | -------------: | ----------------------: |
| previous dootdoot      |                            12 |             15 |                  43.900 |
| VOICE_V3 dootdoot      |                             0 |              1 |                1752.018 |
| BB-8 excited reference |                             0 |              4 |                 792.200 |

The check is directional rather than a golden acceptance gate: BB-8 references still vary
by scene and edit, while dootdoot's sample-level contract is enforced by the golden WAV
hash fixture.
