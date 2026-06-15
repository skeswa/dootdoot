# Documentation Map

Start with the four source-of-truth documents. These stay at the top level because they
define the architecture, requirements, work plan, and enforced development rules:

| Document                 | Use it for                                                                |
| ------------------------ | ------------------------------------------------------------------------- |
| [`design.md`](design.md) | Full rationale, pipeline, runtime/build split, and format contract shape. |
| [`spec.md`](spec.md)     | Normative `FR-*` and `NFR-*` requirements.                                |
| [`plan.md`](plan.md)     | Phased `T-*` implementation tasks and live progress tracking.             |
| [`style.md`](style.md)   | Mandatory Rust style, testing, linting, and workflow rules.               |

Supporting documents live below:

## Reference

Stable contracts and low-level implementation notes.

- [`reference/format_v1.md`](reference/format_v1.md) — committed `format_v1.bin` layout.
- [`reference/mathx.md`](reference/mathx.md) — owned transcendental math contract.
- [`reference/cross-platform-determinism.md`](reference/cross-platform-determinism.md)
  — golden-hash platform guarantee.

## Research

Evidence and proposals that informed the sound design.

- [`research/bb8-sound-signature-analysis.md`](research/bb8-sound-signature-analysis.md)
  — Phase 7 timbre-gap analysis against the local BB-8 reference corpus.
- [`research/bb8-expressiveness-gap-analysis.md`](research/bb8-expressiveness-gap-analysis.md)
  — phrase, affect, word-complexity, and gesture-palette gaps for future `FORMAT_V2`
  work.

## Validation

Short acceptance notes for decisions that were finalized during the `FORMAT_V1` freeze.

- [`validation/voice-tuning.md`](validation/voice-tuning.md) — final Phase 7 voice tuning
  acceptance.
- [`validation/learnability-spread.md`](validation/learnability-spread.md) — final
  semantic cluster spread check.
- [`validation/squash.md`](validation/squash.md) — finalized tanh z-score squash choice.
