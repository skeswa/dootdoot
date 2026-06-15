# VOICE_V1 learnability spread validation

Status: **Finalized for VOICE_V1**.

The final Phase 7 voice keeps the semantic axes audible enough for the v1 learnability
goal. The fixed voice DNA stays consistent, but semantic clusters still occupy different
knob regions and render as distinct gestures.

Validation fixtures:

- `cat` and `dog` remain close semantic clusters and should sound audibly similar.
- `cat` and `airplane` remain farther apart and should sound audibly distinct.
- The existing semantic-sanity tests continue to assert the token and sequence ordering;
  the T-53 spread test adds an audio-distance check over the final rendered buffers.

No axis range adjustment was needed after the final BB-8-family tuning pass.
