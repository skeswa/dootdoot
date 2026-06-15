# Documentation Map

Start with the four source-of-truth documents. These stay at the top level because they
define the architecture, requirements, work plan, and enforced development rules:

| Document                 | Use it for                                                               |
| ------------------------ | ------------------------------------------------------------------------ |
| [`design.md`](design.md) | Full rationale, pipeline, runtime/build split, and voice contract shape. |
| [`spec.md`](spec.md)     | Normative `FR-*` and `NFR-*` requirements.                               |
| [`plan.md`](plan.md)     | Phased `T-*` implementation tasks and live progress tracking.            |
| [`style.md`](style.md)   | Mandatory Rust style, testing, linting, and workflow rules.              |

Supporting documents live below:

## Reference

Stable contracts and low-level implementation notes.

- [`usage.md`](usage.md) — CLI examples, `--explain`, edge cases, and input limits.
- [`reference/asset_regeneration.md`](reference/asset_regeneration.md) — safe `xtask`
  regeneration flow and voice-bump rules.
- [`reference/format_v1.md`](reference/format_v1.md) — committed `format_v1.bin` layout.
- [`reference/voice_v2_scope.md`](reference/voice_v2_scope.md) — scoped V2
  performance channels and bounds.
- [`reference/mathx.md`](reference/mathx.md) — owned transcendental math contract.
- [`reference/packaging.md`](reference/packaging.md) — install, package smoke, and
  release-channel decisions.
- [`reference/cross-platform-determinism.md`](reference/cross-platform-determinism.md)
  — golden-hash platform guarantee.

## Research

Evidence and proposals that informed the sound design.

- [`research/bb8-sound-signature-analysis.md`](research/bb8-sound-signature-analysis.md)
  — Phase 7 timbre-gap analysis against the local BB-8 reference corpus.
- [`research/bb8-expressiveness-gap-analysis.md`](research/bb8-expressiveness-gap-analysis.md)
  — phrase, affect, word-complexity, and gesture-palette gaps that informed
  `VOICE_V2` work.
- [`research/bb8-word-attack-forensics.md`](research/bb8-word-attack-forensics.md) —
  word-boundary onset analysis that informed `VOICE_V5`.
- [`research/bb8-repeated-phrase-tremolo-forensics.md`](research/bb8-repeated-phrase-tremolo-forensics.md)
  — repeated-phrase modulation analysis that informed `VOICE_V6`.

## Validation

Short acceptance notes for decisions that were finalized during voice freezes.

- [`validation/voice-tuning.md`](validation/voice-tuning.md) — final Phase 7 voice tuning
  acceptance.
- [`validation/voice-v2-expressiveness.md`](validation/voice-v2-expressiveness.md) —
  final `VOICE_V2` phrase/affect/complexity/archetype acceptance.
- [`validation/voice-v3-smoothing.md`](validation/voice-v3-smoothing.md) — final
  `VOICE_V3` phrase-continuity acceptance.
- [`validation/voice-v4-onset-smoothing.md`](validation/voice-v4-onset-smoothing.md) —
  final `VOICE_V4` repeated-onset smoothing acceptance.
- [`validation/voice-v5-word-attack-smoothing.md`](validation/voice-v5-word-attack-smoothing.md)
  — final `VOICE_V5` word-attack smoothing acceptance.
- [`validation/voice-v6-repeated-phrase-smoothing.md`](validation/voice-v6-repeated-phrase-smoothing.md)
  — final `VOICE_V6` repeated-phrase smoothing acceptance.
- [`validation/learnability-spread.md`](validation/learnability-spread.md) — final
  semantic cluster spread check.
- [`validation/squash.md`](validation/squash.md) — finalized tanh z-score squash choice.
