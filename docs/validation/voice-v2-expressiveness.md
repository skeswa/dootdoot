# VOICE_V2 expressiveness acceptance

Status: **Accepted for VOICE_V2**.

`VOICE_V2` freezes the deterministic phrase, affect, complexity, and archetype
performance channels documented in
[`reference/voice_v2_scope.md`](../reference/voice_v2_scope.md). The four semantic axes
remain the learnable core; the new channels only shape bounded performance around those
axes.

`VOICE_V3` superseded V2 by smoothing connected phrase rendering, `VOICE_V4` smoothed
repeated connected onsets, `VOICE_V5` smoothed word attacks, and `VOICE_V6` is the
active voice contract. V2 remains the frozen expressiveness checkpoint.

## Checks

- `dootdoot --version` reports `dootdoot VOICE_V2`.
- `scripts/bb8-metrics <reference-recordings-dir> target/bb8-metrics-v2`
  renders the short BB-8 reference corpus and the contextual clips under `bb8-clips/`.
- The contextual metrics report is written to `target/bb8-metrics-v2/contextual/` after
  decoding clips into `target/bb8-metrics-v2/contextual-wav/`.
- `cargo test -p dootdoot-core --test golden_wav_hashes` verifies the committed golden
  WAV hashes for the frozen v2 corpus.

Short-clip directional metrics:

| set             | clips | active fraction | active island median ms | magnitude centroid hz | magnitude rolloff 85 hz | dominant peak motion hz | harmonicity | sub-500 hz power | 500-2000 hz power | 2000-5000 hz power | over-6000 hz power |
| --------------- | ----: | --------------: | ----------------------: | --------------------: | ----------------------: | ----------------------: | ----------: | ---------------: | ----------------: | -----------------: | -----------------: |
| BB-8 reference  |    32 |           0.467 |                 290.249 |              2382.033 |                4651.172 |                1335.059 |       0.806 |            0.527 |             0.328 |              0.036 |              0.003 |
| dootdoot corpus |     6 |           0.548 |                 116.100 |              2139.833 |                3660.645 |                 667.529 |       0.947 |            0.616 |             0.506 |              0.075 |              0.001 |

Contextual-clip directional metrics:

| set             | clips | active fraction | active island median ms | magnitude centroid hz | magnitude rolloff 85 hz | dominant peak motion hz | harmonicity | sub-500 hz power | 500-2000 hz power | 2000-5000 hz power | over-6000 hz power |
| --------------- | ----: | --------------: | ----------------------: | --------------------: | ----------------------: | ----------------------: | ----------: | ---------------: | ----------------: | -----------------: | -----------------: |
| BB-8 reference  |     7 |           0.988 |                 499.229 |              1223.395 |                1744.189 |                2648.584 |       0.800 |            0.333 |             0.534 |              0.003 |              0.000 |
| dootdoot corpus |     6 |           0.548 |                 116.100 |              2139.833 |                3660.645 |                 667.529 |       0.947 |            0.616 |             0.506 |              0.075 |              0.001 |

## Contextual Clips

The directional contextual clip set is:

- `lost-friends-sad`;
- `excited-explanation`;
- `found-fixed-excitement`;
- `left-behind-anxious`;
- `explosion-surprise`;
- `enemy-approaching-alarm`;
- `inquisitive-then-chatty`.

These clips are directional acceptance aids, not asset-spec constants. They check that sad,
excited, anxious, surprised, alarm-like, and inquisitive phrases move the v2 performance
channels in the intended directions while the golden WAV hashes remain the sample-level
contract.

## Decision

By-ear acceptance for the local v2 corpus is recorded as accepted. Phrase timing now
creates deterministic boundaries, resets, lengthening, and declination. Affect supplies
bounded valence/arousal mood shaping. Complexity adds bounded compound articulation from
owned WordPiece and character signals. Archetype selection and rendering add fixed
chatter, yelp, moan, stutter/burst, and tremble gesture families plus sparse deterministic
servo/noise seasoning.
