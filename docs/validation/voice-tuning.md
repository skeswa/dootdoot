# VOICE_V1 voice tuning acceptance

Status: **Accepted for VOICE_V1**.

This note records the integrated Phase 7 tuning decision after T-46 through T-50 landed.
The comparison workflow is `scripts/bb8-metrics /Users/skeswa/repos/anddav87/bb8-sounds
target/bb8-metrics`, which decodes the local BB-8 reference clips, renders the fixed
dootdoot corpus, and reports directional metrics. These numbers are not a golden
contract; they are the tuning aid used before the Phase 8 hash freeze.

| set             | clips | active fraction | active island median ms | magnitude centroid hz | magnitude rolloff 85 hz | dominant peak motion hz | harmonicity | sub-500 hz power | 500-2000 hz power | 2000-5000 hz power | over-6000 hz power |
| --------------- | ----: | --------------: | ----------------------: | --------------------: | ----------------------: | ----------------------: | ----------: | ---------------: | ----------------: | -----------------: | -----------------: |
| BB-8 reference  |    32 |           0.467 |                 290.249 |              2382.033 |                4651.172 |                1335.059 |       0.806 |            0.527 |             0.328 |              0.036 |              0.003 |
| dootdoot corpus |     6 |           0.543 |                 185.760 |              1977.338 |                3552.979 |                 667.529 |       0.951 |            0.571 |             0.488 |              0.059 |              0.001 |

Acceptance rationale:

- **body**: the low-body layer now lands in the same broad region as the BB-8 reference
  corpus instead of the near-zero sub-500 Hz baseline from the pre-tuning voice.
- **upper-mid brightness**: the rolloff and 2-5 kHz band moved upward while keeping >6 kHz
  modest, matching the reference caveat that brightness mainly lives in the upper-mid
  region, not the high treble.
- **gesture motion**: internal pitch/vowel trajectories and compound warble give single
  tokens motion without adding runtime randomness.
- **harmonicity**: transient/body/sparkle layers reduce the perfectly clean oscillator
  identity while preserving a pitched formant core.
- **phrase air**: the envelope and pause template reduce active density and leave more
  space between phrase units.

Known residual tradeoff: dootdoot remains cleaner and more semantically regular than the
local reference set, which contains many one-off sound-effect gestures. That is acceptable
for v1 because the learnable four-axis mapping needs a consistent voice family rather than
a catalog of unrelated effects.
