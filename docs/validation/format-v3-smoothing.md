# FORMAT_V3 phrase-continuity acceptance

Status: **Accepted for FORMAT_V3**.

`FORMAT_V3` keeps the FORMAT_V2 semantic, phrase, affect, complexity, and archetype
channels, then changes the sample-affecting phrase renderer to reduce token-to-token
staccato.

## Scope

- `dootdoot --version` reports `dootdoot FORMAT_V3`.
- The embedded semantic mapping artifact remains the locked `assets/format_v1.bin`.
- Word boundaries keep deterministic duration, but render as quiet transition bridges
  instead of hard zero-filled gaps.
- Connected syllables share oscillator/filter phase and use connected envelope edges.
- The envelope keeps its internal droid dip, but the dip no longer clamps to silence.
- The committed golden WAV hashes were regenerated under `FORMAT_V3`.

## Directional Check

Fixture phrase: `I am so excited wooohooo!`.

| set                    | hard zero runs in phrase body | active islands | median active island ms |
| ---------------------- | ----------------------------: | -------------: | ----------------------: |
| previous dootdoot      |                            12 |             15 |                  43.900 |
| FORMAT_V3 dootdoot     |                             0 |              1 |                1752.018 |
| BB-8 excited reference |                             0 |              4 |                 792.200 |

The check is directional rather than a golden acceptance gate: BB-8 references still vary
by scene and edit, while dootdoot's sample-level contract is enforced by the golden WAV
hash fixture.
