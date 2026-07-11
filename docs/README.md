# Documentation Map

This directory is also the source for the project website. The site renders these Markdown
files directly, so the readable source and published documentation cannot drift. Collection
navigation for `reference/`, `research/`, and `validation/` is generated automatically.

```sh
npm install
npm run docs:dev       # local site with hot reload
npm run test:docs      # site behavior checks
npm run docs:build     # production build
```

Pushes to `main` that touch the site, its golden audio samples, or its toolchain deploy
automatically to `https://skeswa.github.io/dootdoot/` through the pinned
`.github/workflows/docs.yml` GitHub Pages workflow.

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
- [`reference/dootdoot_asset_spec.md`](reference/dootdoot_asset_spec.md) — committed
  `.doot` Protocol Buffers asset spec.
- [`reference/voice_v2_scope.md`](reference/voice_v2_scope.md) — scoped V2
  performance channels and bounds.
- [`reference/mathx.md`](reference/mathx.md) — owned transcendental math contract.
- [`reference/packaging.md`](reference/packaging.md) — install, package smoke, and
  release-channel decisions.
- [`reference/releasing.md`](reference/releasing.md) — how a release is cut, tagged,
  built, and published, plus the required secrets.
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
- [`research/bb8-inquisitive-chatty-gap-analysis.md`](research/bb8-inquisitive-chatty-gap-analysis.md)
  — contextual-performance gaps that informed `VOICE_V7`.
- [`research/bb8-corpus-timbre-texture-analysis.md`](research/bb8-corpus-timbre-texture-analysis.md)
  — corpus timbre/texture comparison that informed `VOICE_V8`.
- [`research/punctuation-prosody-audibility.md`](research/punctuation-prosody-audibility.md)
  — audible-punctuation analysis that informed `VOICE_V9`.
- [`research/bb8-sound-vocabulary-taxonomy.md`](research/bb8-sound-vocabulary-taxonomy.md)
  — gesture-level sound-type taxonomy that informed `VOICE_V10`.
- [`research/droid-synth-prior-art.md`](research/droid-synth-prior-art.md) — prior-art
  survey of droid-voice synthesis techniques.
- [`research/noun-verb-recognizability.md`](research/noun-verb-recognizability.md) —
  auditory-cognition research and plan behind the `VOICE_V12` noun/verb class
  signature.
- [`research/voice-v12-spike-evaluation.md`](research/voice-v12-spike-evaluation.md)
  — the `VOICE_V12` spike A/B worksheet: by-ear rounds, locked recipe, and the
  ambiguity-policy decision.

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
- [`validation/voice-v7-contextual-performance.md`](validation/voice-v7-contextual-performance.md)
  — final `VOICE_V7` contextual-performance acceptance.
- [`validation/voice-v8-semantic-engagement.md`](validation/voice-v8-semantic-engagement.md)
  — final `VOICE_V8` semantic-engagement acceptance.
- [`validation/voice-v9-audible-punctuation.md`](validation/voice-v9-audible-punctuation.md)
  — final `VOICE_V9` audible-punctuation acceptance.
- [`validation/voice-v10-bidirectional-whistle.md`](validation/voice-v10-bidirectional-whistle.md)
  — final `VOICE_V10` gesture-vocabulary acceptance.
- [`validation/voice-v11-natural-voice.md`](validation/voice-v11-natural-voice.md) —
  final `VOICE_V11` natural-voice acceptance.
- [`validation/voice-v12-noun-verb.md`](validation/voice-v12-noun-verb.md) — final
  `VOICE_V12` noun/verb recognizability acceptance (the active voice).
- [`validation/learnability-spread.md`](validation/learnability-spread.md) — final
  semantic cluster spread check.
- [`validation/squash.md`](validation/squash.md) — finalized tanh z-score squash choice.
